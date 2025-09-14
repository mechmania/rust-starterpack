@echo off
cd /d "%~dp0\.."
set RUSTFLAGS=-Awarnings
cargo run --release -- %*
