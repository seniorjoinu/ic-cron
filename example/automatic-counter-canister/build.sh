#!/usr/bin/env bash

cargo build --target wasm32-unknown-unknown --release --package ic-cron-example && \
 ic-cdk-optimizer ./target/wasm32-unknown-unknown/release/ic_cron_example.wasm -o ./target/wasm32-unknown-unknown/release/ic-cron-example-opt.wasm