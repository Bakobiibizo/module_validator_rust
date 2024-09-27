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

sudo ./scripts/setup_database.sh