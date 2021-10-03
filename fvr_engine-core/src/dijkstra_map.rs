//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use fnv::FnvHashSet;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::direction::*;
use crate::distance::*;
use crate::grid_map::*;
use crate::map2d::*;
use crate::map2d_iter_index;
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
pub const DIJKSTRA_DEFAULT_GOAL: DijkstraState = DijkstraState::Goal(0);

//-------------------------------------------------------------------------------------------------
// Enumerates the possible input states for the underlying map.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DijkstraState {
    // An impassable point in the map.
    Blocked,
    // A passable point in the map.
    Passable,
    // A goal in the map.
    Goal(i32),
}

impl DijkstraState {
    pub fn passable(&self) -> bool {
        self == &DijkstraState::Passable
    }
}

// Impl conversion between bool for convenience.
impl From<bool> for DijkstraState {
    fn from(b: bool) -> Self {
        if b {
            DijkstraState::Passable
        } else {
            DijkstraState::Blocked
        }
    }
}

impl From<DijkstraState> for bool {
    fn from(dijkstra_state: DijkstraState) -> Self {
        dijkstra_state.passable()
    }
}

// Passable bu default.
impl Default for DijkstraState {
    fn default() -> Self {
        DijkstraState::Passable
    }
}

//-------------------------------------------------------------------------------------------------
// DijkstraMap describes a 2D map of weights related to goals.
// Adapted from http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps
// NOTE: ONLY supports dimensions up to 255x255!
// NOTE: An alternative constructor and re/calculate methods are provided for a "thin" dijkstrap
//  map in which no internal state grid map is managed. This allows for cutting down on memory
//  usage in the case where multiple structs share the same state (since the grid map's internal
//  data vec would not need to be allocated).
//-------------------------------------------------------------------------------------------------
pub struct DijkstraMap {
    // Stores processed state for coords.
    processed: GridMap<bool>,
    // Hash set for storing coords to process.
    edges: FnvHashSet<(u8, u8)>,
    // Vec for iterating edges.
    edges_vec: Vec<(u8, u8)>,
    // Set of walkable coords.
    walkable: FnvHashSet<(u8, u8)>,
    // Stores the input states.
    states: Option<GridMap<DijkstraState>>,
    // Stores the output weights.
    weights: GridMap<Option<f32>>,
    // Processed coord with the most weight.
    highest_xy: ICoord,
    // The distance method.
    distance: Distance,
}

//-------------------------------------------------------------------------------------------------
// Implementation for both recalculate and recalculate_thin methods below to avoid duplication.
//-------------------------------------------------------------------------------------------------
macro_rules! recalculate_impl {
    ($self:ident, $states:ident) => {
        // Find the adjacency method and max weight value.
        let adjacency = $self.distance.adjacency();
        let start_weight = ($states.width() * $states.height()) as f32;

        // Clear the processed map and edges set.
        $self.processed.data_mut().fill(false);
        $self.edges.clear();

        // Find and set the initial weights for passable and goal coords.
        for coord in $self.walkable.iter() {
            let icoord = (coord.0 as i32, coord.1 as i32);

            match $states.get_xy(icoord).clone().into() {
                DijkstraState::Passable => {
                    // Set all passable coords to the max weight.
                    *$self.weights.get_xy_mut(icoord) = Some(start_weight);
                }
                DijkstraState::Goal(weight) => {
                    // Set all goal coords to their weight and add them as edges.
                    *$self.weights.get_xy_mut(icoord) = Some(weight as f32);
                    $self.edges.insert(*coord);
                }
                _ => {}
            }
        }

        // Iterate the edges until all coords have been processed.
        $self.highest_xy = INVALID_ICOORD;
        let mut max_weight = f32::MIN;

        $self.edges_vec.clear();

        while !$self.edges.is_empty() {
            // Copy the edges into a vec so we can mutate the set inside the loop.
            $self.edges_vec.extend($self.edges.iter());

            for edge in $self.edges_vec.iter() {
                let iedge = (edge.0 as i32, edge.1 as i32);

                // Find the current weight at the edge (which will always be Some).
                let current_weight = $self.weights.get_xy(iedge).unwrap();

                // Iterate all neighboring coords around the edge.
                for neighbor in adjacency.neighbors(iedge) {
                    // If neighbor is out of bounds, has been processed or is blocked, continue.
                    if !$states.in_bounds(neighbor)
                        || *$self.processed.get_xy(neighbor)
                        || !Into::<DijkstraState>::into($states.get_xy(neighbor).clone())
                            .passable()
                    {
                        continue;
                    }

                    // Calculate the new weight for the neighbor (which will always be Some).
                    let neighbor_weight = $self.weights.get_xy(neighbor).unwrap();
                    let new_weight = current_weight + $self.distance.calculate(iedge, neighbor);

                    // If the new weight is less (closer) than the previous weight, update and
                    // add the neighbor to the queue of edges to process.
                    if new_weight < neighbor_weight {
                        *$self.weights.get_xy_mut(neighbor) = Some(new_weight);

                        let coord = (neighbor.0 as u8, neighbor.1 as u8);
                        $self.edges.insert(coord);
                    }

                    if new_weight > max_weight {
                        max_weight = new_weight;
                        $self.highest_xy = neighbor;
                    }
                }

                // Set the edge as processed.
                $self.edges.remove(edge);
                *$self.processed.get_xy_mut(iedge) = true;
            }

            $self.edges_vec.clear();
        }
    };
}

impl DijkstraMap {
    //---------------------------------------------------------------------------------------------
    // Creates a new dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn new(dimensions: ICoord, distance: Distance) -> Self {
        Self {
            processed: GridMap::new(dimensions),
            edges: FnvHashSet::default(),
            edges_vec: Vec::new(),
            walkable: FnvHashSet::default(),
            states: Some(GridMap::new(dimensions)),
            weights: GridMap::new(dimensions),
            highest_xy: INVALID_ICOORD,
            distance,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Creates a new dijkstra map that does not track states.
    //---------------------------------------------------------------------------------------------
    pub fn new_thin(dimensions: ICoord, distance: Distance) -> Self {
        Self {
            processed: GridMap::new(dimensions),
            edges: FnvHashSet::default(),
            edges_vec: Vec::new(),
            walkable: FnvHashSet::default(),
            states: None,
            weights: GridMap::new(dimensions),
            highest_xy: INVALID_ICOORD,
            distance,
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the coord with the most weight. May be one of multiple equal weighted coords.
    //---------------------------------------------------------------------------------------------
    pub fn highest_xy(&self) -> Option<ICoord> {
        if self.highest_xy != INVALID_ICOORD {
            Some(self.highest_xy)
        } else {
            None
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the set of walkable coords of the dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn walkable(&self) -> &FnvHashSet<(u8, u8)> {
        &self.walkable
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the weights of the dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn weights(&self) -> &GridMap<Option<f32>> {
        &self.weights
    }

    //---------------------------------------------------------------------------------------------
    // Returns a ref to the states of the dijkstra map.
    // NOTE: Panics if called on a thin dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn states(&self) -> &GridMap<DijkstraState> {
        self.states.as_ref().unwrap()
    }

    //---------------------------------------------------------------------------------------------
    // Returns a mut ref to the states of the dijkstra map.
    // NOTE: Panics if called on a thin dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn states_mut(&mut self) -> &mut GridMap<DijkstraState> {
        self.states.as_mut().unwrap()
    }

    //---------------------------------------------------------------------------------------------
    // Returns the neighbor with the min weight relative to a coord.
    //---------------------------------------------------------------------------------------------
    pub fn best_neighbor(&self, xy: ICoord) -> Option<(ICoord, f32)> {
        // Return if no path to a goal exists.
        if self.highest_xy == INVALID_ICOORD {
            return None;
        }

        let mut set = false;
        let mut neighbor = xy;
        let adjacency = self.distance.adjacency();
        let mut min_weight = self.weights.get_xy(self.highest_xy).unwrap();

        // Find the best neighbor.
        for n in adjacency.neighbors(xy) {
            if !self.weights.in_bounds(n) {
                continue;
            }

            if let Some(weight) = self.weights.get_xy(n) {
                if *weight < min_weight {
                    set = true;
                    min_weight = *weight;
                    neighbor = n;
                }
            }
        }

        if set {
            Some((neighbor, min_weight))
        } else {
            None
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the neighbor with the min weight relative to a coord and less than some value.
    //---------------------------------------------------------------------------------------------
    pub fn best_neighbor_lt(&self, xy: ICoord, mut min_weight: f32) -> Option<(ICoord, f32)> {
        // Return if no path to a goal exists.
        if self.highest_xy == INVALID_ICOORD {
            return None;
        }

        let mut set = false;
        let mut neighbor = xy;
        let adjacency = self.distance.adjacency();

        // Find the best neighbor.
        for n in adjacency.neighbors(xy) {
            if !self.weights.in_bounds(n) {
                continue;
            }

            if let Some(weight) = self.weights.get_xy(n) {
                if *weight < min_weight {
                    set = true;
                    min_weight = *weight;
                    neighbor = n;
                }
            }
        }

        if set {
            Some((neighbor, min_weight))
        } else {
            None
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the direction of the min weight relative to a coord
    //---------------------------------------------------------------------------------------------
    pub fn best_direction(&self, xy: ICoord) -> Option<(Direction, f32)> {
        // Return if no path to a goal exists.
        if self.highest_xy == INVALID_ICOORD {
            return None;
        }

        let mut set = false;
        let mut direction = NULL_DIRECTION;
        let adjacency = self.distance.adjacency();
        let mut min_weight = self.weights.get_xy(self.highest_xy).unwrap();

        // Find the best direction.
        for dir in adjacency.iter() {
            let coord = (xy.0 + dir.dx(), xy.1 + dir.dy());

            if !self.weights.in_bounds(coord) {
                continue;
            }

            if let Some(weight) = self.weights.get_xy(coord) {
                if *weight < min_weight {
                    set = true;
                    min_weight = *weight;
                    direction = *dir;
                }
            }
        }

        if set {
            Some((direction, min_weight))
        } else {
            None
        }
    }

    //---------------------------------------------------------------------------------------------
    // Returns the direction of the min weight relative to a coord and less than some value.
    //---------------------------------------------------------------------------------------------
    pub fn best_direction_lt(&self, xy: ICoord, mut min_weight: f32) -> Option<(Direction, f32)> {
        // Return if no path to a goal exists.
        if self.highest_xy == INVALID_ICOORD {
            return None;
        }

        let mut set = false;
        let mut direction = NULL_DIRECTION;
        let adjacency = self.distance.adjacency();

        // Find the best direction.
        for dir in adjacency.iter() {
            let coord = (xy.0 + dir.dx(), xy.1 + dir.dy());

            if !self.weights.in_bounds(coord) {
                continue;
            }

            if let Some(weight) = self.weights.get_xy(coord) {
                if *weight < min_weight {
                    set = true;
                    min_weight = *weight;
                    direction = *dir;
                }
            }
        }

        if set {
            Some((direction, min_weight))
        } else {
            None
        }
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
            *weight += modifier;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Invert all of the weights in the dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn invert(&mut self) {
        for item in self.weights.data_mut().iter_mut().flatten() {
            *item *= -1.0;
        }
    }

    //---------------------------------------------------------------------------------------------
    // Refreshes the highest weight and xy value. Call when weights are manually adjusted.
    //---------------------------------------------------------------------------------------------
    pub fn refresh_highest(&mut self) {
        self.highest_xy = INVALID_ICOORD;
        let mut max_weight = f32::MIN;

        map2d_iter_index!(self.weights, x, y, item, {
            if let Some(weight) = item {
                if *weight > max_weight {
                    max_weight = *weight;
                    self.highest_xy = (x, y);
                }
            }
        });
    }

    //---------------------------------------------------------------------------------------------
    // Calculates the output weights.
    // NOTE: Must be called once whenever any blocked state changes.
    // NOTE: Panics if called on a thin dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn calculate(&mut self) {
        // Aquire reference to states.
        if self.states.is_none() {
            panic!("calculate called on thin dijkstra map!");
        }
        let states = self.states.as_ref().unwrap();

        // Recreate the set of walkable coords.
        self.walkable.clear();

        map2d_iter_index!(states, x, y, state, {
            match state {
                DijkstraState::Blocked => {
                    *self.weights.get_xy_mut((x, y)) = None;
                }
                _ => {
                    self.walkable.insert((x as u8, y as u8));
                }
            }
        });

        // Recalculate the weights.
        self.recalculate();
    }

    //---------------------------------------------------------------------------------------------
    // Calculates the output weights given a grid map of dijkstra state.
    // Must be called once whenever any blocked state changes.
    // Intended for usage with thin dijkstra maps.
    //---------------------------------------------------------------------------------------------
    pub fn calculate_thin<M, T>(&mut self, states: &M)
    where
        M: Map2d<T>,
        T: Map2dType + Into<DijkstraState>,
    {
        // Recreate the set of walkable coords.
        self.walkable.clear();

        map2d_iter_index!(states, x, y, state, {
            match state.clone().into() {
                DijkstraState::Blocked => {
                    *self.weights.get_xy_mut((x, y)) = None;
                }
                _ => {
                    self.walkable.insert((x as u8, y as u8));
                }
            }
        });

        // Recalculate the weights.
        self.recalculate_thin(states);
    }

    //---------------------------------------------------------------------------------------------
    // Recalculates the output weights faster, but if only the passable/goals states change.
    // NOTE: Panics if called on a thin dijkstra map.
    //---------------------------------------------------------------------------------------------
    pub fn recalculate(&mut self) {
        // Aquire reference to states.
        if self.states.is_none() {
            panic!("recalculate called on thin dijkstra map!");
        }
        let states = self.states.as_ref().unwrap();

        // See macro above for details.
        recalculate_impl!(self, states);
    }

    //---------------------------------------------------------------------------------------------
    // Recalculates the output weights faster, but if only the passable/goals states change.
    // Intended for usage with thin dijkstra maps.
    //---------------------------------------------------------------------------------------------
    pub fn recalculate_thin<M, T>(&mut self, states: &M)
    where
        M: Map2d<T>,
        T: Map2dType + Into<DijkstraState>,
    {
        // See macro above for details.
        recalculate_impl!(self, states);
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
