//! OutputPins drive the values calculated by Elements onto Wires.

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum OutputPinState {
    Low,
    High,
    HighImpedance,
}

/// An interface between Element and Wire instances.
///
/// An OutputPin has a delay time representing the time it takes for a new value to be calculated and propagated to the
/// attached Wire.
pub struct OutputPin {
    /// A readable name for the pin.
    name: String,

    /// State that will become active once the remaining (simulation) propagation time elapses.
    propagating_state: OutputPinState,
    /// Active pin state.
    state: OutputPinState,

    /// Propagation delay for this pin.
    delay: u64,
    /// Remaining time until the propagating state becomes active.
    remaining_propagation: u64,
}

impl OutputPin {
    /// Create a new OutputPin.
    ///
    /// # Parameters
    ///
    /// - `name`: A human-readable name to assign to the pin.
    /// - `delay`: The propagation delay to assign to the pin.
    /// - `state`: The initial output state of the pin.
    ///
    /// # Example
    ///
    /// ```
    /// # use rvfs_sim_core::opin::{OutputPin, OutputPinState};
    /// let pin = OutputPin::new("/INT", 2, OutputPinState::High);
    ///
    /// assert_eq!("/INT", pin.name());
    /// assert_eq!(2, pin.delay());
    /// assert_eq!(OutputPinState::High, pin.state());
    /// ```
    pub fn new(name: &str, delay: u64, state: OutputPinState) -> Self {
        Self {
            name: name.to_string(),

            propagating_state: OutputPinState::HighImpedance,
            state,

            delay,
            remaining_propagation: u64::MAX,
        }
    }

    /// Obtain the pin name.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Retrieve the propagation delay of the pin.
    pub fn delay(&self) -> u64 {
        self.delay
    }

    /// Obtain the active drive state of the pin.
    ///
    /// This is what will influence the level of any attached Wire.
    pub fn state(&self) -> OutputPinState {
        self.state
    }

    /// Set the state that will propagate through the pin.
    ///
    /// This will become the active state after the associated delay.
    ///
    /// # Parameters
    ///
    /// - `state`: New state to propagate through the pin.
    pub fn set(&mut self, state: OutputPinState) {
        self.propagating_state = state;
        self.remaining_propagation = self.delay;
    }

    /// Update the output state based on the inexorable advance of time.
    ///
    /// # Parameters
    ///
    /// - `delta_t`: The simulation time elapsed since the last step.
    ///
    /// # Example
    ///
    /// ```
    /// # use rvfs_sim_core::opin::{OutputPin, OutputPinState};
    /// let mut pin = OutputPin::new("/INT", 5, OutputPinState::High);
    ///
    /// assert_eq!(OutputPinState::High, pin.state());
    ///
    /// pin.step(4);
    /// pin.set(OutputPinState::Low);
    ///
    /// assert_eq!(OutputPinState::High, pin.state());
    ///
    /// pin.step(4);
    ///
    /// assert_eq!(OutputPinState::High, pin.state());
    ///
    /// pin.step(4);
    ///
    /// assert_eq!(OutputPinState::Low, pin.state());
    /// ```
    pub fn step(&mut self, delta_t: u64) {
        if delta_t >= self.remaining_propagation {
            self.remaining_propagation = 0;
            self.state = self.propagating_state;
        } else {
            self.remaining_propagation -= delta_t;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_pin_create() {
        // GIVEN a name, output delay and initial state
        let name = "foo";
        let delay = 5u64;
        let state = OutputPinState::HighImpedance;
        // WHEN a new OutputPin is created
        let pin = OutputPin::new(name, delay, state);
        // THEN it has the specified name, delay and state set
        assert_eq!(name, pin.name());
        assert_eq!(delay, pin.delay());
        assert_eq!(state, pin.state());
    }
    #[test]
    fn output_pin_set_next_state_with_zero_delay_and_no_step() {
        // GIVEN a pin with initial state and no delay
        let state = OutputPinState::HighImpedance;
        let mut pin = OutputPin::new("foo", 0, state);
        // WHEN a new state is set
        pin.set(OutputPinState::Low);
        // THEN the state remains at the initial value
        assert_eq!(state, pin.state());
    }
    #[test]
    fn output_pin_set_next_state_with_zero_delay_and_step() {
        // GIVEN a pin with initial state and no delay
        let mut pin = OutputPin::new("foo", 0, OutputPinState::HighImpedance);
        // WHEN a new state is set and the pin is stepped
        let state = OutputPinState::Low;
        pin.set(state);
        pin.step(10);
        // THEN the state becomes the new value
        assert_eq!(state, pin.state());
    }
    #[test]
    fn output_pin_set_next_state_with_delay_and_small_step() {
        // GIVEN a pin with initial state and delay
        let state = OutputPinState::HighImpedance;
        let mut pin = OutputPin::new("foo", 10, state);
        // WHEN a new state is set and the pin is stepped an amount smaller than the delay
        pin.set(OutputPinState::Low);
        pin.step(2);
        // THEN the state remains the initial value
        assert_eq!(state, pin.state());
    }
    #[test]
    fn output_pin_set_next_state_with_delay_and_large_step() {
        // GIVEN a pin with initial state and delay
        let mut pin = OutputPin::new("foo", 10, OutputPinState::HighImpedance);
        // WHEN a new state is set and the pin is stepped by more than the delay
        let state = OutputPinState::Low;
        pin.set(state);
        pin.step(20);
        // THEN the state becomes the new value
        assert_eq!(state, pin.state());
    }
    #[test]
    fn output_pin_set_next_state_with_delay_and_multiple_small_steps() {
        // GIVEN a pin with initial state and delay
        let mut pin = OutputPin::new("foo", 10, OutputPinState::HighImpedance);
        // WHEN a new state is set and the pin is stepped multiple times to pass the delay threshold
        let state = OutputPinState::Low;
        pin.set(state);
        pin.step(4);
        pin.step(4);
        // THEN the state remains the original value until the threshold is passed
        assert_eq!(OutputPinState::HighImpedance, pin.state());
        pin.step(4);
        // AND THEN the state becomes the new value
        assert_eq!(state, pin.state());
    }
}
