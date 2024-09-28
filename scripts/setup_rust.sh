#!/bin/bash

curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
echo 'source $HOME/.cargo/env' >> ~/.bashrc

source $HOME/.cargo/env

sudo apt install build-essential clang libsndfile1-dev -y

cargo install sqlx-cli --no-default-features --features postgres

# development tools
# cargo install cargo-watch