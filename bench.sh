#! /bin/bash

cargo build --release
./target/release/shitty_cli.exe compile test.s test.bin

program=$(cat test.s)

./target/release/shitty_cli.exe run -o test.s
./target/release/shitty_cli.exe exec test.bin
./target/release/shitty_cli.exe run "$program"

hyperfine --sort mean-time --warmup 5 --runs 5000 --export-markdown out.md -N \
-n 'run file' './target/release/shitty_cli.exe run -o test.s' \
-n 'run input' "./target/release/shitty_cli.exe run '$program'" \
-n 'exec file' './target/release/shitty_cli.exe exec test.bin' \

rm test.bin
