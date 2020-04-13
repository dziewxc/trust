#!/bin/bash
cargo b --release
ext=$?
echo "$ext"
if [ $ext -ne 0 ]; then
  exit $ext
fi
sudo setcap cap_net_admin=eip target/release/trust
target/release/trust &
pid=$!
sudo ip addr add 198.168.0.1/24 dev tun1
sudo ip link set dev tun1 up
trap "kill $pid" INT TERM
wait $pid