//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use once_cell::sync::Lazy;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::direction::*;
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------
pub static CARDINAL_ADJACENCIES: Lazy<Vec<Direction>> =
    Lazy::new(|| vec![NORTH_DIRECTION, EAST_DIRECTION, SOUTH_DIRECTION, WEST_DIRECTION]);
pub static DIAGONAL_ADJACENCIES: Lazy<Vec<Direction>> = Lazy::new(|| {
    vec![NORTHEAST_DIRECTION, SOUTHEAST_DIRECTION, SOUTHWEST_DIRECTION, NORTHWEST_DIRECTION]
});
pub static EIGHT_WAY_ADJACENCIES: Lazy<Vec<Direction>> = Lazy::new(|| {
    vec![
        NORTH_DIRECTION,
        NORTHEAST_DIRECTION,
        EAST_DIRECTION,
        SOUTHEAST_DIRECTION,
        SOUTH_DIRECTION,
        SOUTHWEST_DIRECTION,
        WEST_DIRECTION,
        NORTHWEST_DIRECTION,
    ]
});

//-------------------------------------------------------------------------------------------------
// Enumerates the types of adjacencies.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Adjacency {
    // The cardinal adjacencies.
    Cardinals,
    // The diagonal adjacencies.
    Diagonals,
    // Both the cardinal and diagonal adjacencies.
    EightWay,
}

impl Adjacency {
    //---------------------------------------------------------------------------------------------
    // Returns an iterator over the directions for an adjacency.
    //---------------------------------------------------------------------------------------------
    pub fn iter(&self) -> impl Iterator<Item = &Direction> {
        match self {
            Self::Cardinals => CARDINAL_ADJACENCIES.iter(),
            Self::Diagonals => DIAGONAL_ADJACENCIES.iter(),
            Self::EightWay => EIGHT_WAY_ADJACENCIES.iter(),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns a reverse iterator over the directions for an adjacency.
    //---------------------------------------------------------------------------------------------
    pub fn iter_rev(&self) -> impl Iterator<Item = &Direction> {
        match self {
            Self::Cardinals => CARDINAL_ADJACENCIES.iter().rev(),
            Self::Diagonals => DIAGONAL_ADJACENCIES.iter().rev(),
            Self::EightWay => EIGHT_WAY_ADJACENCIES.iter().rev(),
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns an iterator over the directions for an adjacency from a starting direction.
    //---------------------------------------------------------------------------------------------
    pub fn iter_from(&self, start: Direction) -> impl Iterator<Item = Direction> {
        let indices = match self {
            Self::Cardinals => {
                // Default to North if the direction is null.
                let dir = if start == NULL_DIRECTION { NORTH_DIRECTION } else { start };

                // Ensure start is one of the cardinal directions.
                let mut index = dir.orientation() as usize;

                if index % 2 == 1 {
                    index += 1;
                }

                // Create and return the iterator.
                (index..=(index + 6)).step_by(2)
            }
            Self::Diagonals => {
                // Default to North if the direction is null.
                let dir = if start == NULL_DIRECTION { NORTHEAST_DIRECTION } else { start };

                // Ensure start is one of the cardinal directions.
                let mut index = dir.orientation() as usize;

                if index % 2 == 0 {
                    index += 1;
                }

                // Create and return the iterator.
                (index..=(index + 6)).step_by(2)
            }
            Self::EightWay => {
                // Default to North if the direction is null.
                let dir = if start == NULL_DIRECTION { NORTHEAST_DIRECTION } else { start };

                let index = dir.orientation() as usize;

                // Create and return the iterator.
                (index..=(index + 7)).step_by(1)
            }
        };

        indices.map(Direction::from_index)
    }

    //---------------------------------------------------------------------------------------------
    // Returns a reverse iterator over the directions for an adjacency from a starting direction.
    //---------------------------------------------------------------------------------------------
    pub fn iter_from_rev(&self, start: Direction) -> impl Iterator<Item = Direction> {
        let indices = match self {
            Self::Cardinals => {
                // Default to North if the direction is null.
                let dir = if start == NULL_DIRECTION { NORTH_DIRECTION } else { start };

                // Ensure start is one of the cardinal directions.
                let mut index = dir.orientation() as i32;

                if index % 2 == 1 {
                    index -= 1;
                }

                // Create and return the iterator.
                num::range_step_inclusive(index, index - 6, -2)
            }
            Self::Diagonals => {
                // Default to North if the direction is null.
                let dir = if start == NULL_DIRECTION { NORTHEAST_DIRECTION } else { start };

                // Ensure start is one of the cardinal directions.
                let mut index = dir.orientation() as i32;

                if index % 2 == 0 {
                    index -= 1;
                }

                // Create and return the iterator.
                num::range_step_inclusive(index, index - 6, -2)
            }
            Self::EightWay => {
                // Default to North if the direction is null.
                let dir = if start == NULL_DIRECTION { NORTHEAST_DIRECTION } else { start };

                let index = dir.orientation() as i32;

                // Create and return the iterator.
                num::range_step_inclusive(index, index - 7, -1)
            }
        };

        indices.map(|i| Direction::from_index(i as usize))
    }

    //---------------------------------------------------------------------------------------------
    // Returns an iterator over the neighboring coords around a coord for a given adjacency.
    //---------------------------------------------------------------------------------------------
    pub fn neighbors(&self, (x, y): ICoord) -> impl Iterator<Item = ICoord> {
        let adjacencies = match self {
            Self::Cardinals => &CARDINAL_ADJACENCIES,
            Self::Diagonals => &DIAGONAL_ADJACENCIES,
            Self::EightWay => &EIGHT_WAY_ADJACENCIES,
        };

        adjacencies.iter().map(move |dir| (x + dir.dx(), y + dir.dy()))
    }

    //---------------------------------------------------------------------------------------------
    // Returns a reverse iterator over the neighboring coords around a coord for a given adjacency.
    //---------------------------------------------------------------------------------------------
    pub fn neighbors_rev(&self, (x, y): ICoord) -> impl Iterator<Item = ICoord> {
        let adjacencies = match self {
            Self::Cardinals => &CARDINAL_ADJACENCIES,
            Self::Diagonals => &DIAGONAL_ADJACENCIES,
            Self::EightWay => &EIGHT_WAY_ADJACENCIES,
        };

        adjacencies.iter().rev().map(move |dir| (x + dir.dx(), y + dir.dy()))
    }

    //---------------------------------------------------------------------------------------------
    // Returns iterator over neighbors around a coord for adjacency with start direction.
    //---------------------------------------------------------------------------------------------
    pub fn neighbors_from(
        &self,
        (x, y): ICoord,
        start: Direction,
    ) -> impl Iterator<Item = ICoord> {
        self.iter_from(start).map(move |dir| (x + dir.dx(), y + dir.dy()))
    }

    //---------------------------------------------------------------------------------------------
    // Returns reverse iterator over neighbors around a coord for adjacency with start direction.
    //---------------------------------------------------------------------------------------------
    pub fn neighbors_from_rev(
        &self,
        (x, y): ICoord,
        start: Direction,
    ) -> impl Iterator<Item = ICoord> {
        self.iter_from_rev(start).map(move |dir| (x + dir.dx(), y + dir.dy()))
    }

    //---------------------------------------------------------------------------------------------
    // Returns whether a coord is in one of the neighboring eight-way coords of a target.
    // TODO: Make method that takes into account agacency type.
    //---------------------------------------------------------------------------------------------
    pub fn is_neighbor(target: ICoord, neighbor: ICoord) -> bool {
        (target.0 - neighbor.0).abs() <= 1 && (target.1 - neighbor.1).abs() <= 1
    }
}

//-------------------------------------------------------------------------------------------------
// Tests.
//-------------------------------------------------------------------------------------------------

// iter_from() tests.
#[test]
fn test_adjacency_iter_from_cardinals() {
    let start = NORTHEAST_DIRECTION;
    let directions: Vec<Direction> = Adjacency::Cardinals.iter_from(start).collect();
    let expected = vec![EAST_DIRECTION, SOUTH_DIRECTION, WEST_DIRECTION, NORTH_DIRECTION];
    assert_eq!(directions, expected);
}

#[test]
fn test_adjacency_iter_from_diagonals() {
    let start = EAST_DIRECTION;
    let directions: Vec<Direction> = Adjacency::Diagonals.iter_from(start).collect();
    let expected =
        vec![SOUTHEAST_DIRECTION, SOUTHWEST_DIRECTION, NORTHWEST_DIRECTION, NORTHEAST_DIRECTION];
    assert_eq!(directions, expected);
}

#[test]
fn test_adjacency_iter_from_eight_way() {
    let start = SOUTHWEST_DIRECTION;
    let directions: Vec<Direction> = Adjacency::EightWay.iter_from(start).collect();
    let expected = vec![
        SOUTHWEST_DIRECTION,
        WEST_DIRECTION,
        NORTHWEST_DIRECTION,
        NORTH_DIRECTION,
        NORTHEAST_DIRECTION,
        EAST_DIRECTION,
        SOUTHEAST_DIRECTION,
        SOUTH_DIRECTION,
    ];
    assert_eq!(directions, expected);
}

// iter_from_rev() tests.
#[test]
fn test_adjacency_iter_from_rev_cardinals() {
    let start = NORTHEAST_DIRECTION;
    let directions: Vec<Direction> = Adjacency::Cardinals.iter_from_rev(start).collect();
    let mut expected = vec![EAST_DIRECTION, SOUTH_DIRECTION, WEST_DIRECTION, NORTH_DIRECTION];
    expected.reverse();
    assert_eq!(directions, expected);
}

#[test]
fn test_adjacency_iter_from_rev_diagonals() {
    let start = EAST_DIRECTION;
    let directions: Vec<Direction> = Adjacency::Diagonals.iter_from_rev(start).collect();
    let mut expected =
        vec![SOUTHEAST_DIRECTION, SOUTHWEST_DIRECTION, NORTHWEST_DIRECTION, NORTHEAST_DIRECTION];
    expected.reverse();
    assert_eq!(directions, expected);
}

#[test]
fn test_adjacency_iter_from_rev_eight_way() {
    let start = SOUTHWEST_DIRECTION;
    let directions: Vec<Direction> = Adjacency::EightWay.iter_from_rev(start).collect();
    let expected = vec![
        SOUTHWEST_DIRECTION,
        SOUTH_DIRECTION,
        SOUTHEAST_DIRECTION,
        EAST_DIRECTION,
        NORTHEAST_DIRECTION,
        NORTH_DIRECTION,
        NORTHWEST_DIRECTION,
        WEST_DIRECTION,
    ];
    assert_eq!(directions, expected);
}

// neighbors() tests.
#[test]
fn test_adjacency_neighbors_cardinals() {
    let xy = (1, 1);
    let neighbors: Vec<ICoord> = Adjacency::Cardinals.neighbors(xy).collect();
    let expected = vec![(1, 0), (2, 1), (1, 2), (0, 1)];
    assert_eq!(neighbors, expected);
}

#[test]
fn test_adjacency_neighbors_diagonals() {
    let xy = (1, 1);
    let neighbors: Vec<ICoord> = Adjacency::Diagonals.neighbors(xy).collect();
    let expected = vec![(2, 0), (2, 2), (0, 2), (0, 0)];
    assert_eq!(neighbors, expected);
}

#[test]
fn test_adjacency_neighbors_eight_way() {
    let xy = (1, 1);
    let neighbors: Vec<ICoord> = Adjacency::EightWay.neighbors(xy).collect();
    let expected = vec![(1, 0), (2, 0), (2, 1), (2, 2), (1, 2), (0, 2), (0, 1), (0, 0)];
    assert_eq!(neighbors, expected);
}

// neighbors_rev() tests.
#[test]
fn test_adjacency_neighbors_rev_cardinals() {
    let xy = (1, 1);
    let neighbors: Vec<ICoord> = Adjacency::Cardinals.neighbors_rev(xy).collect();
    let mut expected = vec![(1, 0), (2, 1), (1, 2), (0, 1)];
    expected.reverse();
    assert_eq!(neighbors, expected);
}

#[test]
fn test_adjacency_neighbors_rev_diagonals() {
    let xy = (1, 1);
    let neighbors: Vec<ICoord> = Adjacency::Diagonals.neighbors_rev(xy).collect();
    let mut expected = vec![(2, 0), (2, 2), (0, 2), (0, 0)];
    expected.reverse();
    assert_eq!(neighbors, expected);
}

#[test]
fn test_adjacency_neighbors_rev_eight_way() {
    let xy = (1, 1);
    let neighbors: Vec<ICoord> = Adjacency::EightWay.neighbors_rev(xy).collect();
    let mut expected = vec![(1, 0), (2, 0), (2, 1), (2, 2), (1, 2), (0, 2), (0, 1), (0, 0)];
    expected.reverse();
    assert_eq!(neighbors, expected);
}

#[test]
fn test_is_neighbor() {
    let target = (3, 3);
    assert_eq!(Adjacency::is_neighbor(target, (3, 2)), true);
    assert_eq!(Adjacency::is_neighbor(target, (4, 2)), true);
    assert_eq!(Adjacency::is_neighbor(target, (4, 3)), true);
    assert_eq!(Adjacency::is_neighbor(target, (4, 4)), true);
    assert_eq!(Adjacency::is_neighbor(target, (3, 4)), true);
    assert_eq!(Adjacency::is_neighbor(target, (2, 4)), true);
    assert_eq!(Adjacency::is_neighbor(target, (2, 3)), true);
    assert_eq!(Adjacency::is_neighbor(target, (2, 2)), true);
    assert_eq!(Adjacency::is_neighbor(target, (3, 1)), false);
    assert_eq!(Adjacency::is_neighbor(target, (5, 3)), false);
    assert_eq!(Adjacency::is_neighbor(target, (3, 5)), false);
    assert_eq!(Adjacency::is_neighbor(target, (1, 3)), false);
}

// TODO: neighbors_from() tests.
// TODO: neighbors_from_rev() tests.
