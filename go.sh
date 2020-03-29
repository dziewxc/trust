#!/bin/sh
cargo b --release
sudo setcap cap_net_admin=eip target/release/trust
target/release/trust &
pid=$!
sudo ip addr add 198.168.0.1/24 dev tun1
sudo ip link set dev tun1 up
wait $pid
