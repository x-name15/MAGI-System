#!/usr/bin/env bash
set -e
# Configure git to match Windows CRLF handling and permit workspace
git config --global --add safe.directory /workspace 2>/dev/null || true
git config --global core.autocrlf true 2>/dev/null || true
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
