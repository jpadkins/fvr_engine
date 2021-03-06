use bracket_geometry::prelude::Point;

pub trait Map2dType: Copy + Default {}
impl<T> Map2dType for T where T: Copy + Default {}

pub trait Map2dView {
    type Type: Map2dType;

    fn width(&self) -> u32;

    fn height(&self) -> u32;

    fn data(&self) -> &[Self::Type];

    fn get(&self, index: usize) -> &Self::Type;

    fn get_xy(&self, x: u32, y: u32) -> &Self::Type;

    fn get_point(&self, point: &Point) -> &Self::Type;
}

pub trait Map2dViewMut {
    type Type: Map2dType;

    fn data_mut(&mut self) -> &mut [Self::Type];

    fn get_mut(&mut self, index: usize) -> &mut Self::Type;

    fn get_xy_mut(&mut self, x: u32, y: u32) -> &mut Self::Type;

    fn get_point_mut(&mut self, point: &Point) -> &mut Self::Type;
}
