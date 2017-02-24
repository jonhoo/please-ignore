#!/bin/bash

echo "==> build"
cargo b --release

# we need sudo
sudo date > /dev/null

# benchmark tarpc server/client in same process with same core
echo "==> profile"
sudo perf record -c 5000 --call-graph=dwarf -g -o tarpc-same.perf.data target/release/same-proc

# make a pretty flamegraph
echo "==> make flamegraph"
sudo perf script -i tarpc-same.perf.data | stackcollapse-perf | ./unmangle.sh | flamegraph > flame.svg

echo "flamegraph: open flame.svg"
echo "perf report: sudo perf report -g --no-children -i tarpc-same.perf.data"
