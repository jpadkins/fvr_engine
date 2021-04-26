pub struct Misc {}

impl Misc {
    pub fn index_2d((x, y): (u32, u32), width: u32) -> usize {
        (x + (y * width)) as usize
    }

    pub fn reverse_index_2d(index: usize, width: u32) -> (u32, u32) {
        ((index % width as usize) as u32, (index / width as usize) as u32)
    }

    pub fn centered_origin(dimension: u32, bounding_dimension: u32) -> u32 {
        (bounding_dimension - dimension) / 2
    }
}

#[test]
fn test_2d_index() {
    assert_eq!(Misc::index_2d((5, 6), 10), 65);
    assert_eq!(Misc::reverse_index_2d(65, 10), (5, 6));
}
