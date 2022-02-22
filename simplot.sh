RUST_LOG=info cargo test --manifest-path support_apps/Cargo.toml \
    --package mfc --test sim_tests -- test_closed_loop --exact --nocapture
python utils/plot_control_sim_output.py
