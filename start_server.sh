#!/usr/bin/env bash

cd server > /dev/null

for session in $(screen -ls | grep -o '[0-9]*\.server'); do
	screen -S "${session}" -X quit;
done

screen -dmS "server"
sleep 0.5 # this is the best I could do, whatever man
screen -r -S "server" -X screen bash -ic "RUST_BACKTRACE=1 cargo run"
