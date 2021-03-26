//-------------------------------------------------------------------------------------------------
// STD includes.
//-------------------------------------------------------------------------------------------------
use std::time::Duration;

//-------------------------------------------------------------------------------------------------
// Timer provides an easy way to track passing time intervals.
//-------------------------------------------------------------------------------------------------
#[derive(Clone, Copy, Debug)]
pub struct Timer {
    // Interval duration tracked by the timer.
    interval: Duration,
    // Passed time since the last interval was reached.
    passed: Duration,
}

impl Timer {
    //---------------------------------------------------------------------------------------------
    // Creates a new timer for a given interval.
    //---------------------------------------------------------------------------------------------
    pub const fn new(interval: Duration) -> Self {
        Self { interval, passed: Duration::from_secs(0) }
    }

    //---------------------------------------------------------------------------------------------
    // Updates the timer with delta time, returning whether the interval has passed.
    //---------------------------------------------------------------------------------------------
    pub fn update(&mut self, delta: Duration) -> bool {
        self.passed += delta;

        // If the passed time is greater than the interval, return true and reset the time.
        // Otherwise, return false.
        if self.passed < self.interval {
            false
        } else {
            self.passed -= self.interval;
            true
        }
    }

    //---------------------------------------------------------------------------------------------
    // Updates the timer with delta time, but does not check if the interval has been met.
    //---------------------------------------------------------------------------------------------
    pub fn update_without_consuming(&mut self, delta: Duration) {
        self.passed += delta;
    }
}
