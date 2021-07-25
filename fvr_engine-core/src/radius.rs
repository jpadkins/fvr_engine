//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::adjacency::*;
use crate::distance::*;

//-------------------------------------------------------------------------------------------------
// Enumerates the shape options.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Radius {
    // A circle around a point.
    Circle,
    // A diamond around a point.
    Diamond,
    // A square around a point.
    Square,
}

impl Radius {
    //---------------------------------------------------------------------------------------------
    // Returns the adjacency for a radius.
    //---------------------------------------------------------------------------------------------
    pub fn adjacency(&self) -> Adjacency {
        match self {
            Self::Circle | Self::Square => Adjacency::EightWay,
            Self::Diamond => Adjacency::Cardinals,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the distance for a radius.
    //---------------------------------------------------------------------------------------------
    pub fn distance(&self) -> Distance {
        match self {
            Self::Circle => Distance::Euclidean,
            Self::Diamond => Distance::Manhattan,
            Self::Square => Distance::Chebyshev,
        }
    }
}
