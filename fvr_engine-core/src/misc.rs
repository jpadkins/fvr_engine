pub fn index_2d(x: u32, y: u32, width: u32) -> usize {
    (x + (y * width)) as usize
}

pub fn reverse_index_2d(index: usize, width: u32) -> (u32, u32) {
    (
        (index % width as usize) as u32,
        (index / width as usize) as u32,
    )
}

#[test]
fn test_2d_index() {
    assert_eq!(index_2d(5, 6, 10), 65);
    assert_eq!(reverse_index_2d(65, 10), (5, 6));
}
