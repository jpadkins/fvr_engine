//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Map2dType constraints the types which a Map2dView/Mut may contain.
//-------------------------------------------------------------------------------------------------
pub trait Map2dType: Clone + Default {}
impl<T> Map2dType for T where T: Clone + Default {}

//-------------------------------------------------------------------------------------------------
// Describes an immutable access API for a 2d grid.
//-------------------------------------------------------------------------------------------------
pub trait Map2dView {
    type Type: Map2dType;

    //---------------------------------------------------------------------------------------------
    // Return the width of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn width(&self) -> u32;

    //---------------------------------------------------------------------------------------------
    // Return the height of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn height(&self) -> u32;

    //---------------------------------------------------------------------------------------------
    // Return the dimensions of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn dimensions(&self) -> UCoord;

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get(&self, index: usize) -> &Self::Type;

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at an xy coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy(&self, xy: UCoord) -> &Self::Type;

    //---------------------------------------------------------------------------------------------
    // Returns whether a coord is in bounds of the Map2d.
    //---------------------------------------------------------------------------------------------
    fn in_bounds(&self, xy: UCoord) -> bool {
        xy.0 < self.width() && xy.1 < self.height()
    }
}

//-------------------------------------------------------------------------------------------------
// Describes a mutable access API for a 2d grid.
//-------------------------------------------------------------------------------------------------
pub trait Map2dViewMut {
    type Type: Map2dType;

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get_mut(&mut self, index: usize) -> &mut Self::Type;

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at an xy coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy_mut(&mut self, xy: UCoord) -> &mut Self::Type;
}

//-------------------------------------------------------------------------------------------------
// Map2d implements both an immutable and mutable access API for a 2d grid.
//-------------------------------------------------------------------------------------------------
pub trait Map2d<T>: Map2dView<Type = T> + Map2dViewMut<Type = T>
where
    T: Map2dType,
{
}

//-------------------------------------------------------------------------------------------------
// Implement Map2d for all Map2dType.
//-------------------------------------------------------------------------------------------------
impl<M, T> Map2d<T> for M
where
    M: Map2dView<Type = T> + Map2dViewMut<Type = T>,
    T: Map2dType,
{
}

//-------------------------------------------------------------------------------------------------
// Helper macro for immutably iterating a Map2d.
//-------------------------------------------------------------------------------------------------
#[macro_export]
macro_rules! map2d_iter {
    ($map2d:expr, $item:ident, $work:expr) => {
        for x in 0..$map2d.width() {
            for y in 0..$map2d.height() {
                let $item = $map2d.get_xy((x, y));
                $work
            }
        }
    };
}

//-------------------------------------------------------------------------------------------------
// Helper macro for immutably iterating a Map2d with indices.
//-------------------------------------------------------------------------------------------------
#[macro_export]
macro_rules! map2d_iter_index {
    ($map2d:expr, $x:ident, $y:ident, $item:ident, $work:expr) => {
        for $x in 0..$map2d.width() {
            for $y in 0..$map2d.height() {
                let $item = $map2d.get_xy(($x, $y));
                $work
            }
        }
    };
}

//-------------------------------------------------------------------------------------------------
// Helper macro for mutably iterating a Map2d.
//-------------------------------------------------------------------------------------------------
#[macro_export]
macro_rules! map2d_iter_mut {
    ($map2d:expr, $item:ident, $work:expr) => {
        for x in 0..$map2d.width() {
            for y in 0..$map2d.height() {
                let $item = $map2d.get_xy_mut((x, y));
                $work
            }
        }
    };
}
