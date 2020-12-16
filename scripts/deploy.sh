#!/bin/bash

source ~/.profile

WORKDIR=$(pwd)

echo "Pulling..."
git pull

echo "Building Rust..."
cd "$WORKDIR/api" && cargo build --release

echo "Building Node..."
cd "$WORKDIR/web" && yarn && yarn --silent build

echo "Building Go..."
cd "$WORKDIR/bot" && go build

echo "Restarting API..."
systemctl --user restart wyvor-api

echo "Restarting Web..."
systemctl --user restart wyvor-web

echo "Restarting Bot..."
systemctl --user restart wyvor-bot
