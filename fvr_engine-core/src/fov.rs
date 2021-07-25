//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::collections::HashSet;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::adjacency::*;
use crate::distance::*;
use crate::grid_map::*;
use crate::map2d_iter_mut;
use crate::misc::*;
use crate::traits::*;

//-------------------------------------------------------------------------------------------------
// Enumerates the possible transparency input states for the underlying map.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Transparency {
    // Blocks visibility.
    Opaque,
    // Does not block visibility.
    Transparent,
}

impl Default for Transparency {
    fn default() -> Self {
        Transparency::Transparent
    }
}

// Impl conversions between bool for convenience.
impl From<bool> for Transparency {
    fn from(b: bool) -> Self {
        if b {
            Self::Transparent
        } else {
            Self::Opaque
        }
    }
}
impl From<Transparency> for bool {
    fn from(transparency: Transparency) -> Self {
        match transparency {
            Transparency::Opaque => false,
            Transparency::Transparent => true,
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Fov calculates field of view, given an input opacity states and source coord.
//-------------------------------------------------------------------------------------------------
pub struct Fov {
    // Stores the opaque/transparent state of the underlying map. false = opaque.
    states: GridMap<Transparency>,
    // Stores the calculated light values. > 0.0 means the coord is visible.
    light: GridMap<f64>,
    // Coords in the current fov.
    current_fov: HashSet<UCoord>,
    // Coords in the previous fov.
    previous_fov: HashSet<UCoord>,
    // The distance method.
    distance: Distance,
}

impl Fov {
    //---------------------------------------------------------------------------------------------
    // Creates a new fov.
    //---------------------------------------------------------------------------------------------
    pub fn new(dimensions: UCoord, distance: Distance) -> Self {
        Self {
            states: GridMap::new(dimensions),
            light: GridMap::new(dimensions),
            current_fov: HashSet::new(),
            previous_fov: HashSet::new(),
            distance,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the input states of the fov.
    //---------------------------------------------------------------------------------------------
    pub fn states(&self) -> &GridMap<Transparency> {
        &self.states
    }

    //---------------------------------------------------------------------------------------------
    // Returns a mut ref to the input states of the fov.
    //---------------------------------------------------------------------------------------------
    pub fn states_mut(&mut self) -> &mut GridMap<Transparency> {
        &mut self.states
    }

    //---------------------------------------------------------------------------------------------
    // Recursive shadowcasting implementation.
    // Adapted from http://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting and the
    // GoRogue library.
    //---------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn cast_shadow(
        &mut self,
        row: i32,
        mut start: f64,
        end: f64,
        xx: i32,
        xy: i32,
        yx: i32,
        yy: i32,
        radius: f64,
        origin: UCoord,
        decay: f64,
    ) {
        if start < end {
            return;
        }

        let mut new_start = 0.0;
        let mut blocked = false;
        let mut d = row;

        while d as f64 <= radius && d < (self.width() + self.height()) as i32 && !blocked {
            let dy = -d;
            let mut dx = dy;

            while dx <= 0 {
                let current_x = origin.0 as i32 + dx * xx + dy * xy;
                let current_y = origin.1 as i32 + dx * yx + dy * yy;
                let slope_left = (dx as f64 - 0.5) / (dy as f64 + 0.5);
                let slope_right = (dx as f64 + 0.5) / (dy as f64 - 0.5);

                if !(current_x >= 0
                    && current_y >= 0
                    && current_x < self.width() as i32
                    && current_y < self.height() as i32)
                    || start < slope_right
                {
                    dx += 1;
                    continue;
                } else if end > slope_left {
                    break;
                }

                let delta_radius = self.distance.calculate_slope(dx as f64, dy as f64);
                let current_coord = (current_x as u32, current_y as u32);

                if delta_radius <= radius {
                    let brightness = 1.0 - decay * delta_radius;
                    *self.light.get_xy_mut(current_coord) = brightness;

                    if brightness > 0.0 {
                        self.current_fov.insert(current_coord);
                    }
                }

                let opaque = !bool::from(*self.states.get_xy(current_coord));

                if blocked {
                    if opaque {
                        new_start = slope_right;
                    } else {
                        blocked = false;
                        start = new_start;
                    }
                } else if opaque && (d as f64) < radius {
                    blocked = true;
                    self.cast_shadow(
                        d + 1,
                        start,
                        slope_left,
                        xx,
                        xy,
                        yx,
                        yy,
                        radius,
                        origin,
                        decay,
                    );
                    new_start = slope_right;
                }

                dx += 1;
            }

            d += 1;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Ccalculates the fov.
    //---------------------------------------------------------------------------------------------
    pub fn calculate(&mut self, origin: UCoord, radius: f64) {
        // Calculate decay.
        let radius = radius.max(1.0);
        let decay = 1.0 / (radius + 1.0);

        // Reset the fov hash sets.
        self.previous_fov.clear();
        self.previous_fov.extend(self.current_fov.drain());

        // Reset the light map.
        map2d_iter_mut!(self.light, item, {
            *item = 0.0;
        });

        // Handle the origin coord.
        *self.light.get_xy_mut(origin) = 1.0;
        self.current_fov.insert(origin);

        // Begin shadowcasting.
        for dir in Adjacency::Diagonals.iter() {
            self.cast_shadow(1, 1.0, 0.0, 0, dir.dx(), dir.dy(), 0, radius, origin, decay);
            self.cast_shadow(1, 1.0, 0.0, dir.dx(), 0, 0, dir.dy(), radius, origin, decay);
        }
    }

    //---------------------------------------------------------------------------------------------
    // Recursive shadowcasting implementation for limited wedge.
    // Adapted from http://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting and the
    // GoRogue library.
    //---------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn cast_shadow_limited(
        &mut self,
        row: i32,
        mut start: f64,
        end: f64,
        xx: i32,
        xy: i32,
        yx: i32,
        yy: i32,
        radius: f64,
        origin: UCoord,
        decay: f64,
        angle: f64,
        span: f64,
    ) {
        if start < end {
            return;
        }

        let mut new_start = 0.0;
        let mut blocked = false;
        let mut d = row;

        while d as f64 <= radius && d < (self.width() + self.height()) as i32 && !blocked {
            let dy = -d;
            let mut dx = dy;

            while dx <= 0 {
                let current_x = origin.0 as i32 + dx * xx + dy * xy;
                let current_y = origin.1 as i32 + dx * yx + dy * yy;
                let slope_left = (dx as f64 - 0.5) / (dy as f64 + 0.5);
                let slope_right = (dx as f64 + 0.5) / (dy as f64 - 0.5);

                if !(current_x >= 0
                    && current_y >= 0
                    && current_x < self.width() as i32
                    && current_y < self.height() as i32)
                    || start < slope_right
                {
                    dx += 1;
                    continue;
                } else if end > slope_left {
                    break;
                }

                let delta_radius = self.distance.calculate_slope(dx as f64, dy as f64);
                let atan2 = (angle
                    - Misc::scaled_atan2(
                        (current_x - origin.0 as i32) as f64,
                        (current_y - origin.1 as i32) as f64,
                    ))
                .abs();
                let current_coord = (current_x as u32, current_y as u32);

                if delta_radius <= radius && (atan2 <= span * 0.5 || atan2 >= 1.0 - span * 0.5) {
                    let brightness = 1.0 - decay * delta_radius;
                    *self.light.get_xy_mut(current_coord) = brightness;

                    if brightness > 0.0 {
                        self.current_fov.insert(current_coord);
                    }
                }

                let opaque = !bool::from(*self.states.get_xy(current_coord));

                if blocked {
                    if opaque {
                        new_start = slope_right;
                    } else {
                        blocked = false;
                        start = new_start;
                    }
                } else if opaque && (d as f64) < radius {
                    blocked = true;
                    self.cast_shadow_limited(
                        d + 1,
                        start,
                        slope_left,
                        xx,
                        xy,
                        yx,
                        yy,
                        radius,
                        origin,
                        decay,
                        angle,
                        span,
                    );
                    new_start = slope_right;
                }

                dx += 1;
            }

            d += 1;
        }
    }

    pub fn calculate_limited(
        &mut self,
        origin: UCoord,
        radius: f64,
        mut angle: f64,
        mut span: f64,
    ) {
        // Calculate decay.
        let radius = radius.max(1.0);
        let decay = 1.0 / (radius + 1.0);

        // Normalize the angle and span as % of a circle.
        angle = (angle % 360.0) * (1.0 / 360.0);
        span *= 1.0 / 360.0;

        // Reset the fov hash sets.
        self.previous_fov.clear();
        self.previous_fov.extend(self.current_fov.drain());

        // Reset the light map.
        map2d_iter_mut!(self.light, item, {
            *item = 0.0;
        });

        // Handle the origin coord.
        *self.light.get_xy_mut(origin) = 1.0;
        self.current_fov.insert(origin);

        // Perform shadowcasting.
        self.cast_shadow_limited(1, 1.0, 0.0, 0, 1, 1, 0, radius, origin, decay, angle, span);
        self.cast_shadow_limited(1, 1.0, 0.0, 1, 0, 0, 1, radius, origin, decay, angle, span);

        self.cast_shadow_limited(1, 1.0, 0.0, 0, -1, 1, 0, radius, origin, decay, angle, span);
        self.cast_shadow_limited(1, 1.0, 0.0, -1, 0, 0, 1, radius, origin, decay, angle, span);

        self.cast_shadow_limited(1, 1.0, 0.0, 0, -1, -1, 0, radius, origin, decay, angle, span);
        self.cast_shadow_limited(1, 1.0, 0.0, -1, 0, 0, -1, radius, origin, decay, angle, span);

        self.cast_shadow_limited(1, 1.0, 0.0, 0, 1, -1, 0, radius, origin, decay, angle, span);
        self.cast_shadow_limited(1, 1.0, 0.0, 1, 0, 0, -1, radius, origin, decay, angle, span);
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dView for GridMap.
//-------------------------------------------------------------------------------------------------
impl Map2dView for Fov {
    type Type = f64;

    //---------------------------------------------------------------------------------------------
    // Return the width of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn width(&self) -> u32 {
        self.light.width()
    }

    //---------------------------------------------------------------------------------------------
    // Return the height of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn height(&self) -> u32 {
        self.light.height()
    }

    //---------------------------------------------------------------------------------------------
    // Return the dimensions of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn dimensions(&self) -> UCoord {
        self.light.dimensions()
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get(&self, index: usize) -> &Self::Type {
        self.light.get(index)
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy(&self, xy: UCoord) -> &Self::Type {
        self.light.get_xy(xy)
    }
}
