An Experiment with Websockets in Rust
======

This repo is a quick-n-dirty attempt at creating web application/server setup writen mostly in rust.
There is plenty of awful/shady code here, but this is mostly an experiment so w/e

Setup
-----
```
curl https://sh.rustup.rs -sSf | sh
rustup toolchain install nightly
rustup default nightly
rustup target add asmjs-unknown-emscripten
```