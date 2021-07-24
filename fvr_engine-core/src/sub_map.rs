//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::misc::*;
use crate::rect::*;
use crate::traits::*;

//-------------------------------------------------------------------------------------------------
// SubMap provides Map2dView functionality for a subsection of another Map2dView
//-------------------------------------------------------------------------------------------------
pub struct SubMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    // Base map of the sub map.
    base_map: &'a mut M,
    // Subsection of the base map to which the sub map is mapped.
    subsection: Rect,
}

impl<'a, M> SubMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    //---------------------------------------------------------------------------------------------
    // Creates a new SubMap.
    //---------------------------------------------------------------------------------------------
    pub fn new(base_map: &'a mut M, subsection: Rect) -> Self {
        Self { base_map, subsection }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the subsection of the sub map.
    //---------------------------------------------------------------------------------------------
    pub fn subsection(&self) -> Rect {
        self.subsection
    }

    //---------------------------------------------------------------------------------------------
    // Sets the subsection of the sub map.
    //---------------------------------------------------------------------------------------------
    pub fn set_subsection(&mut self, subsection: Rect) {
        self.subsection = subsection;
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for translating subsection index to base map index.
    //---------------------------------------------------------------------------------------------
    fn translate(&self, index: usize) -> usize {
        let (x, y) = Misc::reverse_index_2d(index, self.width());
        Misc::index_2d(
            (x + self.subsection.x as u32, y + self.subsection.y as u32),
            self.subsection.width as u32,
        )
    }

    //---------------------------------------------------------------------------------------------
    // Helper function for translating subsection coord to base map coord.
    //---------------------------------------------------------------------------------------------
    fn translate_xy(&self, (x, y): UCoord) -> UCoord {
        (x + self.subsection.x as u32, y + self.subsection.y as u32)
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dView for GridMap.
//-------------------------------------------------------------------------------------------------
impl<'a, M> Map2dView for SubMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    type Type = <M as Map2dView>::Type;

    //---------------------------------------------------------------------------------------------
    // Return the width of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn width(&self) -> u32 {
        self.subsection.width as u32
    }

    //---------------------------------------------------------------------------------------------
    // Return the height of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn height(&self) -> u32 {
        self.subsection.height as u32
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get(&self, index: usize) -> &Self::Type {
        self.base_map.get(self.translate(index))
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy(&self, xy: UCoord) -> &Self::Type {
        let xy = self.translate_xy(xy);
        self.base_map.get_xy(xy)
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dViewMut for GridMap.
//-------------------------------------------------------------------------------------------------
impl<'a, M> Map2dViewMut for SubMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    type Type = <M as Map2dViewMut>::Type;

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get_mut(&mut self, index: usize) -> &mut Self::Type {
        self.base_map.get_mut(self.translate(index))
    }

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy_mut(&mut self, xy: UCoord) -> &mut Self::Type {
        let xy = self.translate_xy(xy);
        self.base_map.get_xy_mut(xy)
    }
}

//-------------------------------------------------------------------------------------------------
// Tests.
//-------------------------------------------------------------------------------------------------

#[test]
fn test_sub_map() {
    fn trait_func<M, T>(obj: &M) -> T
    where
        T: Map2dType,
        M: Map2dView<Type = T>,
    {
        *obj.get_xy((5, 5))
    }

    fn dyn_trait_func<T>(obj: &dyn Map2dView<Type = T>) -> T
    where
        T: Map2dType,
    {
        *obj.get_xy((5, 5))
    }

    let mut grid_map = crate::grid_map::GridMap::new((10, 10));
    *grid_map.get_xy_mut((1, 0)) = 10;
    *grid_map.get_xy_mut((5, 5)) = 10;

    let full_rect = Rect::new((0, 0), grid_map.width() as i32, grid_map.height() as i32);
    let sub_map = SubMap::new(&mut grid_map, full_rect);
    assert_eq!(*sub_map.get_xy((1, 0)), 10);
    assert_eq!(*sub_map.get_xy((5, 5)), 10);
    assert_eq!(trait_func(&sub_map), 10);
    assert_eq!(dyn_trait_func(&sub_map), 10);

    let sub_map = SubMap::new(&mut grid_map, Rect::new((1, 0), 10, 10));
    assert_eq!(*sub_map.get(0), 10);
    assert_eq!(*sub_map.get_xy((4, 5)), 10);
}
