# Control Applications
These apps dictate controls algorithms such as altitude control.

## Resources:
- https://doc.rust-lang.org/book/ch06-03-if-let.html
- https://www.reddit.com/r/rust/comments/6tldct/state_machine_best_practices/
- https://hoverbear.org/blog/rust-state-machine-pattern/
- https://deislabs.io/posts/a-fistful-of-states/

# Notes

## State Machine
The `Controller` is a state machine that constrains the system's behavior based the current `ControlMode`. 
`ControlMode` always begins in the `Init` state.
State transitions are triggered when telemetry meets certain constraints.

* Starting the altitude control app initializes the `Controller` in the `Init` state.
* `Init --> Safe` transition happens when the `Controller` completes the initialization procedure, such as Power On Self Tests and initializing peripheral devices such as the control sensors and valves.
* `Safe` mode is the default mode of the `Controller`. Actuations of the control valves are not allowed unless specific criteria are met (satisfied safety criteria, no prior faults, control mass is available, etc.). The ballast valve and balloon valve are both `locked`, so they would deny any request to open.
* `Safe --> Idle` transition happens when the `Controller` determines it is safe to execute actuation commands. Both valves are `unlocked` after this transition.
* The control loop algorithm is executed on telemetry in this mode to determine what actuation commands should be send to maintain the set point altitude while the `Controller` is in `Idle` mode.
* `Idle --> Vent` transition happens when the control loop determines that the ascent rate must be lowered in order to maintain the set point altitude.
* In the `Vent` mode, the ballast valve is `locked` closed and the balloon's valve is free to open according to a PWM duty cycle provided by the control algorithm.
* `Vent --> Idle` transition happens when the control loop determines that the current altitude and vertical velocity are acceptable for maintaining the set point altitude within the tolerance. The ballast valve is `unlocked` again after this transition.
* `Idle --> Drop` transition happens when the control loop determines that the ascent rate must be raised in order to maintain the set point altitude.
* In the `Drop` mode, the balloon valve is `locked` closed and the ballast valve is free to open according to a PWM duty cycle provided by the control algorithm.
* `Drop --> Idle` transition happens when the control loop determines that the current altitude and vertical velocity are acceptable for maintaining the set point altitude within the tolerance. The balloon valve is `unlocked` again after this transition.
* `* --> Abort` transition happens when a fault is detected or the altitude has strayed out of bounds (like gas remaining too low or altitude too low).
* In the `Abort` mode, the balloon valve is `locked` closed and the balloon valve is `locked` open.
* `Idle --> Safe` (or `Abort --> Safe`) transition happens when the control loop has been commanded to safely terminate the control sequence.

Over a nominal flight, the control flow might look like:
```
                                        Control loop
                                          ---------
                                         |  Drop   |
                                         |    ^    |
                                         |  [Up!]  |
                                         |    v    |
Init --[POST good]--> Safe --[Start!]--> |  Idle   | --[Stop!]--> Safe
                                         |    ^    |
                                         | [Down!] |
                                         |    v    |
                                         |  Vent   |
                                          ---------
```

## Code Structure

* The `Controller` state machine is always running.
* In `Idle`, `Vent` and `Drop` modes, the control loop may use a set of gains and settings for each respective mode to tailor the control response.
