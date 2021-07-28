
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

// Massive variant of all possible action types?
// At the point of return, the action is ensured to be valid/doable.
// Eventually have the variant also include optional references to Animations?
enum Action {
  // e.g. 
  // Move(UCoord)
  // Attack(UCoord)
  // Shoot(UCoord)
  // Cast(UCoord, &Spell)
  // Use(&Item)
  // Say(&String)
  // Interact(UCoord)
  ...
}

// Enum of categories of actions that can be taken in combat.
// Used to describe unique priorities for particular AIs.
enum CombatType {
  Melee,
  Ranged,
  Magic,
  Defense,
  // ...
}

// Variant of types of goals.
enum GoalType {
  Kill,
  Hail,
  Visit,
  Flee,
  Follow,
  Interact,
  // ...
}

// Variant of possible targets for goals.
enum GoalTarget {
  Coord(UCoord),
  Entity(Entity),
  // ...
}

struct Goal {
  type: GoalType,
  target: GoalTarget,

  // If the goal has been accomplished and should be removed from the stack.
  accomplished() -> bool;

  // If the goal is impossible (i.e. goal was to kill an NPC that is now dead).
  impossible() -> bool;
}

struct AI {
  // Index in the goals stack of the last "root" goal.
  intent: usize,
  // Stack of goals in order of priority.
  goals: vec<Goal>,
  // Enum of combat types in order of priority.
  combat_preference: vec<CombatType>,
}