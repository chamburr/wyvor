#!/bin/bash

WORKDIR=$(pwd)

echo "Linting Rust..."
cd "$WORKDIR/api" && cargo clippy

echo "Linting Node..."
cd "$WORKDIR/web" && yarn --silent lint

echo "Linting Go..."
cd "$WORKDIR/bot" && go vet .
