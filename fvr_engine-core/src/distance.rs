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
    // Calculates the distance between two points.
    //---------------------------------------------------------------------------------------------
    pub fn calculate(&self, (x1, y1): (i32, i32), (x2, y2): (i32, i32)) -> f64 {
        let dx = (x1 - x2).abs() as f64;
        let dy = (y1 - y2).abs() as f64;

        match self {
            Self::Chebyshev => f64::max(dx, f64::max(dy, 0.0)),
            Self::Euclidean => ((dx * dx) + (dy * dy)).sqrt(),
            Self::Manhattan => dx + dy,
        }
    }
}
