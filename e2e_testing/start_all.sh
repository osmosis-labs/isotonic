#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

SCRIPT_DIR="$(realpath "$(dirname "$0")")"

# start up the blockchain and wait for it
"$SCRIPT_DIR/scripts/start.sh" &

# uploads and initialize isotonic contracts
"$SCRIPT_DIR/scripts/init.sh" &

# after tests are done, shut down the setup
"$SCRIPT_DIR/scripts/stop.sh" &
