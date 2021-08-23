
## Game Content Misc. Notes

Potential cool sounding NPC names go here

calsut-moran
clint vandenbosch
solomander

Simonos, Orthavion, Derris, and Ledforgete

height: 21, content: 33, current: 11, offset: 7, size: 13
height: 21, content: 33, current: 12, offset: 6, size: 13

Gigantic Structure to Nowhere - name for randomized dungeon?

Quest where crafter asks you to bring absurd amounts of materials to craft one pair of bracers.

## AI Design

(Avg. FPS over 1 min fast exploration 255x255 map)

Fx: 677.0
AHash: 659
Fnv: 652
Hashbrown: 641

"BIGT" - Behavior, Intention, Goal, Task

1. !Goal
2. Goal && !Task
3. Task

struct Actor {
  // Navigation related.
  xy: ICoord,
  nav: {
    prev_weight: f32,
    stationary: bool,
  },

  // A.I. related.
  behavior: Behavior,
  intention: Intention,
  goals: Vec<Goal>,
  tasks: Vec<Task>,
}

struct Behavior {
  faction: BitSet,
}

struct Intention {
  fn push_goals(&mut Vec<Goal>);
}

enum Goal {
  AvoidPlayer,
  ChasePlayer,
  Roam { radius: i32 },
  Wait,
}

impl Goal {
  finished() -> bool;
  impossible() -> bool;
  push_tasks(&mut Vec<Task>);
}

enum Task {
  Move(Direction),
  MoveTo(ICoord),
  Wait,
}

impl Task {
  execute() -> bool;
}

-----

trait Intention {
  fn push_goals(&mut Vec<Goal>);
}

struct DebugAvoidIntention;

impl Intention for DebugAvoidIntention {

}

struct DebugChaseIntention;

impl 