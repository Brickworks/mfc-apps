# Command-Line Interface to the Main Flight Computer
See what commands are available:
```shell
# use the RUST_LOG env var to set the lowest logging level:
#   trace, debug, info, warn, error (default)
# enter cargo commands, arguments, and options before the "--"
# enter MFC CLI commands, arguments, and options after the "--"
RUST_LOG=info cargo run -- --help
```

Get the status of the computer running this crate:
```shell
RUST_LOG=debug cargo run -- status
```
