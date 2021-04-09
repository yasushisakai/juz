#!/bin/bash

tmux kill-server

sleep 1

for i in 1 2 3
do
	PORT="$((8880+i))"
	echo "starting $PORT"
	eval "PID=$(lsof -t -i:$PORT)"
	eval "kill -9 $PID"
	eval "tmux new-session -d -s juz$i 'cargo run --bin server -- $PORT http://localhost:$PORT'"
	sleep 1
done
