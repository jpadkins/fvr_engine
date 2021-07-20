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
use crate::traits::*;

//-------------------------------------------------------------------------------------------------
// Fov calculates shadowcasting field of view, given an opacity map and source coord.
//-------------------------------------------------------------------------------------------------
pub struct Fov {
    opacity_map: GridMap<bool>,
    light: GridMap<f64>,
    current_fov: HashSet<(u32, u32)>,
    previous_fov: HashSet<(u32, u32)>,
}

impl Fov {
    //---------------------------------------------------------------------------------------------
    // Creates a new fov.
    //---------------------------------------------------------------------------------------------
    pub fn new(dimensions: (u32, u32)) -> Self {
        Self {
            opacity_map: GridMap::new(dimensions.0, dimensions.1),
            light: GridMap::new(dimensions.0, dimensions.1),
            current_fov: HashSet::new(),
            previous_fov: HashSet::new(),
        }
    }

    pub fn opacity_map(&self) -> &GridMap<bool> {
        &self.opacity_map
    }

    pub fn opacity_map_mut(&mut self) -> &mut GridMap<bool> {
        &mut self.opacity_map
    }

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
        origin: (u32, u32),
        decay: f64,
        distance: Distance,
    ) {
        if start < end {
            return;
        }

        let mut new_start = 0.0;
        let mut blocked = false;
        let mut d = row;

        while d as f64 <= radius && d < (self.width() + self.height()) as i32 && !blocked {
            let dy = -1 * d;
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

                let delta_radius = distance.calculate_slope(dx as f64, dy as f64);
                let current_coord = (current_x as u32, current_y as u32);

                if delta_radius <= radius {
                    let brightness = 1.0 - decay * delta_radius;
                    *self.light.get_xy_mut(current_coord) = brightness;

                    if brightness > 0.0 {
                        self.current_fov.insert(current_coord);
                    }
                }

                if blocked {
                    if !self.opacity_map.get_xy(current_coord) {
                        new_start = slope_right;
                    } else {
                        blocked = false;
                        start = new_start;
                    }
                } else {
                    if !self.opacity_map.get_xy(current_coord) && (d as f64) < radius {
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
                            distance,
                        );
                        new_start = slope_right;
                    }
                }

                dx += 1;
            }

            d += 1;
        }
    }

    pub fn calculate(&mut self, origin: (u32, u32), radius: f64, distance: Distance) {
        // Calculate decay.
        let radius = radius.max(1.0);
        let decay = 1.0 / (radius + 1.0);

        // Reset the fov hash sets.
        self.previous_fov = self.current_fov.clone();
        self.current_fov = HashSet::new();

        // Reset the light map.
        map2d_iter_mut!(self.light, item, {
            *item = 0.0;
        });

        // Handle the origin coord.
        *self.light.get_xy_mut(origin) = 1.0;
        self.current_fov.insert(origin);

        for dir in Adjacency::Diagonals.iter() {
            self.cast_shadow(
                1,
                1.0,
                0.0,
                0,
                dir.dx(),
                dir.dy(),
                0,
                radius,
                origin,
                decay,
                distance,
            );
            self.cast_shadow(
                1,
                1.0,
                0.0,
                dir.dx(),
                0,
                0,
                dir.dy(),
                radius,
                origin,
                decay,
                distance,
            );
        }
    }
}

impl Map2dView for Fov {
    type Type = f64;

    fn width(&self) -> u32 {
        self.opacity_map.width()
    }

    fn height(&self) -> u32 {
        self.opacity_map.height()
    }

    fn get(&self, index: usize) -> &Self::Type {
        self.light.get(index)
    }

    fn get_xy(&self, xy: (u32, u32)) -> &Self::Type {
        self.light.get_xy(xy)
    }
}
