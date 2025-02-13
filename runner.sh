#!/bin/sh

set -e
(
  cd "$(dirname "$0")"
  cargo build --release --target-dir=/tmp/codecrafters-build-git-rust --manifest-path Cargo.toml
)

exec /tmp/codecrafters-build-git-rust/release/codecrafters-git "$@"
