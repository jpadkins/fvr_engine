//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::cell::RefCell;
use std::rc::Rc;

//-------------------------------------------------------------------------------------------------
// Extern crate includes.
//-------------------------------------------------------------------------------------------------
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;

//-------------------------------------------------------------------------------------------------
// Local includes.
//-------------------------------------------------------------------------------------------------
use crate::distance::*;
use crate::grid_map::*;
use crate::map2d_iter_mut;
use crate::misc::*;
use crate::traits::*;

//-------------------------------------------------------------------------------------------------
// Constants.
//-------------------------------------------------------------------------------------------------
const A_STAR_MIN_WEIGHT: f64 = 1.0;

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

impl Default for Passability {
    fn default() -> Self {
        Self::Passable
    }
}

//-------------------------------------------------------------------------------------------------
// Alias for boxed heuristic closure for convenience.
//-------------------------------------------------------------------------------------------------
type Heuristic = Box<dyn Fn(UCoord, UCoord) -> f64>;

#[derive(Clone, Debug, PartialEq, Eq)]
struct AStarNode {
    xy: UCoord,
    depth: OrderedFloat<f64>,
    distance_left: OrderedFloat<f64>,
    parent: Option<Rc<RefCell<AStarNode>>>,
}

//-------------------------------------------------------------------------------------------------
// AStar calculates A* pathfinding.
//-------------------------------------------------------------------------------------------------
pub struct AStar {
    // Map of nodes for queue.
    nodes: GridMap<Option<Rc<RefCell<AStarNode>>>>,
    // Stores processed state for coords.
    processed: GridMap<bool>,
    // Dimensions of the last map.
    previous_dimensions: UCoord,
    // Queue used when calculating path.
    queue: PriorityQueue<UCoord, OrderedFloat<f64>>,
    // Multiplier used for tie breaking in default heuristic.
    tie_breaker: f64,
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
    fn distance_magnitude(p1: UCoord, p2: UCoord) -> f64 {
        ((p2.0 as i32 - p1.0 as i32).pow(2) + (p2.1 as i32 - p1.1 as i32).pow(2)) as f64
    }

    //-------------------------------------------------------------------------------------------------
    // Helper for refreshing tie breaker value.
    //-------------------------------------------------------------------------------------------------
    fn tie_breaker(dimensions: UCoord) -> f64 {
        A_STAR_MIN_WEIGHT / Self::distance_magnitude((0, 0), dimensions) as f64
    }

    //-------------------------------------------------------------------------------------------------
    // Helper for creating a new heuristic closure.
    //-------------------------------------------------------------------------------------------------
    fn heuristic(distance: Distance, tie_breaker: f64) -> Heuristic {
        Box::new(move |p1, p2| {
            distance.calculate(Misc::utoi(p1), Misc::utoi(p2))
                + (Self::distance_magnitude(p1, p2) * tie_breaker)
        })
    }

    //-------------------------------------------------------------------------------------------------
    // Creates a new a star.
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
    // Creates a new "fast" a star, giving a speed boost at the cost of accuracy.
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
    // Calculates the shortest path between two points and pushes it to a vec.
    //-------------------------------------------------------------------------------------------------
    pub fn push_path<M>(
        &mut self,
        start: UCoord,
        end: UCoord,
        states: &M,
        weights: Option<&GridMap<f64>>,
        points: &mut Vec<UCoord>,
    ) where
        M: Map2d<Passability>,
    {
        // If the start and end coords are equal, or either are not passable, return.
        if start == end
            || *states.get_xy(start) == Passability::Blocked
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
        map2d_iter_mut!(self.nodes, item, {
            *item = None;
        });
        map2d_iter_mut!(self.processed, item, {
            *item = false;
        });

        // Calculate the heuristics for the start node and push it into the queue.
        let start_node = self.nodes.get_xy_mut(start);
        let depth = OrderedFloat(0.0);
        let distance_left = OrderedFloat((self.heuristic)(start, end));

        if let Some(node) = start_node {
            node.borrow_mut().depth = depth;
            node.borrow_mut().distance_left = distance_left;
        } else {
            *start_node = Some(Rc::new(RefCell::new(AStarNode {
                xy: start,
                depth,
                distance_left,
                parent: None,
            })));
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
                let mut xy = UCoord::default();

                while {
                    if node.as_ref().unwrap().borrow().parent.is_some() {
                        xy = node.as_ref().unwrap().borrow().parent.as_ref().unwrap().borrow().xy;
                        points.push(xy);
                        xy != start
                    } else {
                        false
                    }
                } {
                    node = self.nodes.get_xy(xy);
                }

                return;
            }

            // Process the node.
            for xy in adjacency.neighbors(Misc::utoi(node.0)) {
                // Continue if the neighbor is not valid.
                if xy.0 < 0
                    || xy.1 < 0
                    || xy.0 as u32 >= states.width()
                    || xy.1 as u32 >= states.height()
                {
                    continue;
                }

                let xy = Misc::itou(xy);

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
                let mut depth = self.distance.calculate(Misc::utoi(node.0), Misc::utoi(xy));

                if weights.is_some() {
                    depth *= *weights.as_ref().unwrap().get_xy(xy);
                }

                let depth = OrderedFloat(depth)
                    + self.nodes.get_xy(node.0).as_ref().unwrap().borrow().depth;

                // Ensure the node is initialized.
                if !is_visited {
                    *self.nodes.get_xy_mut(xy) = Some(Rc::new(RefCell::new(AStarNode {
                        xy,
                        depth: OrderedFloat(f64::MAX),
                        distance_left: OrderedFloat(f64::MAX),
                        parent: None,
                    })));
                }

                // If the node has been processed and the previous depth is not higher, continue.
                let is_enqueued = self.queue.get(&xy).is_some();

                if (is_visited && is_enqueued)
                    && depth >= self.nodes.get_xy(xy).as_ref().unwrap().borrow().depth
                {
                    continue;
                }

                // Otherwise this is a better path and the node should be updated.
                let parent = Some(self.nodes.get_xy_mut(node.0).as_ref().unwrap().clone());
                let mut neighbor = self.nodes.get_xy_mut(xy).as_ref().unwrap().borrow_mut();
                neighbor.parent = parent;
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
}
