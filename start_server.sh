#!/usr/bin/env bash

cd server > /dev/null

for session in $(screen -ls | grep -o '[0-9]*\.server'); do
	screen -S "${session}" -X quit;
done

screen -dmS "server" bash -i
sleep 0.1 # shutup
screen -r -S "server" -p 0 -X stuff $'RUST_BACKTRACE=1 cargo run\r'
