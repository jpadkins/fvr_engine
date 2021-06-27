use std::slice::{Iter, IterMut};

use bracket_geometry::prelude::{Point, Rect};

use crate::misc::*;
use crate::traits::*;

pub enum MapTranslation {
    SubSection(Rect),
    Lambda(Box<dyn Fn((u32, u32)) -> (u32, u32)>),
}

// Provides GridMap functionality for a subsection of a GridMap.
pub struct TranslateMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    base_map: &'a mut M,
    translation: Option<MapTranslation>,
}

impl<'a, M> TranslateMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    pub fn new(base_map: &'a mut M, translation: Option<MapTranslation>) -> Self {
        Self { base_map, translation }
    }

    pub fn translation(&self) -> &Option<MapTranslation> {
        &self.translation
    }

    pub fn translation_mut(&mut self) -> &Option<MapTranslation> {
        &mut self.translation
    }

    fn translate(&self, index: usize) -> usize {
        if let Some(translation) = &self.translation {
            match translation {
                MapTranslation::SubSection(rect) => {
                    let (x, y) = Misc::reverse_index_2d(index, self.width());
                    Misc::index_2d((x + rect.x1 as u32, y + rect.y1 as u32), rect.width() as u32)
                }
                MapTranslation::Lambda(lambda) => {
                    let xy = Misc::reverse_index_2d(index, self.width());
                    let xy = lambda(xy);
                    Misc::index_2d(xy, self.width())
                }
            }
        } else {
            index
        }
    }

    fn translate_xy(&self, (x, y): (u32, u32)) -> (u32, u32) {
        if let Some(translation) = &self.translation {
            match translation {
                MapTranslation::SubSection(rect) => (x + rect.x1 as u32, y + rect.y1 as u32),
                MapTranslation::Lambda(lambda) => lambda((x, y)),
            }
        } else {
            (x, y)
        }
    }

    fn translate_point(&self, point: &Point) -> Point {
        if let Some(translation) = &self.translation {
            match translation {
                MapTranslation::SubSection(rect) => {
                    Point::new(point.x + rect.x1, point.y + rect.y1)
                }
                MapTranslation::Lambda(lambda) => {
                    let (x, y) = lambda((point.x as u32, point.y as u32));
                    Point::new(x as i32, y as i32)
                }
            }
        } else {
            *point
        }
    }
}

impl<'a, M> Map2dView for TranslateMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    type Type = <M as Map2dView>::Type;

    fn width(&self) -> u32 {
        self.base_map.width()
    }

    fn height(&self) -> u32 {
        self.base_map.height()
    }

    fn data(&self) -> &[Self::Type] {
        // This method should never be called for translate map.
        debug_assert!(false);

        self.base_map.data()
    }

    fn get(&self, index: usize) -> &Self::Type {
        self.base_map.get(self.translate(index))
    }

    fn get_xy(&self, xy: (u32, u32)) -> &Self::Type {
        let xy = self.translate_xy(xy);
        self.base_map.get_xy(xy)
    }

    fn get_point(&self, point: &Point) -> &Self::Type {
        self.base_map.get_point(&self.translate_point(point))
    }

    fn iter(&self) -> Iter<'_, Self::Type> {
        self.base_map.data().iter()
    }
}

impl<'a, M> Map2dViewMut for TranslateMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    type Type = <M as Map2dViewMut>::Type;

    fn data_mut(&mut self) -> &mut [Self::Type] {
        // This method should never be called for translate map.
        debug_assert!(false);

        self.base_map.data_mut()
    }

    fn get_mut(&mut self, index: usize) -> &mut Self::Type {
        self.base_map.get_mut(self.translate(index))
    }

    fn get_xy_mut(&mut self, xy: (u32, u32)) -> &mut Self::Type {
        let xy = self.translate_xy(xy);
        self.base_map.get_xy_mut(xy)
    }

    fn get_point_mut(&mut self, point: &Point) -> &mut Self::Type {
        self.base_map.get_point_mut(&self.translate_point(point))
    }

    fn iter_mut(&mut self) -> IterMut<'_, Self::Type> {
        self.base_map.data_mut().iter_mut()
    }
}

#[test]
fn test_translate_map() {
    fn trait_func<M, T>(obj: &M) -> T
    where
        T: Map2dType,
        M: Map2dView<Type = T>,
    {
        *obj.get_xy((4, 5))
    }

    fn dyn_trait_func<T>(obj: &dyn Map2dView<Type = T>) -> T
    where
        T: Map2dType,
    {
        *obj.get_xy((4, 5))
    }

    let mut grid_map = crate::grid_map::GridMap::new(10, 10);
    *grid_map.get_xy_mut((1, 0)) = 10;
    *grid_map.get_xy_mut((5, 5)) = 10;

    let translate_map = TranslateMap::new(&mut grid_map, None);
    assert_eq!(*translate_map.get_xy((1, 0)), 10);
    assert_eq!(*translate_map.get_xy((5, 5)), 10);

    let translation = MapTranslation::SubSection(Rect::with_size(1, 0, 10, 10));
    let translate_map = TranslateMap::new(&mut grid_map, Some(translation));
    assert_eq!(*translate_map.get(0), 10);
    assert_eq!(*translate_map.get_xy((4, 5)), 10);
    assert_eq!(*translate_map.get_point(&Point::new(4, 5)), 10);

    let translation = MapTranslation::Lambda(Box::new(|(x, y)| (x + 1, y)));
    let translate_map = TranslateMap::new(&mut grid_map, Some(translation));
    assert_eq!(*translate_map.get(0), 10);
    assert_eq!(*translate_map.get_xy((4, 5)), 10);
    assert_eq!(*translate_map.get_point(&Point::new(4, 5)), 10);
    assert_eq!(trait_func(&translate_map), 10);
    assert_eq!(dyn_trait_func(&translate_map), 10);
}
