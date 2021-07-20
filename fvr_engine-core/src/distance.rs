//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::adjacency::*;
use crate::radius::*;

//-------------------------------------------------------------------------------------------------
// Enumerates the distance calculation methods.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Distance {
    // Chessboard distance.
    Chebyshev,
    // Straight line distance.
    Euclidean,
    // Taxicab distance.
    Manhattan,
}

impl Distance {
    //---------------------------------------------------------------------------------------------
    // Returns the adjacency for a distance.
    //---------------------------------------------------------------------------------------------
    pub fn adjacency(&self) -> Adjacency {
        match self {
            Distance::Chebyshev | Distance::Euclidean => Adjacency::EightWay,
            Self::Manhattan => Adjacency::Cardinals,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the radius for a distance.
    //---------------------------------------------------------------------------------------------
    pub fn radius(&self) -> Radius {
        match self {
            Self::Chebyshev => Radius::Square,
            Self::Euclidean => Radius::Circle,
            Self::Manhattan => Radius::Diamond,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Calculates the distance between two points.
    //---------------------------------------------------------------------------------------------
    pub fn calculate(&self, (x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> f64 {
        let dx = (x1 - x2).abs() as f64;
        let dy = (y1 - y2).abs() as f64;
        self.calculate_slope(dx, dy)
    }

    //---------------------------------------------------------------------------------------------
    // Calculates the distance given x and y slope.
    //---------------------------------------------------------------------------------------------
    pub fn calculate_slope(&self, dx: f64, dy: f64) -> f64 {
        match self {
            Self::Chebyshev => f64::max(dx, f64::max(dy, 0.0)),
            Self::Euclidean => ((dx * dx) + (dy * dy)).sqrt(),
            Self::Manhattan => dx + dy,
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Tests.
//-------------------------------------------------------------------------------------------------

#[test]
fn test_distance_calculate_chebyshev() {
    let p1 = (2, 5);
    let p2 = (6, 13);
    let distance = Distance::Chebyshev.calculate(p1, p2);
    let expected = 8.0;
    assert_eq!(distance, expected);
}

#[test]
fn test_distance_calculate_euclidean() {
    let p1 = (2, 5);
    let p2 = (6, 13);
    let distance = Distance::Euclidean.calculate(p1, p2);
    let expected = 8.944;
    assert!((distance - expected).abs() < 0.001);
}

#[test]
fn test_distance_calculate_manhattan() {
    let p1 = (2, 5);
    let p2 = (6, 13);
    let distance = Distance::Manhattan.calculate(p1, p2);
    let expected = 12.0;
    assert_eq!(distance, expected);
}
