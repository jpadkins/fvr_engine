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
    edges: HashSet<ICoord>,
    // Vec for iterating edges.
    edges_vec: Vec<ICoord>,
    // Set of walkable coords.
    walkable: HashSet<ICoord>,
    // Stores the input states.
    states: GridMap<DijkstraState>,
    // Stores the output weights.
    weights: GridMap<Option<f32>>,
    // Processed coord with the most weight.
    farthest_xy: ICoord,
    // The distance method.
    distance: Distance,
}

impl DijkstraMap {
    //---------------------------------------------------------------------------------------------
    // Creates a new dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn new(dimensions: ICoord, distance: Distance) -> Self {
        Self {
            processed: GridMap::new(dimensions),
            edges: HashSet::new(),
            edges_vec: Vec::new(),
            walkable: HashSet::new(),
            states: GridMap::new(dimensions),
            weights: GridMap::new(dimensions),
            farthest_xy: INVALID_ICOORD,
            distance,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the coord with the most weight. May be one of multiple equal weighted coords.
    //---------------------------------------------------------------------------------------------
    pub fn farthest_xy(&self) -> ICoord {
        self.farthest_xy
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the set of walkable coords of the dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn walkable(&self) -> &HashSet<ICoord> {
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
    // Combines the weight value at a coord by a modifier value.
    //---------------------------------------------------------------------------------------------
    pub fn combine_xy(&mut self, xy: ICoord, modifier: f32) {
        if let Some(weight) = self.weights.get_xy_mut(xy) {
            *weight = *weight + modifier;
        }
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
        let start_weight = (self.states.width() * self.states.height()) as f32;

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
                    *self.weights.get_xy_mut(*coord) = Some(start_weight);
                }
                &DijkstraState::Goal(weight) => {
                    // Set all goal coords to their weight and add them as edges.
                    *self.weights.get_xy_mut(*coord) = Some(weight as f32);
                    self.edges.insert(*coord);
                }
                _ => {}
            }
        }

        // Iterate the edges until all coords have been processed.
        let mut max_weight = 0.0;

        self.edges_vec.clear();

        while !self.edges.is_empty() {
            // Copy the edges into a vec so we can mutate the set inside the loop.
            self.edges_vec.extend(self.edges.iter());

            for edge in self.edges_vec.iter() {
                // Find the current weight at the edge (which will always be Some).
                let current_weight = self.weights.get_xy(*edge).unwrap();

                // Iterate all neighboring coords around the edge.
                for neighbor in adjacency.neighbors(*edge) {
                    // If the neighbor is out of bounds, has been processed or is blocked, continue.
                    if !self.states.in_bounds_icoord(neighbor) {
                        continue;
                    }

                    if *self.processed.get_xy(neighbor) || !self.walkable.contains(&neighbor) {
                        continue;
                    }

                    // Calculate the new weight for the neighbor (which will always be Some).
                    let neighbor_weight = self.weights.get_xy(neighbor).unwrap();
                    let new_weight = current_weight + self.distance.calculate(*edge, neighbor);

                    // If the new weight is less (closer) than the previous weight, update and
                    // add the neighbor to the queue of edges to process.
                    if new_weight < neighbor_weight {
                        *self.weights.get_xy_mut(neighbor) = Some(new_weight);
                        self.edges.insert(neighbor);
                    }

                    if new_weight > max_weight {
                        max_weight = new_weight;
                        self.farthest_xy = neighbor;
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

impl Map2dViewMut for DijkstraMap {
    type Type = Option<f32>;

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at an index.
    //---------------------------------------------------------------------------------------------
    fn get_mut(&mut self, index: usize) -> &mut Self::Type {
        self.weights.get_mut(index)
    }

    //---------------------------------------------------------------------------------------------
    // Get mut ref to contents of the Map2dView at a coord.
    //---------------------------------------------------------------------------------------------
    fn get_xy_mut(&mut self, xy: ICoord) -> &mut Self::Type {
        self.weights.get_xy_mut(xy)
    }
}
