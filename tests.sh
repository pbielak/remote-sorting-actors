#!/bin/bash

set -e

if [ ! -f "./target/release/local-sorting-actors" ]; then
    echo "Project not built!"
    cargo build --release
fi

for n in $1; do
  echo "N = $n"
  for r in $(seq 1 $2); do
    ./target/release/local-sorting-actors -i $3 -o - -n $n -k $n;
  done
done

