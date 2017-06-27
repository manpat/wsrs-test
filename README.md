An Experiment with Websockets in Rust
======

This repo is a quick-n-dirty attempt at creating web application/server setup writen mostly in rust.
There is plenty of awful/shady code here, but this is mostly an experiment so w/e

Setup
-----
You need emscripten installed so follow [this shit](https://kripken.github.io/emscripten-site/docs/getting_started/downloads.html)

```
sudo apt install build-essential python

curl https://sh.rustup.rs -sSf | sh
rustup toolchain install nightly
rustup default nightly
rustup target add asmjs-unknown-emscripten
```

ws/client should target emscripten automatically because of it's .cargo/config