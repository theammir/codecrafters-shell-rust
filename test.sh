#!/bin/sh
#
# Run the integration test suite.
#
#   ./test.sh          all stages
#   ./test.sh ez5      one step, by slug
#   ./test.sh base     one stage group
#
# Uses cargo-nextest when installed (process-per-test isolation, which the PTY
# tests prefer) and falls back to cargo test otherwise.

set -e

cd "$(dirname "$0")"

FILTER="$1"

if cargo nextest --version >/dev/null 2>&1; then
  if [ -n "$FILTER" ]; then
    exec cargo nextest run --tests -E "test(~$FILTER)"
  fi
  exec cargo nextest run --tests
fi

# cargo test takes a substring filter positionally. `--no-fail-fast` matters here:
# without it cargo stops at the first stage file that fails, hiding the progress
# of every later stage.
if [ -n "$FILTER" ]; then
  exec cargo test --tests --no-fail-fast -- "$FILTER"
fi
exec cargo test --tests --no-fail-fast
