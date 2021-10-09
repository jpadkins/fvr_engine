//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::cmp::{max, min};

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Rect describes a rectangle.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    // X origin of the rect.
    pub x: i32,
    // Y origin of the rect.
    pub y: i32,
    // Width of the rect.
    pub width: i32,
    // Height of the rect.
    pub height: i32,
}

impl Rect {
    //---------------------------------------------------------------------------------------------
    // Creates a new rect.
    //---------------------------------------------------------------------------------------------
    pub fn new((x, y): ICoord, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    //---------------------------------------------------------------------------------------------
    // Create a new rect centered on a point.
    //---------------------------------------------------------------------------------------------
    pub fn with_center(center: ICoord, width: i32, height: i32) -> Self {
        Self { x: center.0 - (width / 2), y: center.1 - (height / 2), width, height }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the origin of the rect.
    //---------------------------------------------------------------------------------------------
    pub fn origin(&self) -> ICoord {
        (self.x, self.y)
    }

    //---------------------------------------------------------------------------------------------
    // Returns the max extent of the rect (opposite of origin).
    //---------------------------------------------------------------------------------------------
    pub fn max_entent(&self) -> ICoord {
        (self.x + self.width - 1, self.y + self.height - 1)
    }

    //---------------------------------------------------------------------------------------------
    // Returns the dimensions of the rect.
    //---------------------------------------------------------------------------------------------
    pub fn dimensions(&self) -> ICoord {
        (self.width, self.height)
    }

    //---------------------------------------------------------------------------------------------
    // Returns the area of the rect.
    //---------------------------------------------------------------------------------------------
    pub fn area(&self) -> i32 {
        self.width * self.height
    }

    //---------------------------------------------------------------------------------------------
    // Returns the center of the rect.
    //---------------------------------------------------------------------------------------------
    pub fn center(&self) -> ICoord {
        (self.x + (self.width / 2), self.y + (self.height / 2))
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether the rect is empty (has an area of 0).
    //---------------------------------------------------------------------------------------------
    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether the rect contains a point.
    //---------------------------------------------------------------------------------------------
    pub fn contains(&self, xy: ICoord) -> bool {
        xy.0 >= self.x
            && xy.0 < self.x + self.width
            && xy.1 >= self.y
            && xy.1 < self.y + self.height
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether the rect intersects another rect.
    //---------------------------------------------------------------------------------------------
    pub fn intersects(&self, other: &Rect) -> bool {
        other.x < self.x + self.width
            && self.x < other.x + other.width
            && other.y < self.y + self.height
            && self.y < other.y + other.height
    }

    //---------------------------------------------------------------------------------------------
    // Pushes points in the rect into a vec.
    //---------------------------------------------------------------------------------------------
    pub fn push_points(&self, points: &mut Vec<ICoord>) {
        if self.is_empty() {
            return;
        }

        for x in self.x..(self.x + self.width) {
            for y in self.y..(self.y + self.height) {
                points.push((x, y));
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns points in the rect as a vec.
    //---------------------------------------------------------------------------------------------
    pub fn points(&self) -> Vec<ICoord> {
        let mut points = Vec::new();
        self.push_points(&mut points);
        points
    }

    //---------------------------------------------------------------------------------------------
    // Pushes points on the rect's perimeter into a vec.
    //---------------------------------------------------------------------------------------------
    pub fn push_perimeter_points(&self, points: &mut Vec<ICoord>) {
        if self.is_empty() {
            return;
        }

        for x in self.x..(self.x + self.width) {
            points.push((x, self.y));
            points.push((x, self.y + self.height - 1));
        }

        for y in (self.y + 1)..(self.y + self.height - 1) {
            points.push((self.x, y));
            points.push((self.x + self.height - 1, y));
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the points on the rect's perimeter as a vec.
    //---------------------------------------------------------------------------------------------
    pub fn perimeter_points(&self) -> Vec<ICoord> {
        let mut points = Vec::new();
        self.push_perimeter_points(&mut points);
        points
    }

    //---------------------------------------------------------------------------------------------
    // Pushes the points in the rect but not in another rect into a vec.
    //---------------------------------------------------------------------------------------------
    pub fn push_difference(&self, other: &Rect, points: &mut Vec<ICoord>) {
        for xy in self.points() {
            if other.contains(xy) {
                continue;
            }

            points.push(xy);
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the points in the rect but not in another rect as a vec.
    //---------------------------------------------------------------------------------------------
    pub fn difference(&self, other: &Rect) -> Vec<ICoord> {
        let mut points = Vec::new();
        self.push_difference(other, &mut points);
        points
    }

    //---------------------------------------------------------------------------------------------
    // Pushes the points shared by the rect and another rect into a vec.
    //---------------------------------------------------------------------------------------------
    pub fn push_union(&self, other: &Rect, points: &mut Vec<ICoord>) {
        for xy in self.points() {
            if other.contains(xy) {
                points.push(xy);
            }
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the points shared by the rect and another rect as a vec.
    //---------------------------------------------------------------------------------------------
    pub fn union(&self, other: &Rect) -> Vec<ICoord> {
        let mut points = Vec::new();
        self.push_union(other, &mut points);
        points
    }

    //---------------------------------------------------------------------------------------------
    // Returns the largest rect contained in the rect and another rect.
    //---------------------------------------------------------------------------------------------
    pub fn intersection(&self, other: &Rect) -> Option<Rect> {
        if self.intersects(other) {
            let x = max(self.x, other.x);
            let y = max(self.y, other.y);
            let x_max = min(self.x + self.width, other.x + other.width);
            let y_max = min(self.y + self.height, other.y + other.height);

            return Some(Rect { x, y, width: x_max - x, height: y_max - y });
        }

        None
    }

    //---------------------------------------------------------------------------------------------
    // Returns the smallest rect that can contain both the rect and another rect.
    //---------------------------------------------------------------------------------------------
    pub fn containing(&self, other: &Rect) -> Rect {
        let x = min(self.x, other.x);
        let y = min(self.y, other.y);
        let width = max(self.x + self.width, other.x + other.width) - x;
        let height = max(self.y + self.height, other.y + other.height) - y;

        Rect { x, y, width, height }
    }

    //---------------------------------------------------------------------------------------------
    // Fits the rect into a bounding rect if possible.
    //---------------------------------------------------------------------------------------------
    pub fn fit_boundary(&mut self, other: &Rect) {
        if self.x < other.x {
            self.x = other.x;
        } else if self.x + self.width > other.x + other.width {
            self.x = (other.x + other.width) - self.width;
        }

        if self.y < other.y {
            self.y = other.y;
        } else if self.y + self.height > other.y + other.height {
            self.y = (other.y + other.height) - self.height;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Normalize a coord out of the rect.
    //---------------------------------------------------------------------------------------------
    pub fn extract_xy(&self, xy: ICoord) -> Option<ICoord> {
        if !self.contains(xy) {
            return None;
        }

        Some((xy.0 - self.x, xy.1 - self.y))
    }

    //---------------------------------------------------------------------------------------------
    // Normalize a coord into the rect.
    //---------------------------------------------------------------------------------------------
    pub fn insert_xy(&self, xy: ICoord) -> ICoord {
        (xy.0 + self.x, xy.1 + self.y)
    }
}
