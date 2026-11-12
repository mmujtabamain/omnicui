#!/bin/sh
set -eu
cd "$(dirname "$0")"
mkdir -p build
todo_build_arch="$(uname -m)"
swiftc -j 1 -parse-as-library -swift-version 5 -target "${todo_build_arch}-apple-macos26.0" \
  ../../../apps/macos/Sources/TodoMac/*.swift Adapter.swift -o build/adapter
