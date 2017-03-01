#!/bin/bash

echo "==> build"
cargo b --release

# we need sudo
sudo date > /dev/null

# benchmark
echo "==> profile"
sudo perf record -c 20000 --call-graph=dwarf -g -o tokio.perf.data target/release/tokio
sudo perf record -c 20000 --call-graph=dwarf -g -o tokio-proto.perf.data target/release/tokio-proto
sudo perf record -c 20000 --call-graph=dwarf -g -o tarpc.perf.data target/release/tarpc

# make pretty flamegraphs
echo "==> make flamegraphs"
sudo perf script -i tokio.perf.data | stackcollapse-perf | ./unmangle.sh | flamegraph > tokio.svg
sudo perf script -i tokio-proto.perf.data | stackcollapse-perf | ./unmangle.sh | flamegraph > tokio-proto.svg
sudo perf script -i tarpc.perf.data | stackcollapse-perf | ./unmangle.sh | flamegraph > tarpc.svg

echo "flamegraph: open [tokio|tokio-proto|tarpc].svg"
echo "perf report: sudo perf report -g --no-children -i [tokio|tokio-proto|tarpc].perf.data"
