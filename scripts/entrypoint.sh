#!/usr/bin/env bash
set -e

# If called with development tools or shells, execute directly
case "$1" in
    bash|sh|cargo|rustc|git|spacetime)
        exec "$@"
        ;;
esac

# Incrementally build magi binary inside container so changes are always compiled
cargo build --bin magi --quiet

# Otherwise execute magi CLI with arguments
exec /workspace/target/debug/magi "$@"
