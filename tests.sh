#!/bin/bash

# Args: data num-actors num-chunks

set -e

if [ ! -f "./target/release/supervisor" ] && [ ! -f "./target/release/sorting_actor" ]; then
    echo "Project not built!"
    cargo build --release
fi

for n in $(seq 1 $2); do
    ./target/release/sorting_actor --supervisor-addr 127.0.0.1:8000 1>/dev/null 2>&1 &
done

./target/release/supervisor --addr 127.0.0.1:8000 --input $1 --nb-actors $2 --nb-chunks $3 --output -

