//! The Simulation orchestrates the passage of simulated time and the transitions of states within the system.

use crate::library::Library;
use crate::wire::Wire;
use crate::Id;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;
use threadpool::ThreadPool;

/// Default timeout for all items in a simulation step phase to complete and send their results back to the Simulation.
const DEFAULT_STEP_PHASE_TIMEOUT: Duration = Duration::from_millis(1000);

/// A simulation result.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SimResult {
    /// Simulation is continuing.
    Continuing,
    /// Simulation has completed.
    Finished,
}

/// A result for a single simulation step.
#[derive(Debug, Clone, PartialEq)]
enum StepResult {
    /// The result of a simulation step for a single Wire.
    Wire(Result<SimResult, String>, Wire),
    /// The result of a simulation step for a single Element.
    Element(Result<SimResult, String> /* TODO: , Element */),
}

/// Top level representation of a simulation and executor of the simulation steps.
#[derive(Debug)]
pub struct Simulation {
    /// Time step size.
    interval: u64,
    /// Present simulation time.
    time: u64,

    /// Thread pool for executing individual simulation step phases.
    pool: ThreadPool,
    /// Message passing FIFO sender to clone for passing results back to the Simulation.
    sender: Sender<StepResult>,
    /// Message passing FIFO receiver for the Simulation to obtain step phase results.
    receiver: Receiver<StepResult>,
    /// Maximum time to wait for all results of a step phase before raising an error.
    phase_timeout: Duration,

    /// Collection of all Wires that have been added to the Simulation.
    wires: Library<Wire>,
}

impl Simulation {
    /// Create a new, empty Simulation.
    ///
    /// # Parameters
    ///
    /// - `interval`: Time to elapse for each step of the simulation in arbitrary time units.
    ///
    /// # Example
    ///
    /// ```
    /// # use rvfs_sim_core::sim::Simulation;
    /// let sim = Simulation::new(10);
    ///
    /// assert!(sim.is_empty());
    /// ```
    pub fn new(interval: u64) -> Self {
        assert_ne!(0, interval);

        let (sender, receiver) = mpsc::channel();
        Self {
            interval,
            time: 0,

            pool: ThreadPool::default(),
            sender,
            receiver,
            phase_timeout: DEFAULT_STEP_PHASE_TIMEOUT,

            wires: Library::new(),
        }
    }

    /// Query whether a Simulation has had any components added to it.
    ///
    /// A Simulation is empty if it has no Wires, Input/OutputPins, or Elements.
    pub fn is_empty(&self) -> bool {
        self.wires.iter().count() == 0
    }

    /// Change the maximum time to wait for all results of a step phase before raising an error.
    ///
    /// # Parameters
    ///
    /// - `timeout`: New phase timeout value.
    pub fn set_phase_timeout(&mut self, timeout: Duration) {
        self.phase_timeout = timeout;
    }

    /// Add a Wire to the Simulation.
    ///
    /// The Id in the successful result allows the Wire to be looked up later.
    ///
    /// # Parameters
    ///
    /// - `wire`: The Wire instance, which will be owned by the Simulation.
    pub fn add_wire(&mut self, wire: Wire) -> Result<Id, String> {
        Ok(self.wires.add(wire))
    }

    /// Look up a Wire by ID.
    ///
    /// # Parameters
    ///
    /// - `id`: The Id of the Wire which was returned when it was [added](`Self::add_wire`).
    pub fn wire(&self, id: Id) -> Result<&Wire, String> {
        self.wires
            .inspect(id)
            .as_ref()
            .ok_or("No wire found for the given ID".to_string())
    }

    /// Run the simulation.
    ///
    /// Begin stepping the components of the simulation.  Running the simulation consumes the Simulation instance.  The
    /// simulation will run forever unless some component eventually returns a result of [SimResult::Finished].
    pub fn run(mut self) -> Result<SimResult, String> {
        let mut result = Ok(SimResult::Finished);
        if !self.is_empty() {
            loop {
                result = self.step();
                if let Ok(SimResult::Continuing) = result {
                    continue;
                } else {
                    break;
                }
            }
        }

        result
    }

    /// Advance the simulation by one time step.
    fn step(&mut self) -> Result<SimResult, String> {
        let mut result = self.step_input_pins();
        if let Ok(SimResult::Continuing) = result {
            result = self.step_elements();
            if let Ok(SimResult::Continuing) = result {
                result = self.step_wires();
            }
        }

        self.time += self.interval;

        result
    }

    /// Execute the first phase of a Simulation step by updating the [InputPins](InputPin).
    fn step_input_pins(&self) -> Result<SimResult, String> {
        // TODO: implement this
        Ok(SimResult::Continuing)
    }

    /// Execute the second phase of a Simulation step by updating the [Elements](Element).
    fn step_elements(&self) -> Result<SimResult, String> {
        // TODO: implement this
        Ok(SimResult::Continuing)
    }

    /// Receive and unwrap a step result.
    fn receive_result(&mut self) -> Result<StepResult, String> {
        // Wait for every Wire step to complete (or time out), and obtain the results.
        let execution_result = self
            .receiver
            .recv_timeout(self.phase_timeout)
            .map_err(|err| {
                (match err {
                    RecvTimeoutError::Timeout => {
                        "Timed out waiting for wire step phase to complete!"
                    }
                    RecvTimeoutError::Disconnected => {
                        "Disconnected while waiting for wire step phase to complete!"
                    }
                })
                .to_string()
            })?;

        Ok(execution_result)
    }

    /// Execute the third phase of a Simulation step by updating the [Wires](Wire).
    fn step_wires(&mut self) -> Result<SimResult, String> {
        let mut finished = false;

        for id in self.wires.iter() {
            let mut wire = self.wires.checkout(id)?;
            // "Check out" the Wire for the step execution.

            let sender = self.sender.clone();
            let interval = self.interval;
            // TODO: "Check-out" OutputPins and temporarily inject into Wire.

            // Delegate the Wire step execution to the thread pool.
            self.pool.execute(move || {
                wire.step(interval);
                let _ = sender.send(StepResult::Wire(Ok(SimResult::Continuing), wire));
            });
        }

        for id in self.wires.iter() {
            if let StepResult::Wire(op_result, wire) = self.receive_result()? {
                finished |= op_result? == SimResult::Finished;

                // Check-in the Wire and OutputPins.
                self.wires.checkin(id, wire)?;

                // TODO: Check-in OutputPins.
            }
        }

        if finished {
            Ok(SimResult::Finished)
        } else {
            Ok(SimResult::Continuing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wire::WirePull;
    use float_cmp::assert_approx_eq;

    // Tests for Simulation
    #[test]
    fn simulation_create() {
        // WHEN a simulation is created
        let sim = Simulation::new(10);
        // THEN instantiation succeeds and the new instance is empty and has the default phase timeout
        assert!(sim.is_empty());
        assert_eq!(DEFAULT_STEP_PHASE_TIMEOUT, sim.phase_timeout);
    }
    #[test]
    fn simulation_add_wire() {
        // GIVEN a simulation instance and a wire
        let wire = Wire::new(&"foo".to_string(), WirePull::None);
        let mut sim = Simulation::new(10);
        // WHEN a wire is created
        let result = sim.add_wire(wire);
        // THEN adding the wire succeeds
        assert!(result.is_ok());
    }
    #[test]
    fn simulation_run_empty() {
        // GIVEN an empty Simulation
        let sim = Simulation::new(10);
        // WHEN the simulation is run
        let result = sim.run();
        // THEN the result is success and indicates the simulation is finished
        assert_eq!(Ok(SimResult::Finished), result);
    }
    #[test]
    fn simulation_step_input_pins_empty() {
        // GIVEN an empty Simulation
        let sim = Simulation::new(10);
        // WHEN the input pins are stepped
        let result = sim.step_input_pins();
        // THEN the result is success and indicates the simulation should continue
        assert_eq!(Ok(SimResult::Continuing), result);
    }
    #[test]
    fn simulation_step_elements_empty() {
        // GIVEN an empty Simulation
        let sim = Simulation::new(10);
        // WHEN the components are stepped
        let result = sim.step_elements();
        // THEN the result is success and indicates the simulation should continue
        assert_eq!(Ok(SimResult::Continuing), result);
    }
    #[test]
    fn simulation_step_wires_empty() {
        // GIVEN an empty Simulation
        let mut sim = Simulation::new(10);
        // WHEN the wires are stepped
        let result = sim.step_wires();
        // THEN the result is success and indicates the simulation should continue
        assert_eq!(Ok(SimResult::Continuing), result);
    }
    #[test]
    fn simulation_step_empty() {
        // GIVEN an empty Simulation and a simulation interval
        let interval = 10;
        let mut sim = Simulation::new(interval);
        // WHEN the simulation is stepped
        let result = sim.step();
        // THEN the result is success and indicates the simulation should continue
        assert_eq!(Ok(SimResult::Continuing), result);
        // AND THEN the time has stepped by one interval
        assert_eq!(interval, sim.time);
    }
    #[test]
    fn simulation_step_with_wires() {
        // GIVEN a Simulation with two wires
        let wire1 = Wire::new(&"foo".to_string(), WirePull::Up);
        let wire2 = Wire::new(&"bar".to_string(), WirePull::Down);
        let mut sim = Simulation::new(10);
        let result1 = sim.add_wire(wire1);
        let result2 = sim.add_wire(wire2);
        // WHEN the wires are stepped
        let result3 = sim.step_wires();
        // THEN the wires were added, and the step result is success and indicates the simulation should continue
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert_eq!(Ok(SimResult::Continuing), result3);
    }
    #[test]
    fn simulation_lookup_wire() {
        // GIVEN a Simulation with two wires
        let wire1 = Wire::new(&"foo".to_string(), WirePull::Up);
        let name = "bar".to_string();
        let wire2 = Wire::new(&name, WirePull::Down);
        let mut sim = Simulation::new(10);
        let result1 = sim.add_wire(wire1);
        let result2 = sim.add_wire(wire2);
        // WHEN a wire is looked up in the simulation
        assert!(result2.is_ok());
        let result3 = if let Ok(id) = result2 {
            sim.wire(id)
        } else {
            Err("No wire!".to_string())
        };
        // THEN the wires were added, and the second wire was looked up correctly
        assert!(result1.is_ok());
        assert!(result2.is_ok());
        assert!(result3.is_ok());
        if let Ok(w) = result3 {
            assert_eq!(name, *w.name());
        }
    }
    #[test]
    fn simulation_step_with_wire_pulled_down() {
        // GIVEN a Simulation with a wire defaulting to pulled-up, but driven down
        let tau = 5f32;
        let mut wire = Wire::new(&"foo".to_string(), WirePull::Up);
        let mut sim = Simulation::new(10);
        wire.set_time_constant(tau);
        wire.set_pull(WirePull::Down);
        let result1 = sim.add_wire(wire);
        // WHEN the wire simulation is stepped
        let result2 = sim.step_wires();
        // THEN the wire was added, and the step result is success and indicates the simulation should continue
        assert!(result1.is_ok());
        assert_eq!(Ok(SimResult::Continuing), result2);
        // AND THEN the wire value has been updated
        if let Ok(id) = result1 {
            assert_approx_eq!(f32, 0.13533528f32, sim.wire(id).unwrap().measure().into());
        }
    }
}
