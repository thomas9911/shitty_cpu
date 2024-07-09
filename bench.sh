#! /bin/bash

file="./scripts/factorial.s"
cargo build --release
./target/release/shitty_cli compile $file test.bin

echo "$file"
program=$(cat $file)

./target/release/shitty_cli run -o $file
./target/release/shitty_cli exec test.bin
./target/release/shitty_cli run "$program"

hyperfine --sort mean-time --warmup 5 --runs 5000 --export-markdown out.md -N \
  -n 'run file' "./target/release/shitty_cli run -o $file" \
  -n 'run input' "./target/release/shitty_cli run '$program'" \
  -n 'exec file' './target/release/shitty_cli exec test.bin' \

rm test.bin
