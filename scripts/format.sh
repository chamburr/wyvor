#!/bin/bash

WORKDIR=$(pwd)

echo "Formatting Rust..."
cd "$WORKDIR/api" && cargo fmt

echo "Formatting Node..."
cd "$WORKDIR/web" && yarn --silent format --loglevel warn

echo "Formatting Go..."
cd "$WORKDIR/bot" && go fmt
