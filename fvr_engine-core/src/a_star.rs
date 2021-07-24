//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

pub struct AStar {
    queue: PriorityQueue<(u32, u32), OrderedFloat<f64>>,
}

impl AStar {

}

pub struct FastAStar;