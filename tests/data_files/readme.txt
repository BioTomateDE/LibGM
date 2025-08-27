==================== How to use the unit tests ====================
- Fill this directory "/tests/data_files/" with the gamemaker data
  files (aka. "data.win" or "game.unx" files) you want to test.
  > Accepted file extensions for data files are: win, unx, ios, droid.
- Navigate to the crate root directory in your console.
- Run `cargo test` to run all tests or `cargo test --test my_test_name` to run a specific test.
  > You can use `cargo test -- --nocapture` to see logs.
  > You can use `cargo test --release .......` for faster runtime, if needed.
