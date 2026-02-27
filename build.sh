#!/bin/bash
set -e
cargo build --release --features serve
cp target/release/md ./md
echo "Done: ./md"
