use bracket_geometry::prelude::Point;

use crate::misc::*;
use crate::traits::*;

pub struct GridMap<T>
where
    T: Map2dType,
{
    width: u32,
    height: u32,
    data: Vec<T>,
}

// Wraps and provides access to a 2D grid of T.
#[allow(dead_code)]
impl<T> GridMap<T>
where
    T: Map2dType,
{
    pub fn new(width: u32, height: u32) -> Self {
        let data = vec![Default::default(); (width * height) as usize];
        Self {
            width,
            height,
            data,
        }
    }
}

impl<T> Map2dView for GridMap<T>
where
    T: Map2dType,
{
    type Type = T;

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn data(&self) -> &[Self::Type] {
        &self.data
    }

    fn get(&self, index: usize) -> &Self::Type {
        &self.data[index]
    }

    fn get_xy(&self, x: u32, y: u32) -> &Self::Type {
        let index = index_2d(x, y, self.width);
        &self.data[index]
    }

    fn get_point(&self, point: &Point) -> &Self::Type {
        let index = index_2d(point.x as u32, point.y as u32, self.width);
        &self.data[index]
    }
}

impl<T> Map2dViewMut for GridMap<T>
where
    T: Map2dType,
{
    type Type = T;

    fn data_mut(&mut self) -> &mut [Self::Type] {
        &mut self.data
    }

    fn get_mut(&mut self, index: usize) -> &mut Self::Type {
        &mut self.data[index]
    }

    fn get_xy_mut(&mut self, x: u32, y: u32) -> &mut Self::Type {
        let index = index_2d(x, y, self.width);
        &mut self.data[index]
    }

    fn get_point_mut(&mut self, point: &Point) -> &mut Self::Type {
        let index = index_2d(point.x as u32, point.y as u32, self.width);
        &mut self.data[index]
    }
}

#[test]
fn test_grid_map() {
    let width = 4;
    let height = 6;
    let x = width / 2;
    let y = height / 2;
    let mut grid_map = GridMap::new(width, height);

    // Test get*().
    assert_eq!(*grid_map.get(index_2d(x, y, width)), u32::default());
    assert_eq!(*grid_map.get_xy(x, y), u32::default());
    assert_eq!(
        *grid_map.get_point(&Point {
            x: x as i32,
            y: y as i32
        }),
        u32::default()
    );

    // Test get_*_mut();
    *grid_map.get_mut(index_2d(x, y, width)) = 1;
    assert_eq!(*grid_map.get(index_2d(x, y, width)), 1);

    *grid_map.get_xy_mut(x, y) = 2;
    assert_eq!(*grid_map.get_xy(x, y), 2);

    *grid_map.get_point_mut(&Point {
        x: x as i32,
        y: y as i32,
    }) = 3;
    assert_eq!(
        *grid_map.get_point(&Point {
            x: x as i32,
            y: y as i32
        }),
        3
    );

    // Test iter().
    let mut expected = vec![Default::default(); 4 * 6];
    expected[index_2d(x, y, grid_map.width())] = 3;
    assert_eq!(
        grid_map.data().iter().map(|&v| v).collect::<Vec<u32>>(),
        expected
    );

    // Test iter_mut().
    expected[1] = 1;
    for (i, v) in grid_map.data_mut().iter_mut().enumerate() {
        if i == 1 {
            *v = 1;
        }
    }
    assert_eq!(
        grid_map.data().iter().map(|&v| v).collect::<Vec<u32>>(),
        expected
    );
}
