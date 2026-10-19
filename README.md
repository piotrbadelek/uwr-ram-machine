# UWr-compatible RAM machine

This is a RAM machine that is *mostly* compatible with the one required by the *Intro to Computer Science* course at the
University of Wrocław.

## Differences

It differs from the [web wersion](https://ram.ii.uni.wroc.pl/) in two areas:

- It has `debug` instruction which prints out everything currently in RAM.
- It allows reading unallocated memory and the accumulator (located at position 0).

So, every program compatible with the original web version should work in this implementation, but not every program
compatible with this implementation will work in the original web version.

Additionally, this implementation is constrained to 1 KB of RAM (256 addresses of 32-bit integers).

## Motivation
The web version is very annoying to use for writing long programs, that's all.

## Compile & run
```sh
rustc src/main.rs
./main ./your_program.ram
```