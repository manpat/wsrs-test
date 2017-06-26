#!/usr/bin/env bash

screen -D -RR server -X quit || true
screen -dmS server 
screen -S server -X stuff $'cd server; RUST_BACKTRACE=1 cargo run\n'
