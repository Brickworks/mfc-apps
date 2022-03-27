# mfc-apps
MFC applications

## Run a simulated flight
### Configure the sim
Set the simulation configuration in `support_apps/config/sim_config.toml`

### Start the sim
Navigate to the `cli` directory to run the MFC Command Line Interface.
Then use the `sim start` command to start a flight simulation. Use the `RUST_LOG` env var to specify the log level to report.
```sh
cd cli
RUST_LOG=info cargo run -- sim start
```

### View the flight data
First install the Brickworks support tooling, `firebrick`.
```sh
git clone git@github.com:Brickworks/firebrick.git
cd firebrick
pip install .

firebrick --help
```
Then start up a telemetry dashboard.
```sh
firebrick dashboard -t $PATH_TO_TELEMETRY_CSV
```
Then navigate to the server address specified in the log output.