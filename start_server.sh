#!/usr/bin/env bash

cd server > /dev/null

for session in $(screen -ls | grep -o '[0-9]*\.server'); do
	screen -S "${session}" -X quit;
done

screen -dmS "server" bash -c "RUST_BACKTRACE=1 cargo run ; exec bash"
