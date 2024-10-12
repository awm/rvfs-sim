//! Wires propagate signals from OutputPin instances to InputPin instances.

use crate::wirevalue::WireValue;

/// Types of pull which may be exerted on a Wire.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum WirePull {
    /// Wire value is pulled towards 1.0.
    Up,
    /// Wire value is pulled towards 0.0.
    Down,
    /// Wire value is floating with no pull towards a specific value.
    None,
}

/// A connection between OutputPin and InputPin instances.
///
/// A Wire may have a default pull direction, which is the logic state that it wants to "naturally" settle into if it is
/// not being driven by an OutputPin.  Only one OutputPin may drive a Wire at a time.  A Wire takes time to transition
/// from one state to another, as determined by its time constant.
#[derive(Debug, Clone, PartialEq)]
pub struct Wire {
    /// A readable, unique name for the Wire within the Simulation.
    name: String,

    /// Default pull that the Wire feels when the active pull is None.
    default_pull: WirePull,
    /// Active pull that the Wire feels at the present time.
    pull: WirePull,
    /// Time constant which determines how quickly the Wire approaches its final value.
    tau: f32,
    /// Present value of the Wire.
    value: WireValue,
}

impl Wire {
    /// Create a new Wire.
    ///
    /// # Parameters
    ///
    /// - `name`: A human-readable name to assign to the Wire.
    /// - `default_pull`: The default pull behaviour of the Wire in the absence of any explicit driver.
    ///
    /// # Example
    ///
    /// ```
    /// # use rvfs_sim_core::wire::{Wire, WirePull};
    /// let wire = Wire::new("/RESET", WirePull::Up);
    ///
    /// assert_eq!("/RESET", wire.name());
    /// assert_eq!(WirePull::Up, wire.pull());
    /// ```
    pub fn new(name: &str, default_pull: WirePull) -> Self {
        let value = match default_pull {
            WirePull::Up => WireValue::new(1.0),
            WirePull::Down => WireValue::new(0.0),
            WirePull::None => WireValue::new(0.5),
        };

        Self {
            name: name.to_string(),

            default_pull,
            pull: WirePull::None,
            tau: 0.0f32,
            value,
        }
    }

    /// Get the name assigned to the Wire.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Determine the present pull direction of the Wire.
    ///
    /// The active pull direction will take precedence over the default pull value.
    pub fn pull(&self) -> WirePull {
        if self.pull == WirePull::None {
            self.default_pull
        } else {
            self.pull
        }
    }

    /// Measure the present level of the Wire.
    ///
    /// # Example
    ///
    /// ```
    /// # use rvfs_sim_core::wire::{Wire, WirePull};
    /// let wire = Wire::new("/RESET", WirePull::Down);
    ///
    /// assert_eq!(0.0, wire.measure().into());
    /// ```
    pub fn measure(&self) -> WireValue {
        self.value
    }

    /// Set the time constant which controls the rate at which the Wire's value moves in the pulled direction.
    ///
    /// # Parameters
    ///
    /// - `tau`: Time constant.  This value will be clamped to the range [0, +∞).
    pub fn set_time_constant(&mut self, tau: f32) {
        self.tau = tau.clamp(0.0, f32::INFINITY);
    }

    /// Set the active pull direction of the Wire.
    ///
    /// # Parameters
    ///
    /// - `pull`: New active pull direction of the Wire.
    pub fn set_pull(&mut self, pull: WirePull) {
        self.pull = pull;
    }

    /// Calculate the new value of the wire, based on the present value, pull direction, and time constant.
    ///
    /// # Parameters
    ///
    /// - `delta_t`: Simulation time elapsed since the last step.
    pub fn step(&mut self, delta_t: u64) {
        let pull = self.pull();

        if pull != WirePull::None {
            let newval = f32::from(self.value) * (-(delta_t as f32) / self.tau).exp();
            if pull == WirePull::Up {
                self.value = (1.0f32 - newval).into();
            } else {
                self.value = newval.into();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use float_cmp::assert_approx_eq;

    #[test]
    fn wire_create() {
        // GIVEN a wire name
        let name = "foo";
        // WHEN a new wire is created
        let wire = Wire::new(name, WirePull::None);
        // THEN the creation succeeds, the name is set, the pull is set, and the time constant is 0
        assert_eq!(name, wire.name());
        assert_eq!(WirePull::None, wire.pull());
        assert_approx_eq!(f32, 0.0, wire.tau);
    }
    #[test]
    fn wire_default_measurement_no_pull() {
        // GIVEN a wire name
        let name = "foo";
        // WHEN a new wire is created with no pull
        let wire = Wire::new(name, WirePull::None);
        // THEN the default wire value is 0.5
        assert_eq!(WireValue::new(0.5), wire.measure());
    }
    #[test]
    fn wire_default_measurement_pull_up() {
        // GIVEN a wire name
        let name = "foo";
        // WHEN a new wire is created with pull-up
        let wire = Wire::new(name, WirePull::Up);
        // THEN the default wire value is 1.0
        assert_eq!(WireValue::new(1.0), wire.measure());
    }
    #[test]
    fn wire_default_measurement_pull_down() {
        // GIVEN a wire name
        let name = "foo";
        // WHEN a new wire is created with pull-up
        let wire = Wire::new(name, WirePull::Down);
        // THEN the default wire value is 0.0
        assert_eq!(WireValue::new(0.0), wire.measure());
    }
    #[test]
    fn wire_set_time_constant() {
        // GIVEN a new wire and a time constant
        let tau = 5f32;
        let mut wire = Wire::new("foo", WirePull::None);
        // WHEN the time constant is set on the wire
        wire.set_time_constant(tau);
        // THEN the time constant has been set as expected
        assert_approx_eq!(f32, tau, wire.tau);
    }
    #[test]
    fn wire_set_negative_time_constant() {
        // GIVEN a new wire and a negative time constant
        let tau = -5f32;
        let mut wire = Wire::new("foo", WirePull::None);
        // WHEN the time constant is set on the wire
        wire.set_time_constant(tau);
        // THEN the time constant has been clamped to 0
        assert_approx_eq!(f32, 0.0, wire.tau);
    }
    #[test]
    fn wire_step_pull_up() {
        // GIVEN an initialized wire with a set time constant and pull-up
        let tau = 5f32;
        let mut wire = Wire::new("foo", WirePull::None);
        wire.set_time_constant(tau);
        wire.set_pull(WirePull::Up);
        // WHEN step is called
        wire.step(10);
        // THEN the value has changed in the pull-up direction
        assert_approx_eq!(f32, 0.93233235f32, wire.measure().into());
    }
    #[test]
    fn wire_step_pull_down() {
        // GIVEN an initialized wire with a set time constant and pull-down
        let tau = 5f32;
        let mut wire = Wire::new("foo", WirePull::None);
        wire.set_time_constant(tau);
        wire.set_pull(WirePull::Down);
        // WHEN step is called
        wire.step(10);
        // THEN the value has changed in the pull-down direction
        assert_approx_eq!(f32, 0.06766764f32, wire.measure().into());
    }
    #[test]
    fn wire_step_no_pull() {
        // GIVEN an initialized wire with a set time constant and no pull
        let tau = 5f32;
        let mut wire = Wire::new("foo", WirePull::None);
        wire.set_time_constant(tau);
        wire.set_pull(WirePull::None);
        // WHEN step is called
        wire.step(10);
        // THEN the value has not changed from the default
        assert_approx_eq!(f32, 0.5, wire.measure().into());
    }
    #[test]
    fn wire_step_explicit_pull_overrides_default() {
        // GIVEN an initialized wire with a set time constant and default pull-up, but explicit pull down
        let tau = 5f32;
        let mut wire = Wire::new("foo", WirePull::Up);
        wire.set_time_constant(tau);
        wire.set_pull(WirePull::Down);
        // WHEN step is called
        wire.step(10);
        // THEN the value has changed in the pull-down direction
        assert_approx_eq!(f32, 0.13533528f32, wire.measure().into());
    }
    #[test]
    fn wire_zero_tau_with_pull_up() {
        // GIVEN an initialized wire with a tau of zero and explicit pull-up
        let tau = 0.0f32;
        let mut wire = Wire::new("foo", WirePull::None);
        wire.set_time_constant(tau);
        wire.set_pull(WirePull::Up);
        // WHEN step is called
        wire.step(10);
        // THEN the value is immediately at maximum
        assert_approx_eq!(f32, 1.0f32, wire.measure().into());
    }
    #[test]
    fn wire_zero_tau_with_pull_down() {
        // GIVEN an initialized wire with a tau of zero and explicit pull-down
        let tau = 0.0f32;
        let mut wire = Wire::new("foo", WirePull::None);
        wire.set_time_constant(tau);
        wire.set_pull(WirePull::Down);
        // WHEN step is called
        wire.step(10);
        // THEN the value is immediately at minimum
        assert_approx_eq!(f32, 0.0f32, wire.measure().into());
    }
}
