#!/usr/bin/env bash

screen -D -RR test1 -X quit || true
screen -dmS server 
screen -S server -X stuff $'cd server; RUST_BACKTRACE=1 cargo run\n'
