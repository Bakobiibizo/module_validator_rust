#!/bin/bash

apt-get update && apt-get upgrade -y

if ! command -v sudo &> /dev/null; then 
    sudo apt-get install -y sudo
    sudo apt-get install -y sudo
    sudo newgrp sudo 
    sudo usermod -aG sudo $USER
fi

sudo apt-get update && sudo apt-get install -y postgresql

sudo chmod +x scripts/setup_database.sh

sudo bash scripts/setup_database.sh

sudo chmod +x scripts/setup_rust.sh

sudo bash scripts/setup_rust.sh

source $HOME/.cargo/env

python -m venv .venv

source .venv/bin/activate

pip install --upgrade pip

pip install setuptools wheel

pip install -e .

cargo build --release

cargo run help

echo "Setup complete"

exit 0
