#!/usr/bin/env bash

git pull
pushd client; cargo build --release; popd
pushd server; cargo build; popd
