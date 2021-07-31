//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::collections::HashSet;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::direction::*;
use crate::distance::*;
use crate::grid_map::*;
use crate::map2d::*;
use crate::misc::*;
use crate::{map2d_iter_index, map2d_iter_mut};

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
pub const DIJKSTRA_DEFAULT_GOAL: DijkstraState = DijkstraState::Goal(0);

//-------------------------------------------------------------------------------------------------
// Enumerates the possible input states for the underlying map.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DijkstraState {
    // An impassable point in the map.
    Blocked,
    // A passable point in the map.
    Passable,
    // A goal in the map.
    Goal(i32),
}

impl Default for DijkstraState {
    fn default() -> Self {
        DijkstraState::Passable
    }
}

//-------------------------------------------------------------------------------------------------
// DijkstraMap describes a 2D map of weights related to goals.
// Adapted from http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps
//-------------------------------------------------------------------------------------------------
pub struct DijkstraMap {
    // Stores processed state for coords.
    processed: GridMap<bool>,
    // Hash set for storing coords to process.
    edges: HashSet<UCoord>,
    // Vec for iterating edges.
    edges_vec: Vec<UCoord>,
    // Set of walkable coords.
    walkable: HashSet<UCoord>,
    // Stores the input states.
    states: GridMap<DijkstraState>,
    // Stores the output weights.
    weights: GridMap<Option<f64>>,
    // The distance method.
    distance: Distance,
}

impl DijkstraMap {
    //---------------------------------------------------------------------------------------------
    // Creates a new dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn new(dimensions: UCoord, distance: Distance) -> Self {
        Self {
            processed: GridMap::new(dimensions),
            edges: HashSet::new(),
            edges_vec: Vec::new(),
            walkable: HashSet::new(),
            states: GridMap::new(dimensions),
            weights: GridMap::new(dimensions),
            distance,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the set of walkable coords of the dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn walkable(&self) -> &HashSet<UCoord> {
        &self.walkable
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the states of the dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn states(&self) -> &GridMap<DijkstraState> {
        &self.states
    }

    //---------------------------------------------------------------------------------------------
    // Returns a mut ref to the states of the dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn states_mut(&mut self) -> &mut GridMap<DijkstraState> {
        &mut self.states
    }

    //---------------------------------------------------------------------------------------------
    // Returns the direction of the min weight relative to a coord.
    //---------------------------------------------------------------------------------------------
    pub fn min_direction(&self, xy: UCoord) -> Direction {
        let mut min_weight = f64::MAX;
        let mut direction = NULL_DIRECTION;
        let adjacency = self.distance.adjacency();

        for dir in adjacency.iter() {
            let coord = (xy.0 as i32 + dir.dx(), xy.1 as i32 + dir.dy());

            if !self.weights.in_bounds_icoord(coord) {
                continue;
            }

            if let Some(weight) = self.weights.get_xy(Misc::itou(coord)) {
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
    pub fn max_direction(&self, xy: UCoord) -> Direction {
        let mut max_weight = f64::MIN;
        let mut direction = NULL_DIRECTION;
        let adjacency = self.distance.adjacency();

        for dir in adjacency.iter() {
            let coord = (xy.0 as i32 + dir.dx(), xy.1 as i32 + dir.dy());

            if !self.weights.in_bounds_icoord(coord) {
                continue;
            }

            if let Some(weight) = self.weights.get_xy(Misc::itou(coord)) {
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
        M: Map2dView<Type = Option<f64>>,
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
    // Calculates the output weights.
    // Must be called once whenever any blocked state changes.
    //---------------------------------------------------------------------------------------------
    pub fn calculate(&mut self) {
        // Recreate the set of walkable coords.
        self.walkable.clear();

        map2d_iter_index!(self.states, x, y, state, {
            match state {
                DijkstraState::Blocked => {
                    *self.weights.get_xy_mut((x, y)) = None;
                }
                _ => {
                    self.walkable.insert((x, y));
                }
            }
        });

        // Recalculate the weights.
        self.recalculate();
    }

    //---------------------------------------------------------------------------------------------
    // Recalculates the output weights faster, but if only the passable/goals states change.
    //---------------------------------------------------------------------------------------------
    pub fn recalculate(&mut self) {
        // Find the adjacency method and max weight value.
        let adjacency = self.distance.adjacency();
        let max_weight = (self.states.width() * self.states.height()) as f64;

        // Clear the processed map and edges set.
        map2d_iter_mut!(self.processed, item, {
            *item = false;
        });
        self.edges.clear();

        // Find and set the initial weights for passable and goal coords.
        for coord in self.walkable.iter() {
            match self.states.get_xy(*coord) {
                DijkstraState::Passable => {
                    // Set all passable coords to the max weight.
                    *self.weights.get_xy_mut(*coord) = Some(max_weight);
                }
                &DijkstraState::Goal(weight) => {
                    // Set all goal coords to their weight and add them as edges.
                    *self.weights.get_xy_mut(*coord) = Some(weight as f64);
                    self.edges.insert(*coord);
                }
                _ => {}
            }
        }

        // Iterate the edges until all coords have been processed.
        self.edges_vec.clear();

        while !self.edges.is_empty() {
            // Copy the edges into a vec so we can mutate the set inside the loop.
            self.edges_vec.extend(self.edges.iter());

            for edge in self.edges_vec.iter() {
                // Find the current weight at the edge (which will always be Some).
                let current_weight = self.weights.get_xy(*edge).unwrap();

                // Iterate all neighboring coords around the edge.
                let edge_coord = Misc::utoi(*edge);
                for neighbor in adjacency.neighbors(edge_coord) {
                    // If the neighbor is out of bounds, has been processed or is blocked, continue.
                    if !self.states.in_bounds_icoord(neighbor) {
                        continue;
                    }

                    let neighbor_coord = Misc::itou(neighbor);

                    if *self.processed.get_xy(neighbor_coord)
                        || !self.walkable.contains(&neighbor_coord)
                    {
                        continue;
                    }

                    // Calculate the new weight for the neighbor (which will always be Some).
                    let neighbor_weight = self.weights.get_xy(neighbor_coord).unwrap();
                    let new_weight =
                        current_weight + self.distance.calculate(edge_coord, neighbor);

                    // If the new weight is less (closer) than the previous weight, update and
                    // add the neighbor to the queue of edges to process.
                    if new_weight < neighbor_weight {
                        *self.weights.get_xy_mut(neighbor_coord) = Some(new_weight);
                        self.edges.insert(neighbor_coord);
                    }
                }

                // Set the edge as processed.
                self.edges.remove(edge);
                *self.processed.get_xy_mut(*edge) = true;
            }

            self.edges_vec.clear();
        }
    }
}

//-------------------------------------------------------------------------------------------------
// Impl Map2dView for DijkstraMap.
//-------------------------------------------------------------------------------------------------
impl Map2dView for DijkstraMap {
    type Type = Option<f64>;

    //---------------------------------------------------------------------------------------------
    // Return the width of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn width(&self) -> u32 {
        self.weights.width()
    }

    //---------------------------------------------------------------------------------------------
    // Return the height of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn height(&self) -> u32 {
        self.weights.height()
    }

    //---------------------------------------------------------------------------------------------
    // Return the dimensions of the Map2dView.
    //---------------------------------------------------------------------------------------------
    fn dimensions(&self) -> UCoord {
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
    fn get_xy(&self, xy: UCoord) -> &Self::Type {
        self.weights.get_xy(xy)
    }
}
