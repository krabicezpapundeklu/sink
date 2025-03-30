#!/bin/bash
rm -rf dist
mkdir dist

cargo build --release --target=x86_64-unknown-linux-musl
cargo build --release --target=x86_64-pc-windows-gnu

cp target/x86_64-unknown-linux-musl/release/sink dist/
cp target/x86_64-pc-windows-gnu/release/sink.exe dist/

ls -l dist/
