//-------------------------------------------------------------------------------------------------
// Alias for convenience.
//-------------------------------------------------------------------------------------------------
pub type Behaviors = Vec<Box<dyn Behavior + Send + Sync>>;

//-------------------------------------------------------------------------------------------------
// Behavior describe how an actor should interact with different stimuli.
//-------------------------------------------------------------------------------------------------
pub trait Behavior {}

//-------------------------------------------------------------------------------------------------
// A generic behavior implementation.
//-------------------------------------------------------------------------------------------------
pub struct BasicBehavior;

impl Behavior for BasicBehavior {}