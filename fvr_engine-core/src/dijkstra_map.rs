//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::collections::HashSet;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::distance::*;
use crate::grid_map::*;
use crate::map2d_iter_index;
use crate::traits::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
pub const DIJKSTRA_DEFAULT_GOAL: DijkstraState = DijkstraState::Goal(0);

//-------------------------------------------------------------------------------------------------
// Enumerates the possible input states for the dijkstra map.
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
    // Helper hash set for storing processed cooords.
    processed: HashSet<(u32, u32)>,
    // Helper hash set for storing coords to process.
    edges: HashSet<(u32, u32)>,
    // Set of walkable coords.
    walkable: HashSet<(u32, u32)>,
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
    pub fn new(dimensions: (u32, u32), distance: Distance) -> Self {
        Self {
            processed: HashSet::new(),
            edges: HashSet::new(),
            walkable: HashSet::new(),
            states: GridMap::new(dimensions.0, dimensions.1),
            weights: GridMap::new(dimensions.0, dimensions.1),
            distance,
        }
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

    // pub fn min_direction(&self, xy: (u32, u32)) -> Direction
    // pub fn max_direction(&self, xy: (u32, u32)) -> Direction

    //---------------------------------------------------------------------------------------------
    // Calculates the output weights of the dijkstra map.
    // Must be called once whenever any blocked state changes.
    //---------------------------------------------------------------------------------------------
    pub fn calculate(&mut self) {
        // Recreate the set of walkable coords.
        self.walkable = HashSet::new();

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

        // Clear the processed and edges sets.
        self.processed = HashSet::new();
        self.edges = HashSet::new();

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
        let mut edge_vec: Vec<(u32, u32)> = Vec::new();

        while !self.edges.is_empty() {
            // Copy the edges into a vec so we can mutate the set inside the loop.
            edge_vec.extend(self.edges.iter());

            for edge in edge_vec.iter() {
                // Find the current weight at the edge (which will always be Some).
                let current_weight = self.weights.get_xy(*edge).unwrap();

                // Iterate all neighboring coords around the edge.
                let edge_point = (edge.0 as i32, edge.1 as i32);
                for neighbor in adjacency.neighbors(edge_point) {
                    // If the neighbor has been processed or is blocked, continue.
                    let neighbor_coord = (neighbor.0 as u32, neighbor.1 as u32);
                    if self.processed.contains(&neighbor_coord)
                        || !self.walkable.contains(&neighbor_coord)
                    {
                        continue;
                    }

                    // Calculate the new weight for the neighbor (which will always be Some).
                    let neighbor_weight = self.weights.get_xy(neighbor_coord).unwrap();
                    let new_weight =
                        current_weight + self.distance.calculate(edge_point, neighbor);

                    // If the new weight is less (closer) than the previous weight, update and
                    // add the neighbor to the queue of edges to process.
                    if new_weight < neighbor_weight {
                        *self.weights.get_xy_mut(neighbor_coord) = Some(new_weight);
                        self.edges.insert(neighbor_coord);
                    }
                }

                // Set the edge as processed.
                self.edges.remove(edge);
                self.processed.insert(*edge);
            }

            edge_vec.clear();
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
    // Get ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get(&self, index: usize) -> &Self::Type {
        self.weights.get(index)
    }

    //---------------------------------------------------------------------------------------------
    // Get ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy(&self, xy: (u32, u32)) -> &Self::Type {
        self.weights.get_xy(xy)
    }
}
