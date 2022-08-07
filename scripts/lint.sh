#!/bin/bash

echo "Linting Rust..."
cargo clippy

echo "Linting Node..."
yarn --silent lint
