//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::f32;
use std::fmt::{Display, Formatter};

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------
pub static NULL_COORD: ICoord = (0, 0);

pub static NORTH_DIRECTION: Direction =
    Direction { delta: (0, -1), orientation: Orientation::North };
pub static NORTHEAST_DIRECTION: Direction =
    Direction { delta: (1, -1), orientation: Orientation::Northeast };
pub static EAST_DIRECTION: Direction = Direction { delta: (1, 0), orientation: Orientation::East };
pub static SOUTHEAST_DIRECTION: Direction =
    Direction { delta: (1, 1), orientation: Orientation::Southeast };
pub static SOUTH_DIRECTION: Direction =
    Direction { delta: (0, 1), orientation: Orientation::South };
pub static SOUTHWEST_DIRECTION: Direction =
    Direction { delta: (-1, 1), orientation: Orientation::Southwest };
pub static WEST_DIRECTION: Direction =
    Direction { delta: (-1, 0), orientation: Orientation::West };
pub static NORTHWEST_DIRECTION: Direction =
    Direction { delta: (-1, -1), orientation: Orientation::Northwest };
pub static NULL_DIRECTION: Direction = Direction { delta: (0, 0), orientation: Orientation::Null };

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
#[repr(u8)]
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
    // Delta x and y values of the direction.
    delta: ICoord,
    // Orientation of the direction.
    orientation: Orientation,
}

impl Direction {
    //---------------------------------------------------------------------------------------------
    // Returns the dx of the direction.
    //---------------------------------------------------------------------------------------------
    pub fn dx(&self) -> i32 {
        self.delta.0
    }

    //---------------------------------------------------------------------------------------------
    // Returns the dy of the direction.
    //---------------------------------------------------------------------------------------------
    pub fn dy(&self) -> i32 {
        self.delta.1
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
    pub fn delta(&self) -> ICoord {
        self.delta
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
    pub fn closest_cardinal_direction((x1, y1): ICoord, (x2, y2): ICoord) -> Direction {
        let (dx, dy) = (x2 - x1, y2 - y1);

        // TODO: Why did GoRogue return NULL here?
        // if dx == 0 || dy == 0 {
        //     return NULL_DIRECTION;
        // }

        let angle = (dy as f32).atan2(dx as f32);
        let mut degree = angle * (180.0 / f32::consts::PI);
        degree += 450.0; // Rotate angle so that it is all positive with 0 up.
        degree %= 360.0; // Normalize angle to 0-360.

        println!("degree: {}", degree);

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
    pub fn closest_direction((x1, y1): ICoord, (x2, y2): ICoord) -> Direction {
        let (dx, dy) = (x2 - x1, y2 - y1);

        // TODO: Why did GoRogue return NULL here?
        // if dx == 0 || dy == 0 {
        //     return NULL_DIRECTION;
        // }

        let angle = (dy as f32).atan2(dx as f32);
        let mut degree = angle * 180.0 / f32::consts::PI;
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

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.orientation {
            Orientation::North => write!(f, "Orientation::North"),
            Orientation::Northeast => write!(f, "Orientation::Northeast"),
            Orientation::East => write!(f, "Orientation::East"),
            Orientation::Southeast => write!(f, "Orientation::Southeast"),
            Orientation::South => write!(f, "Orientation::South"),
            Orientation::Southwest => write!(f, "Orientation::Southwest"),
            Orientation::West => write!(f, "Orientation::West"),
            Orientation::Northwest => write!(f, "Orientation::Northwest"),
            Orientation::Null => write!(f, "Orientation::Null"),
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Tests.
//-------------------------------------------------------------------------------------------------

#[test]
fn test_direction_closest_cardinal_direction() {
    assert_eq!(Direction::closest_cardinal_direction((1, 1), (2, 1)), EAST_DIRECTION);
    assert_eq!(Direction::closest_cardinal_direction((1, 1), (2, 10)), SOUTH_DIRECTION);
    assert_eq!(Direction::closest_cardinal_direction((7, 2), (0, 2)), WEST_DIRECTION);
    assert_eq!(Direction::closest_cardinal_direction((1, 1), (0, 0)), NORTH_DIRECTION);
}

#[test]
fn test_direction_closest_direction() {
    assert_eq!(Direction::closest_direction((1, 1), (2, 1)), EAST_DIRECTION);
    assert_eq!(Direction::closest_direction((1, 1), (1, 2)), SOUTH_DIRECTION);
    assert_eq!(Direction::closest_direction((7, 2), (0, 2)), WEST_DIRECTION);
    assert_eq!(Direction::closest_direction((1, 1), (1, 0)), NORTH_DIRECTION);
    assert_eq!(Direction::closest_direction((1, 1), (0, 0)), NORTHWEST_DIRECTION);
    assert_eq!(Direction::closest_direction((1, 1), (2, 2)), SOUTHEAST_DIRECTION);
    assert_eq!(Direction::closest_direction((1, 1), (0, 2)), SOUTHWEST_DIRECTION);
    assert_eq!(Direction::closest_direction((1, 1), (2, 0)), NORTHEAST_DIRECTION);
}
