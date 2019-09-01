#!/bin/bash

sudo apt-get update && sudo apt-get install -y gcc
curl https://sh.rustup.rs -o rustup.sh && sh rustup.sh -y
rm rustup.sh
