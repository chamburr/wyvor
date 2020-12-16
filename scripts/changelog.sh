#!/bin/bash

echo "Installing..."
npm install -g conventional-changelog

echo "Generating changelog..."
conventional-changelog -p angular -i CHANGELOG.md -s -k web/package.json
