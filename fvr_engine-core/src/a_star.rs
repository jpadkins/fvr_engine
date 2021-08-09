//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::dijkstra_map::*;
use crate::distance::*;
use crate::grid_map::*;
use crate::map2d::*;
use crate::misc::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
const A_STAR_MIN_WEIGHT: f32 = 1.0;

//-------------------------------------------------------------------------------------------------
// Enumerates the possible passability input states for the underlying map.
//-------------------------------------------------------------------------------------------------
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Passability {
    // An impassable point in the map.
    Blocked,
    // A passable point in the map.
    Passable,
}

impl Passability {
    pub fn passable(&self) -> bool {
        self == &Passability::Passable
    }
}

// Impl conversion between bool for convenience.
impl From<bool> for Passability {
    fn from(b: bool) -> Self {
        match b {
            true => Passability::Passable,
            false => Passability::Blocked,
        }
    }
}

impl From<Passability> for bool {
    fn from(passability: Passability) -> Self {
        passability.passable()
    }
}

// Impl conversion between dijkstra state for convenience.
impl From<DijkstraState> for Passability {
    fn from(dijkstra_state: DijkstraState) -> Self {
        match dijkstra_state {
            DijkstraState::Blocked => Self::Blocked,
            DijkstraState::Goal { .. } | DijkstraState::Passable => Self::Passable,
        }
    }
}

impl From<Passability> for DijkstraState {
    fn from(passability: Passability) -> Self {
        match passability {
            Passability::Blocked => DijkstraState::Blocked,
            Passability::Passable => DijkstraState::Passable,
        }
    }
}

// Passable bu default.
impl Default for Passability {
    fn default() -> Self {
        Self::Passable
    }
}

//-------------------------------------------------------------------------------------------------
// Alias for boxed heuristic closure for convenience.
//-------------------------------------------------------------------------------------------------
type Heuristic = Box<dyn Fn(ICoord, ICoord) -> f32>;

//-------------------------------------------------------------------------------------------------
// Wraps heuristic data for a node.
//-------------------------------------------------------------------------------------------------
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
struct AStarNode {
    // Coord of the node.
    xy: ICoord,
    // Distance from the start to the node.
    depth: OrderedFloat<f32>,
    // Distance from the node to the end.
    distance_left: OrderedFloat<f32>,
    // Coord of the node's parent.
    parent: Option<ICoord>,
}

//-------------------------------------------------------------------------------------------------
// AStar calculates A* pathfinding.
//-------------------------------------------------------------------------------------------------
pub struct AStar {
    // Map of nodes for queue.
    nodes: GridMap<Option<AStarNode>>,
    // Stores processed state for coords.
    processed: GridMap<bool>,
    // Dimensions of the last map.
    previous_dimensions: ICoord,
    // Queue used when calculating path.
    queue: PriorityQueue<ICoord, OrderedFloat<f32>>,
    // Multiplier used for tie breaking in default heuristic.
    tie_breaker: f32,
    // The weight heuristic.
    heuristic: Heuristic,
    // The distance method.
    distance: Distance,
    // Whether to speed up the heuristic at the cost of accuracy.
    fast: bool,
}

impl AStar {
    //-------------------------------------------------------------------------------------------------
    // Helper for quick distance comparison.
    //-------------------------------------------------------------------------------------------------
    fn distance_magnitude(p1: ICoord, p2: ICoord) -> f32 {
        ((p2.0 - p1.0).pow(2) + (p2.1 - p1.1).pow(2)) as f32
    }

    //-------------------------------------------------------------------------------------------------
    // Helper for refreshing tie breaker value.
    //-------------------------------------------------------------------------------------------------
    fn tie_breaker(dimensions: ICoord) -> f32 {
        A_STAR_MIN_WEIGHT / Self::distance_magnitude((0, 0), dimensions) as f32
    }

    //-------------------------------------------------------------------------------------------------
    // Helper for creating a new heuristic closure.
    //-------------------------------------------------------------------------------------------------
    fn heuristic(distance: Distance, tie_breaker: f32) -> Heuristic {
        Box::new(move |p1, p2| {
            distance.calculate(p1, p2) + (Self::distance_magnitude(p1, p2) * tie_breaker)
        })
    }

    //-------------------------------------------------------------------------------------------------
    // Creates a new a-star.
    //-------------------------------------------------------------------------------------------------
    pub fn new(distance: Distance) -> Self {
        let tie_breaker = 1.0;

        Self {
            nodes: GridMap::new((0, 0)),
            processed: GridMap::new((0, 0)),
            previous_dimensions: (0, 0),
            queue: PriorityQueue::new(),
            tie_breaker,
            heuristic: Self::heuristic(distance, tie_breaker),
            distance,
            fast: false,
        }
    }

    //-------------------------------------------------------------------------------------------------
    // Creates a new "fast" a-star, giving a speed boost at the cost of accuracy.
    //-------------------------------------------------------------------------------------------------
    pub fn fast(distance: Distance) -> Self {
        let tie_breaker = 1.0;

        Self {
            nodes: GridMap::new((0, 0)),
            processed: GridMap::new((0, 0)),
            previous_dimensions: (0, 0),
            queue: PriorityQueue::new(),
            tie_breaker,
            heuristic: Self::heuristic(Distance::Manhattan, tie_breaker),
            distance,
            fast: true,
        }
    }

    //-------------------------------------------------------------------------------------------------
    // Calculates the shortest path between two points and pushes it into a vec.
    //-------------------------------------------------------------------------------------------------
    pub fn push_path<M>(
        &mut self,
        start: ICoord,
        end: ICoord,
        states: &M,
        weights: Option<&GridMap<f32>>,
        points: &mut Vec<ICoord>,
    ) where
        M: Map2d<Passability>,
    {
        // If the start and end coords are equal, or either are not passable, return.
        if start == end
            // TODO: Should we always assume the starting coord is passable?
            // || *states.get_xy(start) == Passability::Blocked
            || *states.get_xy(end) == Passability::Blocked
        {
            return;
        }

        // Check if dimensions of states are different than the previous and refresh if necessary.
        let dimensions = states.dimensions();
        let adjacency = self.distance.adjacency();

        // Always clear the queue.
        self.queue.clear();

        if dimensions != self.previous_dimensions {
            self.nodes.resize(dimensions);
            self.processed.resize(dimensions);
            self.previous_dimensions = dimensions;
            self.tie_breaker = Self::tie_breaker(dimensions);

            // Fast heuristic always uses Manhattan distance.
            if self.fast {
                self.heuristic = Self::heuristic(Distance::Manhattan, self.tie_breaker);
            } else {
                self.heuristic = Self::heuristic(self.distance, self.tie_breaker);
            }

            // Resize capacity to known limit ahead of time.
            self.queue.reserve((dimensions.0 * dimensions.1) as usize);
        }

        // Clear the nodes map to None and the processed map to false.
        self.nodes.data_mut().fill(None);
        self.processed.data_mut().fill(false);

        // Calculate the heuristics for the start node and push it into the queue.
        let start_node = self.nodes.get_xy_mut(start);
        let depth = OrderedFloat(0.0);
        let distance_left = OrderedFloat((self.heuristic)(start, end));

        if let Some(node) = start_node {
            node.depth = depth;
            node.distance_left = distance_left;
        } else {
            *start_node = Some(AStarNode { xy: start, depth, distance_left, parent: None });
        }

        self.queue.push(start, -distance_left);

        // Begin constructing the path.
        while !self.queue.is_empty() {
            // Retrieve the next node in the queue and set it as processed.
            let node = self.queue.pop().unwrap();
            *self.processed.get_xy_mut(node.0) = true;

            // If we have reached the end, populate the path and return.
            if node.0 == end {
                points.push(node.0);
                let mut node = self.nodes.get_xy(node.0);
                let mut xy = ICoord::default();

                while {
                    if let Some(parent_xy) = node.as_ref().unwrap().parent {
                        points.push(parent_xy);
                        xy = parent_xy;

                        parent_xy != start
                    } else {
                        false
                    }
                } {
                    node = self.nodes.get_xy(xy);
                }

                return;
            }

            // Process the node.
            for xy in adjacency.neighbors(node.0) {
                // Continue if the neighbor is not valid.
                if !states.in_bounds(xy) {
                    continue;
                }

                // Continue if the neighbor is not passable.
                if *states.get_xy(xy) == Passability::Blocked {
                    continue;
                }

                let is_visited = self.nodes.get_xy(xy).is_some();

                // If the coord has been visited and processed, continue.
                if is_visited && *self.processed.get_xy(xy) {
                    continue;
                }

                // Calculate new depth value.
                let mut depth = self.distance.calculate(node.0, xy);

                if weights.is_some() {
                    depth *= *weights.as_ref().unwrap().get_xy(xy);
                }

                let depth =
                    OrderedFloat(depth) + self.nodes.get_xy(node.0).as_ref().unwrap().depth;

                // Ensure the node is initialized.
                if !is_visited {
                    *self.nodes.get_xy_mut(xy) = Some(AStarNode {
                        xy,
                        depth: OrderedFloat(f32::MAX),
                        distance_left: OrderedFloat(f32::MAX),
                        parent: None,
                    });
                }

                // If the node has been processed and the previous depth is not higher, continue.
                let is_enqueued = self.queue.get(&xy).is_some();

                if (is_visited && is_enqueued)
                    && depth >= self.nodes.get_xy(xy).as_ref().unwrap().depth
                {
                    continue;
                }

                // Otherwise this is a better path and the node should be updated.
                let mut neighbor = self.nodes.get_xy_mut(xy).as_mut().unwrap();
                neighbor.parent = Some(node.0);
                neighbor.depth = depth;
                neighbor.distance_left = depth + OrderedFloat((self.heuristic)(xy, end));

                if is_enqueued {
                    self.queue.change_priority(&xy, -neighbor.distance_left);
                } else {
                    self.queue.push(xy, -neighbor.distance_left);
                }
            }
        }
    }

    //-------------------------------------------------------------------------------------------------
    // Calculates the shortest path between two points and returns it in a new vec.
    //-------------------------------------------------------------------------------------------------
    pub fn path<M>(
        &mut self,
        start: ICoord,
        end: ICoord,
        states: &M,
        weights: Option<&GridMap<f32>>,
    ) -> Vec<ICoord>
    where
        M: Map2d<Passability>,
    {
        let mut points = Vec::new();
        self.push_path(start, end, states, weights, &mut points);
        points
    }
}
