#!/bin/bash
#
# Runs the Rust test suite. Separate from rust.sh, which builds only what ships: `cargo build`
# never compiles `#[cfg(test)]` code, so without this a broken test is invisible.
#
# Extra arguments go to cargo: ./rust-test.sh delete_messages
set -euo pipefail

cd "$(dirname "$0")/composeApp/src/main/rust"

# No -p: the whole workspace, so the extractor is never silently untested.
"${CARGO:-cargo}" test --workspace "$@"
