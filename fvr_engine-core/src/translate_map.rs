use bracket_geometry::prelude::{Point, Rect};

use crate::misc::*;
use crate::traits::*;

pub enum TranslateMapTranslation {
    SubSection(Rect),
    Lambda(Box<dyn Fn(u32, u32) -> (u32, u32)>),
}

// Provides GridMap functionality for a subsection of a GridMap.
pub struct TranslateMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    base_map: &'a mut M,
    translation: Option<TranslateMapTranslation>,
}

impl<'a, M> TranslateMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    pub fn new(base_map: &'a mut M, translation: Option<TranslateMapTranslation>) -> Self {
        Self { base_map, translation }
    }

    pub fn translation(&self) -> &Option<TranslateMapTranslation> {
        &self.translation
    }

    pub fn translation_mut(&mut self) -> &Option<TranslateMapTranslation> {
        &mut self.translation
    }

    fn translate(&self, index: usize) -> usize {
        if let Some(translation) = &self.translation {
            match translation {
                TranslateMapTranslation::SubSection(rect) => {
                    let (x, y) = reverse_index_2d(index, self.width());
                    index_2d(x + rect.x1 as u32, y + rect.y1 as u32, rect.width() as u32)
                }
                TranslateMapTranslation::Lambda(lambda) => {
                    let (x, y) = reverse_index_2d(index, self.width());
                    let (x, y) = lambda(x, y);
                    index_2d(x, y, self.width())
                }
            }
        } else {
            index
        }
    }

    fn translate_xy(&self, x: u32, y: u32) -> (u32, u32) {
        if let Some(translation) = &self.translation {
            match translation {
                TranslateMapTranslation::SubSection(rect) => {
                    (x + rect.x1 as u32, y + rect.y1 as u32)
                }
                TranslateMapTranslation::Lambda(lambda) => lambda(x, y),
            }
        } else {
            (x, y)
        }
    }

    fn translate_point(&self, point: &Point) -> Point {
        if let Some(translation) = &self.translation {
            match translation {
                TranslateMapTranslation::SubSection(rect) => {
                    Point::new(point.x + rect.x1, point.y + rect.y1)
                }
                TranslateMapTranslation::Lambda(lambda) => {
                    let (x, y) = lambda(point.x as u32, point.y as u32);
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
        self.base_map.data()
    }

    fn get(&self, index: usize) -> &Self::Type {
        self.base_map.get(self.translate(index))
    }

    fn get_xy(&self, x: u32, y: u32) -> &Self::Type {
        let (x, y) = self.translate_xy(x, y);
        self.base_map.get_xy(x, y)
    }

    fn get_point(&self, point: &Point) -> &Self::Type {
        self.base_map.get_point(&self.translate_point(point))
    }
}

impl<'a, M> Map2dViewMut for TranslateMap<'a, M>
where
    M: Map2dView + Map2dViewMut,
{
    type Type = <M as Map2dViewMut>::Type;

    fn data_mut(&mut self) -> &mut [Self::Type] {
        self.base_map.data_mut()
    }

    fn get_mut(&mut self, index: usize) -> &mut Self::Type {
        self.base_map.get_mut(self.translate(index))
    }

    fn get_xy_mut(&mut self, x: u32, y: u32) -> &mut Self::Type {
        let (x, y) = self.translate_xy(x, y);
        self.base_map.get_xy_mut(x, y)
    }

    fn get_point_mut(&mut self, point: &Point) -> &mut Self::Type {
        self.base_map.get_point_mut(&self.translate_point(point))
    }
}

#[test]
fn test_view_map() {
    fn test_trait_obj_function<T>(obj: &dyn Map2dView<Type = T>) -> T
    where
        T: Map2dType,
    {
        *obj.get_xy(4, 5)
    }

    let mut grid_map = crate::grid_map::GridMap::new(10, 10);
    *grid_map.get_xy_mut(1, 0) = 10;
    *grid_map.get_xy_mut(5, 5) = 10;

    let view_map = TranslateMap::new(&mut grid_map, None);
    assert_eq!(*view_map.get_xy(1, 0), 10);
    assert_eq!(*view_map.get_xy(5, 5), 10);

    let translation = TranslateMapTranslation::SubSection(Rect::with_size(1, 0, 10, 10));
    let view_map = TranslateMap::new(&mut grid_map, Some(translation));
    assert_eq!(*view_map.get(0), 10);
    assert_eq!(*view_map.get_xy(4, 5), 10);
    assert_eq!(*view_map.get_point(&Point::new(4, 5)), 10);

    let translation = TranslateMapTranslation::Lambda(Box::new(|x, y| (x + 1, y)));
    let view_map = TranslateMap::new(&mut grid_map, Some(translation));
    assert_eq!(*view_map.get(0), 10);
    assert_eq!(*view_map.get_xy(4, 5), 10);
    assert_eq!(*view_map.get_point(&Point::new(4, 5)), 10);
    assert_eq!(test_trait_obj_function(&view_map), 10);
}
