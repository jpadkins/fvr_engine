//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::direction::*;

//-------------------------------------------------------------------------------------------------
// Statics.
//-------------------------------------------------------------------------------------------------
pub static CARDINAL_ADJACENCIES: [Direction; 4] =
    [NORTH_DIRECTION, EAST_DIRECTION, SOUTH_DIRECTION, WEST_DIRECTION];
pub static DIAGONAL_ADJACENCIES: [Direction; 4] =
    [NORTHEAST_DIRECTION, SOUTHEAST_DIRECTION, SOUTHWEST_DIRECTION, NORTHWEST_DIRECTION];
pub static EIGHT_WAY_ADJACENCIES: [Direction; 8] = [
    NORTH_DIRECTION,
    NORTHEAST_DIRECTION,
    EAST_DIRECTION,
    SOUTHEAST_DIRECTION,
    SOUTH_DIRECTION,
    SOUTHWEST_DIRECTION,
    WEST_DIRECTION,
    NORTHWEST_DIRECTION,
];

//-------------------------------------------------------------------------------------------------
// Enumerates the types of adjacencies.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
                (index..=index + 6).step_by(2)
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
                (index..=index + 6).step_by(2)
            }
            Self::EightWay => {
                // Default to North if the direction is null.
                let dir = if start == NULL_DIRECTION { NORTHEAST_DIRECTION } else { start };

                let index = dir.orientation() as usize;

                // Create and return the iterator.
                (index..=index + 7).step_by(1)
            }
        };

        indices.map(|i| Direction::from_index(i))
    }
}

#[test]
fn test_adjacency_iter_from_cardinal() {
    let start = NORTHEAST_DIRECTION;
    let directions: Vec<Direction> = Adjacency::Cardinals.iter_from(start).collect();
    let expected = vec![EAST_DIRECTION, SOUTH_DIRECTION, WEST_DIRECTION, NORTH_DIRECTION];
    assert_eq!(directions, expected);
}

#[test]
fn test_adjacency_iter_from_diagonal() {
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
