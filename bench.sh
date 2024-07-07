#! /bin/bash

./target/release/shitty_cli.exe compile test.s test.bin

hyperfine --warmup 5 --runs 5000 -N './target/release/shitty_cli.exe run test.s' './target/release/shitty_cli.exe exec test.bin'
