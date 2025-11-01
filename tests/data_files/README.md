# How to use the unit tests
## Prerequisites
- Place your GameMaker data files in this directory (`/tests/data_files/`).
  - Supported extensions: `.win`, `.unx`, `.ios` and `.droid`.
  - Files with other extensions will be ignored.
- Navigate to the crate root directory in your console (`cd /path/to/libgm`).

## Basic Testing
- Run `cargo test` to run all tests.
- Run `cargo test --test my_test_name` to run a specific test. 
- Run `cargo test -- --nocapture` to see logs.
- Use the `--release` flag for faster runtime, if needed.
- You can combine these options:
  - `cargo test --release --test my_test_name -- --nocapture`
