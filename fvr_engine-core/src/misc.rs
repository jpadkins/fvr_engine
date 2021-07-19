//-------------------------------------------------------------------------------------------------
// Misc provides a static API of misc. helper functions.
//-------------------------------------------------------------------------------------------------
pub struct Misc {}

impl Misc {
    //---------------------------------------------------------------------------------------------
    // Returns the 1D index for a coord.
    //---------------------------------------------------------------------------------------------
    pub fn index_2d((x, y): (u32, u32), width: u32) -> usize {
        (x + (y * width)) as usize
    }

    //---------------------------------------------------------------------------------------------
    // Returns the coord for a 1D index.
    //---------------------------------------------------------------------------------------------
    pub fn reverse_index_2d(index: usize, width: u32) -> (u32, u32) {
        ((index % width as usize) as u32, (index / width as usize) as u32)
    }

    //---------------------------------------------------------------------------------------------
    // Finds the origin (top left or top right) for centering content within larger bounds.
    //---------------------------------------------------------------------------------------------
    pub fn centered_origin(dimension: u32, bounding_dimension: u32) -> u32 {
        (bounding_dimension - dimension) / 2
    }
}

#[test]
fn test_2d_index() {
    assert_eq!(Misc::index_2d((5, 6), 10), 65);
    assert_eq!(Misc::reverse_index_2d(65, 10), (5, 6));
}
