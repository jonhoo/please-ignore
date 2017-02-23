#!/bin/bash
cargo b --release

# we need sudo
sudo date > /dev/null

# benchmark memcached server
sudo perf record --call-graph=dwarf -g -o memcached-server.perf.data -- memcached -p 2222 -t 1 -u $USER &
pid=$!
sleep .5 # wait for sudo to spawn other process
pid=$(ps --ppid $pid -o pid=) # through sudo
sleep 1 # let the server start
target/release/memcached
sudo kill -SIGINT $pid
sleep 1 # let the server quit

# benchmark memcached client
memcached -p 2222 -t 1 -u $USER &
pid=$!
sleep 1 # let the server start
sudo perf record --call-graph=dwarf -g -o memcached-client.perf.data target/release/memcached
kill -SIGINT $pid
sleep 1 # let the server quit

# benchmark tarpc server
sudo perf record --call-graph=dwarf -g -o tarpc-server.perf.data target/release/server &
pid=$!
sleep .5 # wait for sudo to spawn other process
pid=$(ps --ppid $pid -o pid=) # through sudo
sleep 1 # let the server start
target/release/client
sudo kill -SIGINT $pid
sleep 1 # let the server quit

# benchmark tarpc client
target/release/server &
pid=$!
sleep 1 # let the server start
sudo perf record --call-graph=dwarf -g -o tarpc-client.perf.data target/release/client
kill -SIGINT $pid
sleep 1

# benchmark tarpc server/client in same process with same core
sudo perf record --call-graph=dwarf -g -o tarpc-same.perf.data target/release/same-proc

sudo perf report -g -i memcached-server.perf.data
sudo perf report -g -i memcached-client.perf.data
sudo perf report -g -i tarpc-server.perf.data
sudo perf report -g -i tarpc-client.perf.data
sudo perf report -g -i tarpc-same.perf.data
