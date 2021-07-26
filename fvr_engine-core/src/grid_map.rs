//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::map2d::*;
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// GridMap describes a 2D grid represented internally by a 1D array.
//-------------------------------------------------------------------------------------------------
pub struct GridMap<T>
where
    T: Map2dType,
{
    // Dimensions of the grid map.
    dimensions: UCoord,
    // Underlying data of the grid map.
    data: Vec<T>,
}

impl<T> GridMap<T>
where
    T: Map2dType,
{
    //---------------------------------------------------------------------------------------------
    // Creates a new GridMap.
    //---------------------------------------------------------------------------------------------
    pub fn new(dimensions: UCoord) -> Self {
        let data = vec![Default::default(); (dimensions.0 * dimensions.1) as usize];
        Self { dimensions, data }
    }

    //---------------------------------------------------------------------------------------------
    // Resizes the grid map to new dimensions.
    //---------------------------------------------------------------------------------------------
    pub fn resize(&mut self, dimensions: UCoord) {
        self.dimensions = dimensions;
        self.data.resize((dimensions.0 * dimensions.1) as usize, Default::default());
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dView for GridMap.
//-------------------------------------------------------------------------------------------------
impl<T> Map2dView for GridMap<T>
where
    T: Map2dType,
{
    type Type = T;

    //---------------------------------------------------------------------------------------------
    // Return the width of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn width(&self) -> u32 {
        self.dimensions.0
    }

    //---------------------------------------------------------------------------------------------
    // Return the height of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn height(&self) -> u32 {
        self.dimensions.1
    }

    //---------------------------------------------------------------------------------------------
    // Return the dimensions of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn dimensions(&self) -> UCoord {
        self.dimensions
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get(&self, index: usize) -> &Self::Type {
        &self.data[index]
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy(&self, xy: UCoord) -> &Self::Type {
        let index = Misc::index_2d(xy, self.width());
        &self.data[index]
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dViewMut for GridMap.
//-------------------------------------------------------------------------------------------------
impl<T> Map2dViewMut for GridMap<T>
where
    T: Map2dType,
{
    type Type = T;

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get_mut(&mut self, index: usize) -> &mut Self::Type {
        &mut self.data[index]
    }

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy_mut(&mut self, xy: UCoord) -> &mut Self::Type {
        let index = Misc::index_2d(xy, self.width());
        &mut self.data[index]
    }
}

//-------------------------------------------------------------------------------------------------
// Tests.
//-------------------------------------------------------------------------------------------------

#[test]
fn test_grid_map() {
    let width = 4;
    let height = 6;
    let x = width / 2;
    let y = height / 2;
    let mut grid_map = GridMap::new((width, height));

    // Test get*().
    assert_eq!(*grid_map.get(Misc::index_2d((x, y), width)), u32::default());
    assert_eq!(*grid_map.get_xy((x, y)), u32::default());

    // Test get_*_mut();
    *grid_map.get_mut(Misc::index_2d((x, y), width)) = 1;
    assert_eq!(*grid_map.get(Misc::index_2d((x, y), width)), 1);

    *grid_map.get_xy_mut((x, y)) = 2;
    assert_eq!(*grid_map.get_xy((x, y)), 2);
}
