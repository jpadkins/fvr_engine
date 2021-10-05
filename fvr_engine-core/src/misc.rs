//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
pub const INVALID_ICOORD: ICoord = (-1, -1);

// Conversion value for radian/degrees.
pub const DEGREE_PER_RADIAN: f32 = 1.0 / 360.0;

//-------------------------------------------------------------------------------------------------
// Aliases for commonly used tuple types.
//-------------------------------------------------------------------------------------------------
pub type ICoord = (i32, i32);

//-------------------------------------------------------------------------------------------------
// Misc provides a static API of misc. helper functions.
//-------------------------------------------------------------------------------------------------
pub struct Misc;

impl Misc {
    //---------------------------------------------------------------------------------------------
    // Returns the 1D index for a coord.
    //---------------------------------------------------------------------------------------------
    pub fn index_2d((x, y): ICoord, width: i32) -> usize {
        (x + (y * width)) as usize
    }

    //---------------------------------------------------------------------------------------------
    // Returns the coord for a 1D index.
    //---------------------------------------------------------------------------------------------
    pub fn reverse_index_2d(index: usize, width: i32) -> ICoord {
        ((index % width as usize) as i32, (index / width as usize) as i32)
    }

    //---------------------------------------------------------------------------------------------
    // Finds the origin (top left or top right) for centering content within larger bounds.
    //---------------------------------------------------------------------------------------------
    pub fn centered_origin(dimension: i32, bounding_dimension: i32) -> i32 {
        (bounding_dimension - dimension) / 2
    }

    //---------------------------------------------------------------------------------------------
    // Approximation of atan2 that scales result to range [0.0..1.0].
    // Adapted from SquidLib.
    //---------------------------------------------------------------------------------------------
    pub fn scaled_atan2(x: f64, y: f64) -> f64 {
        // TODO: Why did Squidlib return 0.0 in this case?
        // if x == 0.0 || y == 0.0 {
        //     return 0.0;
        // }

        let ax = x.abs();
        let ay = y.abs();

        let r = if ax < ay {
            let a = ax / ay;
            let s = a * a;
            0.25 - (((-0.0464964749 * s + 0.15931422) * s - 0.327622764) * s * a + a)
                * 0.15915494309189535
        } else {
            let a = ay / ax;
            let s = a * a;
            (((-0.0464964749 * s + 0.15931422) * s - 0.327622764) * s * a + a)
                * 0.15915494309189535
        };

        if x < 0.0 {
            if y < 0.0 {
                0.5 + r
            } else {
                0.5 - r
            }
        } else if y < 0.0 {
            1.0 - r
        } else {
            r
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the angle between two points in degrees in the range [0.0..360.0].
    //---------------------------------------------------------------------------------------------
    pub fn angle_between((x1, y1): ICoord, (x2, y2): ICoord) -> f32 {
        let (dx, dy) = (x2 - x1, y2 - y1);
        let angle = (dy as f32).atan2(dx as f32);
        let degrees = angle * (180.0 / std::f32::consts::PI);
        (degrees + 360.0) % 360.0
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for converting a i32 coord to i32.
    //---------------------------------------------------------------------------------------------
    pub fn utoi((x, y): (u32, u32)) -> ICoord {
        (x as i32, y as i32)
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for converting an i32 coord to i32.
    //---------------------------------------------------------------------------------------------
    pub fn itou((x, y): ICoord) -> (u32, u32) {
        (x as u32, y as u32)
    }
}

//-------------------------------------------------------------------------------------------------
// Helper macro for iterating over a cartesian product given xy coords.
//-------------------------------------------------------------------------------------------------
#[macro_export]
macro_rules! xy_iter {
    ($x:ident, $y:ident, $width:expr, $height:expr, $work:expr) => {
        for $x in 0..$width {
            for $y in 0..$height {
                $work
            }
        }
    };
}

//-------------------------------------------------------------------------------------------------
// Helper macro for iterating over a cartesian product given dimensions tuple.
//-------------------------------------------------------------------------------------------------
#[macro_export]
macro_rules! xy_tuple_iter {
    ($x:ident, $y:ident, $dimensions:expr, $work:expr) => {
        for $x in 0..$dimensions.0 {
            for $y in 0..$dimensions.1 {
                $work
            }
        }
    };
}

//-------------------------------------------------------------------------------------------------
// Tests.
//-------------------------------------------------------------------------------------------------

#[test]
fn test_2d_index() {
    assert_eq!(Misc::index_2d((5, 6), 10), 65);
    assert_eq!(Misc::reverse_index_2d(65, 10), (5, 6));
}
