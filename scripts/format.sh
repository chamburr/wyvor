#!/bin/bash

echo "Formatting Rust..."
cargo fmt

echo "Formatting Node..."
yarn --silent format --loglevel warn
