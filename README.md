# UWr-compatible RAM machine

This is a RAM machine that is *mostly* compatible with the one required by the *Intro to Computer Science* course at the
University of Wrocław.

## Differences

It differs from the [web wersion](https://ram.ii.uni.wroc.pl/) in two areas:

- It has a `debug` instruction which prints out everything currently in RAM.
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

## Sample program

A sample `program.ram` which checks if the 3 digits given by the user can be used to create a triangle.

```
// reading
read 1
read 2
read 3
// 1st attempt
load 1
sub 2
sub 3
add =1
jgtz fail
// 2nd attempt
load 2
sub 3
sub 1
add =1
jgtz fail
// 3rd attempt
load 3
sub 1
sub 2
add =1
jgtz fail
write =1
halt
fail: write =0
```