#!/bin/bash
cargo build --profile profiling
samply record ./target/profiling/sally
