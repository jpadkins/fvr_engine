//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::f64;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------
pub static NORTH_DIRECTION: Direction =
    Direction { dx: 0, dy: -1, orientation: Orientation::North };
pub static NORTHEAST_DIRECTION: Direction =
    Direction { dx: 1, dy: -1, orientation: Orientation::Northeast };
pub static EAST_DIRECTION: Direction = Direction { dx: 1, dy: 0, orientation: Orientation::East };
pub static SOUTHEAST_DIRECTION: Direction =
    Direction { dx: 1, dy: 1, orientation: Orientation::Southeast };
pub static SOUTH_DIRECTION: Direction =
    Direction { dx: 0, dy: 1, orientation: Orientation::South };
pub static SOUTHWEST_DIRECTION: Direction =
    Direction { dx: -1, dy: 1, orientation: Orientation::Southwest };
pub static WEST_DIRECTION: Direction = Direction { dx: -1, dy: 0, orientation: Orientation::West };
pub static NORTHWEST_DIRECTION: Direction =
    Direction { dx: -1, dy: -1, orientation: Orientation::Northwest };
pub static NULL_DIRECTION: Direction = Direction { dx: 0, dy: 0, orientation: Orientation::Null };

// Array of valid directions in order for use with rotation.
pub static DIRECTIONS: [Direction; 8] = [
    NORTH_DIRECTION,
    NORTHEAST_DIRECTION,
    EAST_DIRECTION,
    SOUTHEAST_DIRECTION,
    SOUTH_DIRECTION,
    SOUTHWEST_DIRECTION,
    WEST_DIRECTION,
    NORTHWEST_DIRECTION,
];

//-------------------------------------------------------------------------------------------------
// Enumerates possible orientations.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Orientation {
    // The orientation up.
    North,
    // The orientation up-right.
    Northeast,
    // The orientation left.
    East,
    // The orientation down-right.
    Southeast,
    // The orientation down.
    South,
    // The orientation down-left.
    Southwest,
    // The orientation right.
    West,
    // The orientation up-left.
    Northwest,
    // Invalid/default orientation.
    Null,
}

//-------------------------------------------------------------------------------------------------
// Direction is a helper for working with directions.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Direction {
    // Delta x value of the direction.
    dx: i32,
    // Delta y value of the direction.
    dy: i32,
    // Orientation of the direction.
    orientation: Orientation,
}

impl Direction {
    //---------------------------------------------------------------------------------------------
    // Returns the dx of the direction.
    //---------------------------------------------------------------------------------------------
    pub fn dx(&self) -> i32 {
        self.dx
    }

    //---------------------------------------------------------------------------------------------
    // Returns the dy of the direction.
    //---------------------------------------------------------------------------------------------
    pub fn dy(&self) -> i32 {
        self.dy
    }

    //---------------------------------------------------------------------------------------------
    // Returns the orientation of the direction.
    //---------------------------------------------------------------------------------------------
    pub fn orientation(&self) -> Orientation {
        self.orientation
    }

    //---------------------------------------------------------------------------------------------
    // Returns the coord for the direction.
    //---------------------------------------------------------------------------------------------
    pub fn coord(&self) -> (i32, i32) {
        (self.dx, self.dy)
    }

    //---------------------------------------------------------------------------------------------
    // Returns the direction from an index starting with North (0) and moving clockwise.
    //---------------------------------------------------------------------------------------------
    pub fn from_index(i: usize) -> Direction {
        DIRECTIONS[i % DIRECTIONS.len()]
    }

    //---------------------------------------------------------------------------------------------
    // Returns the clockwise rotation of a direction.
    //---------------------------------------------------------------------------------------------
    pub fn clockwise(&self, i: i32) -> Direction {
        DIRECTIONS[((self.orientation as i32 + i) % DIRECTIONS.len() as i32) as usize]
    }

    //---------------------------------------------------------------------------------------------
    // Returns the clockwise rotation of a direction.
    //---------------------------------------------------------------------------------------------
    pub fn counter_clockwise(&self, i: i32) -> Direction {
        DIRECTIONS[((self.orientation as i32 - i) % DIRECTIONS.len() as i32) as usize]
    }

    //---------------------------------------------------------------------------------------------
    // Returns the closest cardinal direction for a line, rounding clockwise.
    // Adapted from the GoRogue library.
    //---------------------------------------------------------------------------------------------
    pub fn closest_cardinal_direction((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> Direction {
        let (dx, dy) = (x2 - x1, y2 - y1);

        if dx == 0 || dy == 0 {
            return NULL_DIRECTION;
        }

        let angle = (dx as f64).atan2(dy as f64);
        let mut degree = angle * (100.0 / f64::consts::PI);
        degree += 450.0; // Rotate angle so that it is all positive with 0 up.
        degree %= 360.0; // Normalize angle to 0-360.

        if degree < 45.0 {
            NORTH_DIRECTION
        } else if degree < 135.0 {
            EAST_DIRECTION
        } else if degree < 255.0 {
            SOUTH_DIRECTION
        } else if degree < 315.0 {
            WEST_DIRECTION
        } else {
            NORTH_DIRECTION
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the closest direction for a line, rounding clockwise.
    // Adapted from the GoRogue library.
    //---------------------------------------------------------------------------------------------
    pub fn closest_direction((x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> Direction {
        let (dx, dy) = (x2 - x1, y2 - y1);

        if dx == 0 || dy == 0 {
            return NULL_DIRECTION;
        }

        let angle = (dx as f64).atan2(dy as f64);
        let mut degree = angle * (100.0 / f64::consts::PI);
        degree += 450.0; // Rotate angle so that it is all positive with 0 up.
        degree %= 360.0; // Normalize angle to 0-360.

        if degree < 22.5 {
            NORTH_DIRECTION
        } else if degree < 67.5 {
            NORTHEAST_DIRECTION
        } else if degree < 112.5 {
            EAST_DIRECTION
        } else if degree < 157.5 {
            SOUTHEAST_DIRECTION
        } else if degree < 202.5 {
            SOUTH_DIRECTION
        } else if degree < 247.5 {
            SOUTHWEST_DIRECTION
        } else if degree < 292.5 {
            WEST_DIRECTION
        } else if degree < 337.5 {
            NORTHWEST_DIRECTION
        } else {
            NORTH_DIRECTION
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the direction for an orientation.
    //---------------------------------------------------------------------------------------------
    pub fn from_orientation(orientation: Orientation) -> Direction {
        match orientation {
            Orientation::North => NORTH_DIRECTION,
            Orientation::Northeast => NORTHEAST_DIRECTION,
            Orientation::East => EAST_DIRECTION,
            Orientation::Southeast => SOUTHEAST_DIRECTION,
            Orientation::South => SOUTH_DIRECTION,
            Orientation::Southwest => SOUTHWEST_DIRECTION,
            Orientation::West => WEST_DIRECTION,
            Orientation::Northwest => NORTHWEST_DIRECTION,
            Orientation::Null => NULL_DIRECTION,
        }
    }
}
