# Control Applications
These apps dictate controls algorithms such as altitude control.

# Usage

## Build & Run
```shell
cargo build # compile the app
RUST_LOG=info cargo run # run the app with pretty printing [debug, info, warn]
```

## Test with simple flight model
The Altitude Control applications can be run in a software-in-the-loop
simulation that implements a basic flight model based on the 
US 1976 Standard Atmosphere for atmospheric parameters.

```shell
cd ../support_apps # navigate to the `support_apps` directory of this repo
RUST_LOG=info cargo test --test sim_tests -- test_closed_loop # run the simulator and output to out.csv
```
The simulation result can be plotted using `utils/plot_control_sim_output.py`

# Notes

## State Machine
The `Controller` is a state machine that constrains the system's behavior based the current `ControlMode`.
State transitions are triggered when telemetry meets certain constraints.

```
 ┌─┐
 └┼┘
  │
  ▼
┌──────────────────────────────┐
│ Init                         │
├──────────────────────────────┤
│ Initialize command handler   │
│ Initialize telemetry monitor │
│ Power On Self Test           │
└─┬────────────┬───────────────┘
  │POST        │POST
  │error       │good
  │            ▼
  │    ┌─────────────────────────────────────────┐
  │    │ Ready                                   │
  │    ├─────────────────────────────────────────┤
  │    │ Wait to be above lower alitude limit    │
  │    │                  AND                    │
  │    │ Wait to be close to the target altitude │
  │    └───────┬─────────────────────────────────┘
  │            │
  │            │
  │            ▼
  │    ┌─────────────────────────────────────────────────────────────────────────────────┐
  │    │ Stabilize                                                                       │
  │    ├─────────────────────────────────────────────────────────────────────────────────┤
  │    │ Loop checking speed and altitude, then update state                             │
  │    │ Speed and altitude deadzones limit actuation                                    │
  │    ├─────────────────────────────────────────────────────────────────────────────────┤
  │    │                                                                                 │
  │    │                                                                                 │
  │    │                                                                                 │
  │    │                          ┌──────────────────────────┐                           │
  │    │      ┌──────────────────►│ Idle                     │◄───────────────────┐      │
  │    │      │                   ├──────────────────────────┤                    │      │
  │    │      │                   │ Within altitude deadzone │                    │      │
  │    │      │                   │ Within speed deadzone    │                    │      │
  │    │      │                   └─────────────┬────────────┘                    │      │
  │    │      │                                 │                                 │      │
  │    │      │                                 │Outside deadzone                 │      │
  │    │      │                                 │for speed OR altitude            │      │
  │    │      │                                 │                                 │      │
  │    │      │                                 ▼                                 │      │
  │    │      │                                ┌┬┐                                │      │
  │    │      │                                └┼┘                                │      │
  │    │      │              Altitude too high  │  Altitude too low               │      │
  │    │      │            ┌────────────────────┴─────────────────────┐           │      │
  │    │      │            ▼                                          ▼           │      │
  │    │      │           ┌┬┐                                        ┌┬┐          │      │
  │    │      │           └┼┘                                        └┼┘          │      │
  │    │      │ Descending │                                          │ Ascending │      │
  │    │      └────────────┤                                          ├───────────┘      │
  │    │      ▲            │                                          │           ▲      │
  │    │      │            │Ascending                       Descending│           │      │
  │    │      │            ▼                                          ▼           │      │
  │    │      │       ┌──────────────────┐                   ┌─────────────────┐  │      │
  │    │      │       │ Vent             │                   │ Dump            │  │      │
  │    │      │       ├──────────────────┤                   ├─────────────────┤  │      │
  │    │      │       │ Release lift gas │                   │ Release ballast │  │      │
  │    │      │       └────────┬─────────┘                   └────────┬────────┘  │      │
  │    │      │                │                                      │           │      │
  │    │      └────────────────┘                                      └───────────┘      │
  │    │                                                                                 │
  │    └─┬───────┬──────────────────────────────────────────────┬────────────────────────┘
  │POST  │Out of │Software                                      │Out of
  │error │gas    │problem                                       │ballast
  │      ▼       ▼                                              ▼
 ┌┴─────────────────┐              Out of ballast              ┌──────────────────────────┐
 │ Abort            ├─────────────────────────────────────────►│ Safe                     │
 ├──────────────────┤                                          ├──────────────────────────┤
 │ Dump all ballast │                                          │ Vent valve locked closed │
 └──────────────────┘                                          │ Dump valve locked closed │
                                                               └──────────────────────────┘
```

## Code Structure

* The `Controller` state machine is always running.
* In `Idle`, `Vent` and `Drop` modes, the control loop may use a set of gains and settings for each respective mode to tailor the control response.

## Resources:
- https://doc.rust-lang.org/book/ch06-03-if-let.html
- https://www.reddit.com/r/rust/comments/6tldct/state_machine_best_practices/
- https://hoverbear.org/blog/rust-state-machine-pattern/
- https://deislabs.io/posts/a-fistful-of-states/