//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::collections::HashSet;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::dijkstra_map::*;
use crate::direction::*;
use crate::distance::*;
use crate::grid_map::*;
use crate::map2d::*;
use crate::map2d_iter_index;
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
const FLEE_MAP_MAGNITUDE: f32 = -1.2;

//-------------------------------------------------------------------------------------------------
// FleeMap describes a 2D map of weights for determining the optimal path to flee.
// Adapted from http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps
//-------------------------------------------------------------------------------------------------
pub struct FleeMap {
    // Helper hash set for storing processed cooords.
    processed: HashSet<ICoord>,
    // Helper hash set for storing coords to process.
    edges: HashSet<ICoord>,
    // Helper vec for iterating edges.
    edges_vec: Vec<ICoord>,
    // Calculated weights of the flee map.
    weights: GridMap<Option<f32>>,
    // Priority queue used for calculating weights.
    queue: PriorityQueue<ICoord, OrderedFloat<f32>>,
    // The distance method.
    distance: Distance,
}

impl FleeMap {
    //---------------------------------------------------------------------------------------------
    // Creates a new flee map.
    //---------------------------------------------------------------------------------------------
    pub fn new(dimensions: ICoord, distance: Distance) -> Self {
        Self {
            processed: HashSet::new(),
            edges: HashSet::new(),
            edges_vec: Vec::new(),
            weights: GridMap::new(dimensions),
            queue: PriorityQueue::new(),
            distance,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the direction of the min weight relative to a coord.
    //---------------------------------------------------------------------------------------------
    pub fn min_direction(&self, xy: ICoord) -> Direction {
        let mut min_weight = f32::MAX;
        let mut direction = NULL_DIRECTION;
        let adjacency = self.distance.adjacency();

        for dir in adjacency.iter() {
            let coord = (xy.0 as i32 + dir.dx(), xy.1 as i32 + dir.dy());

            if !self.weights.in_bounds_icoord(coord) {
                continue;
            }

            if let Some(weight) = self.weights.get_xy(coord) {
                if *weight < min_weight {
                    min_weight = *weight;
                    direction = *dir;
                }
            }
        }

        direction
    }

    //---------------------------------------------------------------------------------------------
    // Returns the direction of the max weight relative to a coord.
    //---------------------------------------------------------------------------------------------
    pub fn max_direction(&self, xy: ICoord) -> Direction {
        let mut max_weight = f32::MIN;
        let mut direction = NULL_DIRECTION;
        let adjacency = self.distance.adjacency();

        for dir in adjacency.iter() {
            let coord = (xy.0 as i32 + dir.dx(), xy.1 as i32 + dir.dy());

            if !self.weights.in_bounds_icoord(coord) {
                continue;
            }

            if let Some(weight) = self.weights.get_xy(coord) {
                if *weight > max_weight {
                    max_weight = *weight;
                    direction = *dir;
                }
            }
        }

        direction
    }

    //---------------------------------------------------------------------------------------------
    // Combines the weights of another dijkstra / flee map into the dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn combine<M>(&mut self, weights: &M)
    where
        M: Map2dView<Type = Option<f32>>,
    {
        map2d_iter_index!(weights, x, y, item, {
            if let Some(weight) = self.weights.get_xy_mut((x, y)) {
                if let Some(other_weight) = item {
                    *weight += other_weight;
                }
            }
        });
    }

    //---------------------------------------------------------------------------------------------
    // Calculates the flee map weights from a dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn calculate(&mut self, states: &DijkstraMap) {
        // Find the adjacency method and clear the states.
        let adjacency = self.distance.adjacency();

        self.processed.clear();
        self.edges.clear();
        self.edges_vec.clear();
        self.queue.clear();

        // Inverse the weights of the walkable coords and populate the queue.
        for coord in states.walkable() {
            let weight = states.get_xy(*coord).unwrap();
            let inverse_weight = weight * FLEE_MAP_MAGNITUDE;
            *self.weights.get_xy_mut(*coord) = Some(inverse_weight);
            self.queue.push(*coord, OrderedFloat(inverse_weight));
        }

        // Iterate until all coords have been processed.
        while !self.queue.is_empty() {
            let next = self.queue.pop().unwrap().0;
            self.processed.insert(next);

            // Iterate all neighboring coords around the next coord, populating the edge set.
            for neighbor in adjacency.neighbors(next) {
                if neighbor.0 >= states.width() as i32 || neighbor.1 >= states.height() as i32 {
                    continue;
                }

                if !self.processed.contains(&neighbor) && states.walkable().contains(&neighbor) {
                    self.edges.insert(neighbor);
                }
            }

            while !self.edges.is_empty() {
                // Copy the edges into a vec so we can mutate the set inside the loop.
                self.edges_vec.extend(self.edges.iter());

                for edge in self.edges_vec.iter() {
                    // Find the current weight at the edge (which will always be Some).
                    let current_weight = self.weights.get_xy(*edge).unwrap();

                    // Iterate all neighboring coords around the edge.
                    for neighbor in adjacency.neighbors(*edge) {
                        // If the neighbor has been processed or is blocked, continue.
                        if neighbor.0 >= states.width() as i32
                            || neighbor.1 >= states.height() as i32
                        {
                            continue;
                        }

                        if self.processed.contains(&neighbor)
                            || !states.walkable().contains(&neighbor)
                        {
                            continue;
                        }

                        // Calculate the new weight for the neighbor (which will always be Some).
                        let neighbor_weight = self.weights.get_xy(neighbor).unwrap();
                        let new_weight = current_weight + self.distance.calculate(*edge, neighbor);

                        // If the new weight is less (closer) than the previous weight, update and
                        // add the neighbor to the queue of edges to process.
                        if new_weight < neighbor_weight {
                            *self.weights.get_xy_mut(neighbor) = Some(new_weight);
                            self.queue.change_priority(&neighbor, OrderedFloat(new_weight));
                            self.edges.insert(neighbor);
                        }
                    }

                    // Set the edge as processed.
                    self.edges.remove(edge);
                    self.processed.insert(*edge);
                    let _ = self.queue.remove(edge).unwrap();
                }

                self.edges_vec.clear();
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dView for FleeMap.
//-------------------------------------------------------------------------------------------------
impl Map2dView for FleeMap {
    type Type = Option<f32>;

    //---------------------------------------------------------------------------------------------
    // Return the width of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn width(&self) -> i32 {
        self.weights.width()
    }

    //---------------------------------------------------------------------------------------------
    // Return the height of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn height(&self) -> i32 {
        self.weights.height()
    }

    //---------------------------------------------------------------------------------------------
    // Return the dimensions of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn dimensions(&self) -> ICoord {
        self.weights.dimensions()
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get(&self, index: usize) -> &Self::Type {
        self.weights.get(index)
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy(&self, xy: ICoord) -> &Self::Type {
        self.weights.get_xy(xy)
    }
}
