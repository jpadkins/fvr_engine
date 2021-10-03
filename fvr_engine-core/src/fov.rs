//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use fnv::FnvHashSet;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::adjacency::*;
use crate::dijkstra_map::*;
use crate::distance::*;
use crate::grid_map::*;
use crate::map2d::*;
use crate::misc::*;

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
        match b {
            true => Self::Transparent,
            false => Self::Opaque,
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

// Impl conversion between dijkstra state for convenience.
impl From<DijkstraState> for Transparency {
    fn from(dijkstra_state: DijkstraState) -> Self {
        match dijkstra_state {
            DijkstraState::Blocked => Self::Opaque,
            DijkstraState::Goal { .. } | DijkstraState::Passable => Self::Transparent,
        }
    }
}

impl From<Transparency> for DijkstraState {
    fn from(transparency: Transparency) -> Self {
        match transparency {
            Transparency::Opaque => DijkstraState::Blocked,
            Transparency::Transparent => DijkstraState::Passable,
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Fov calculates field of view, given an input opacity states and source coord.
// NOTE: An alternative constructor and calculate/_limited methods are provided for a "thin" fov in
//  which no internal state grid map is managed. This allows for cutting down on memory usage in
//  the case where multiple structs share the same state (since the grid map's internal data vec
//  would not need to be allocated).
//-------------------------------------------------------------------------------------------------
pub struct Fov {
    // Stores the opaque/transparent state of the underlying map. false = opaque.
    states: Option<GridMap<Transparency>>,
    // Stores the calculated light values. > 0.0 means the coord is visible.
    light: GridMap<f32>,
    // Coords in the current fov.
    current_fov: FnvHashSet<(u8, u8)>,
    // Coords in the previous fov.
    previous_fov: FnvHashSet<(u8, u8)>,
    // The distance method.
    distance: Distance,
}

//-------------------------------------------------------------------------------------------------
// Implementation for both cast_shadow/thin methods below to avoid duplication.
// Adapted from http://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting and the
// GoRogue library.
//-------------------------------------------------------------------------------------------------
macro_rules! cast_shadow_impl {
    (
        $self:expr,
        $row:expr,
        $start:expr,
        $end:expr,
        $xx:expr,
        $xy:expr,
        $yx:expr,
        $yy:expr,
        $radius:expr,
        $origin:expr,
        $decay:expr,
        $states:expr,
        $d:ident,
        $slope_left:ident,
        $recur:expr,
    ) => {
        if $start < $end {
            return;
        }

        let mut new_start = 0.0;
        let mut blocked = false;
        let mut $d = $row;

        while $d as f32 <= $radius && $d < ($self.width() + $self.height()) as i32 && !blocked {
            let dy = -$d;
            let mut dx = dy;

            while dx <= 0 {
                let current_x = $origin.0 as i32 + dx * $xx + dy * $xy;
                let current_y = $origin.1 as i32 + dx * $yx + dy * $yy;
                let $slope_left = (dx as f32 - 0.5) / (dy as f32 + 0.5);
                let slope_right = (dx as f32 + 0.5) / (dy as f32 - 0.5);

                if !(current_x >= 0
                    && current_y >= 0
                    && current_x < $self.width() as i32
                    && current_y < $self.height() as i32)
                    || $start < slope_right
                {
                    dx += 1;
                    continue;
                } else if $end > $slope_left {
                    break;
                }

                let delta_radius = $self.distance.calculate_slope(dx as f32, dy as f32);
                let coord = (current_x, current_y);

                if delta_radius <= $radius {
                    let brightness = 1.0 - $decay * delta_radius;
                    *$self.light.get_xy_mut(coord) = brightness;

                    if brightness > 0.0 {
                        $self.current_fov.insert((coord.0 as u8, coord.1 as u8));
                    }
                }

                let opaque = Into::<Transparency>::into($states.get_xy(coord).clone())
                    == Transparency::Opaque;

                if blocked {
                    if opaque {
                        new_start = slope_right;
                    } else {
                        blocked = false;
                        $start = new_start;
                    }
                } else if opaque && ($d as f32) < $radius {
                    blocked = true;
                    $recur
                    new_start = slope_right;
                }

                dx += 1;
            }

            $d += 1;
        }
    };
}

//---------------------------------------------------------------------------------------------
// Implementation for both cast_shadow_limited/_thin methods below to avoid duplication.
// Adapted from http://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting and the
// GoRogue library.
//---------------------------------------------------------------------------------------------
macro_rules! cast_shadow_limited_impl {
    (
        $self:expr,
        $row:expr,
        $start:expr,
        $end:expr,
        $xx:expr,
        $xy:expr,
        $yx:expr,
        $yy:expr,
        $radius:expr,
        $origin:expr,
        $decay:expr,
        $angle:expr,
        $span:expr,
        $states:expr,
        $d:ident,
        $slope_left:ident,
        $recur:expr,
    ) => {
        if $start < $end {
            return;
        }

        let mut new_start = 0.0;
        let mut blocked = false;
        let mut $d = $row;

        while $d as f32 <= $radius && $d < ($self.width() + $self.height()) as i32 && !blocked {
            let dy = -$d;
            let mut dx = dy;

            while dx <= 0 {
                let current_x = $origin.0 as i32 + dx * $xx + dy * $xy;
                let current_y = $origin.1 as i32 + dx * $yx + dy * $yy;
                let $slope_left = (dx as f32 - 0.5) / (dy as f32 + 0.5);
                let slope_right = (dx as f32 + 0.5) / (dy as f32 - 0.5);

                if !(current_x >= 0
                    && current_y >= 0
                    && current_x < $self.width() as i32
                    && current_y < $self.height() as i32)
                    || $start < slope_right
                {
                    dx += 1;
                    continue;
                } else if $end > $slope_left {
                    break;
                }

                let delta_radius = $self.distance.calculate_slope(dx as f32, dy as f32);
                let atan2 = ($angle
                    - Misc::scaled_atan2(
                        (current_x - $origin.0 as i32) as f64,
                        (current_y - $origin.1 as i32) as f64,
                    ) as f32)
                    .abs();
                let coord = (current_x, current_y);

                if delta_radius <= $radius && (atan2 <= $span * 0.5 || atan2 >= 1.0 - $span * 0.5)
                {
                    let brightness = 1.0 - $decay * delta_radius;
                    *$self.light.get_xy_mut(coord) = brightness;

                    if brightness > 0.0 {
                        $self.current_fov.insert((coord.0 as u8, coord.1 as u8));
                    }
                }

                let opaque = Into::<Transparency>::into($states.get_xy(coord).clone())
                    == Transparency::Opaque;

                if blocked {
                    if opaque {
                        new_start = slope_right;
                    } else {
                        blocked = false;
                        $start = new_start;
                    }
                } else if opaque && ($d as f32) < $radius {
                    blocked = true;
                    $recur
                    new_start = slope_right;
                }

                dx += 1;
            }

            $d += 1;
        }
    };
}

impl Fov {
    //---------------------------------------------------------------------------------------------
    // Creates a new fov.
    //---------------------------------------------------------------------------------------------
    pub fn new(dimensions: ICoord, distance: Distance) -> Self {
        Self {
            states: Some(GridMap::new(dimensions)),
            light: GridMap::new(dimensions),
            current_fov: FnvHashSet::default(),
            previous_fov: FnvHashSet::default(),
            distance,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Creates a new fov.
    //---------------------------------------------------------------------------------------------
    pub fn new_thin(dimensions: ICoord, distance: Distance) -> Self {
        Self {
            states: None,
            light: GridMap::new(dimensions),
            current_fov: FnvHashSet::default(),
            previous_fov: FnvHashSet::default(),
            distance,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the input states of the fov.
    // Panics if called on a thin fov.
    //---------------------------------------------------------------------------------------------
    pub fn states(&self) -> &GridMap<Transparency> {
        self.states.as_ref().unwrap()
    }

    //---------------------------------------------------------------------------------------------
    // Returns a mut ref to the input states of the fov.
    // Panics if called on a thin fov.
    //---------------------------------------------------------------------------------------------
    pub fn states_mut(&mut self) -> &mut GridMap<Transparency> {
        self.states.as_mut().unwrap()
    }

    //---------------------------------------------------------------------------------------------
    // Recursive shadowcasting implementation.
    // NOTE: Panics if called on a thin fov.
    //---------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn cast_shadow(
        &mut self,
        row: i32,
        mut start: f32,
        end: f32,
        xx: i32,
        xy: i32,
        yx: i32,
        yy: i32,
        radius: f32,
        origin: ICoord,
        decay: f32,
    ) {
        cast_shadow_impl!(
            self,
            row,
            start,
            end,
            xx,
            xy,
            yx,
            yy,
            radius,
            origin,
            decay,
            self.states.as_ref().unwrap(),
            d,
            slope_left,
            { self.cast_shadow(d + 1, start, slope_left, xx, xy, yx, yy, radius, origin, decay,) },
        );
    }

    //---------------------------------------------------------------------------------------------
    // Recursive shadowcasting implementation. Intended for usage on a thin fov.
    //---------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn cast_shadow_thin<M, T>(
        &mut self,
        row: i32,
        mut start: f32,
        end: f32,
        xx: i32,
        xy: i32,
        yx: i32,
        yy: i32,
        radius: f32,
        origin: ICoord,
        decay: f32,
        states: &M,
    ) where
        M: Map2d<T>,
        T: Map2dType + Into<Transparency>,
    {
        cast_shadow_impl!(
            self,
            row,
            start,
            end,
            xx,
            xy,
            yx,
            yy,
            radius,
            origin,
            decay,
            states,
            d,
            slope_left,
            {
                self.cast_shadow_thin(
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
                    states,
                );
            },
        );
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for resetting state before recalculating.
    //---------------------------------------------------------------------------------------------
    fn calculate_cleanup(&mut self, origin: ICoord, radius: f32) -> (f32, f32) {
        // Calculate decay.
        let radius = radius.max(1.0);
        let decay = 1.0 / (radius + 1.0);

        // Reset the fov hash sets.
        self.previous_fov.clear();
        self.previous_fov.extend(self.current_fov.drain());

        // Reset the light map.
        self.light.data_mut().fill(0.0);

        // Handle the origin coord.
        *self.light.get_xy_mut(origin) = 1.0;
        self.current_fov.insert((origin.0 as u8, origin.1 as u8));

        (radius, decay)
    }

    //---------------------------------------------------------------------------------------------
    // Calculates the fov.
    // NOTE: Panics if called on a thin fov.
    //---------------------------------------------------------------------------------------------
    pub fn calculate(&mut self, origin: ICoord, radius: f32) {
        if self.states.is_none() {
            panic!("calculate called on thin fov!");
        }

        let (radius, decay) = self.calculate_cleanup(origin, radius);

        // Begin shadowcasting.
        for dir in Adjacency::Diagonals.iter() {
            self.cast_shadow(1, 1.0, 0.0, 0, dir.dx(), dir.dy(), 0, radius, origin, decay);
            self.cast_shadow(1, 1.0, 0.0, dir.dx(), 0, 0, dir.dy(), radius, origin, decay);
        }
    }

    //---------------------------------------------------------------------------------------------
    // Calculates the fov. Intended for usage with thin fov.
    //---------------------------------------------------------------------------------------------
    pub fn calculate_thin<M, T>(&mut self, origin: ICoord, radius: f32, states: &M)
    where
        M: Map2d<T>,
        T: Map2dType + Into<Transparency>,
    {
        let (radius, decay) = self.calculate_cleanup(origin, radius);

        // Begin shadowcasting.
        for dir in Adjacency::Diagonals.iter() {
            self.cast_shadow_thin(
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
                states,
            );
            self.cast_shadow_thin(
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
                states,
            );
        }
    }

    //---------------------------------------------------------------------------------------------
    // Recursive shadowcasting implementation for limited wedge.
    // NOTE: Panics if called on a thin fov.
    //---------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn cast_shadow_limited(
        &mut self,
        row: i32,
        mut start: f32,
        end: f32,
        xx: i32,
        xy: i32,
        yx: i32,
        yy: i32,
        radius: f32,
        origin: ICoord,
        decay: f32,
        angle: f32,
        span: f32,
    ) {
        cast_shadow_limited_impl!(
            self,
            row,
            start,
            end,
            xx,
            xy,
            yx,
            yy,
            radius,
            origin,
            decay,
            angle,
            span,
            self.states.as_ref().unwrap(),
            d,
            slope_left,
            {
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
            },
        );
    }

    //---------------------------------------------------------------------------------------------
    // Recursive shadowcasting implementation for limited wedge. Intended for usage on a thin fov.
    //---------------------------------------------------------------------------------------------
    #[allow(clippy::too_many_arguments)]
    fn cast_shadow_limited_thin<M, T>(
        &mut self,
        row: i32,
        mut start: f32,
        end: f32,
        xx: i32,
        xy: i32,
        yx: i32,
        yy: i32,
        radius: f32,
        origin: ICoord,
        decay: f32,
        angle: f32,
        span: f32,
        states: &M,
    ) where
        M: Map2d<T>,
        T: Map2dType + Into<Transparency>,
    {
        cast_shadow_limited_impl!(
            self,
            row,
            start,
            end,
            xx,
            xy,
            yx,
            yy,
            radius,
            origin,
            decay,
            angle,
            span,
            states,
            d,
            slope_left,
            {
                self.cast_shadow_limited_thin(
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
                    states,
                );
            },
        );
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for resetting state before limited fov recalculating.
    //---------------------------------------------------------------------------------------------
    fn calculate_limited_cleanup(
        &mut self,
        origin: ICoord,
        radius: f32,
        angle: &mut f32,
        span: &mut f32,
    ) -> (f32, f32) {
        // Calculate decay.
        let radius = radius.max(1.0);
        let decay = 1.0 / (radius + 1.0);

        // Normalize the angle and span as % of a circle.
        *angle = (*angle % 360.0) * (1.0 / 360.0);
        *span *= 1.0 / 360.0;

        // Reset the fov hash sets.
        self.previous_fov.clear();
        self.previous_fov.extend(self.current_fov.drain());

        // Reset the light map.
        self.light.data_mut().fill(0.0);

        // Handle the origin coord.
        *self.light.get_xy_mut(origin) = 1.0;
        self.current_fov.insert((origin.0 as u8, origin.1 as u8));

        (radius, decay)
    }

    //---------------------------------------------------------------------------------------------
    // Calculates a limited (wedge) fov.
    // NOTE: Panics if called on a thin fov.
    //---------------------------------------------------------------------------------------------
    pub fn calculate_limited(
        &mut self,
        origin: ICoord,
        radius: f32,
        mut angle: f32,
        mut span: f32,
    ) {
        if self.states.is_none() {
            panic!("calculate_limited called on thin fov!");
        }

        let (radius, decay) =
            self.calculate_limited_cleanup(origin, radius, &mut angle, &mut span);

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

    //---------------------------------------------------------------------------------------------
    // Calculates a limited (wedge) fov. Intended for usage on a thin fov.
    //---------------------------------------------------------------------------------------------
    pub fn calculate_limited_thin<M, T>(
        &mut self,
        origin: ICoord,
        radius: f32,
        mut angle: f32,
        mut span: f32,
        states: &M,
    ) where
        M: Map2d<T>,
        T: Map2dType + Into<Transparency>,
    {
        if self.states.is_none() {
            panic!("calculate_limited called on thin fov!");
        }

        let (radius, decay) =
            self.calculate_limited_cleanup(origin, radius, &mut angle, &mut span);

        // Perform shadowcasting.
        self.cast_shadow_limited_thin(
            1, 1.0, 0.0, 0, 1, 1, 0, radius, origin, decay, angle, span, states,
        );
        self.cast_shadow_limited_thin(
            1, 1.0, 0.0, 1, 0, 0, 1, radius, origin, decay, angle, span, states,
        );

        self.cast_shadow_limited_thin(
            1, 1.0, 0.0, 0, -1, 1, 0, radius, origin, decay, angle, span, states,
        );
        self.cast_shadow_limited_thin(
            1, 1.0, 0.0, -1, 0, 0, 1, radius, origin, decay, angle, span, states,
        );

        self.cast_shadow_limited_thin(
            1, 1.0, 0.0, 0, -1, -1, 0, radius, origin, decay, angle, span, states,
        );
        self.cast_shadow_limited_thin(
            1, 1.0, 0.0, -1, 0, 0, -1, radius, origin, decay, angle, span, states,
        );

        self.cast_shadow_limited_thin(
            1, 1.0, 0.0, 0, 1, -1, 0, radius, origin, decay, angle, span, states,
        );
        self.cast_shadow_limited_thin(
            1, 1.0, 0.0, 1, 0, 0, -1, radius, origin, decay, angle, span, states,
        );
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dView for GridMap.
//-------------------------------------------------------------------------------------------------
impl Map2dView for Fov {
    type Type = f32;

    //---------------------------------------------------------------------------------------------
    // Return the width of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn width(&self) -> i32 {
        self.light.width()
    }

    //---------------------------------------------------------------------------------------------
    // Return the height of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn height(&self) -> i32 {
        self.light.height()
    }

    //---------------------------------------------------------------------------------------------
    // Return the dimensions of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn dimensions(&self) -> ICoord {
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
    fn get_xy(&self, xy: ICoord) -> &Self::Type {
        self.light.get_xy(xy)
    }
}
