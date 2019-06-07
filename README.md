Branch|Build
---|---
master|![Build Status](https://travis-ci.org/swiftgist/diskspace.svg?branch=master)

# DiskSpace
The program `ds` returns the 20 largest directories and files from the current directory.  The output is human friendly with appropriate units.  For example, the output of the src directory is

```
   14M ./target/debug
   14M ./target/debug/deps
    7M ./target/debug/deps/ds-6f1846660d97a074
    7M ./target/debug/ds-6f1846660d97a074
    4M ./target/debug/deps/ds-cd0cb9a5662c3402
    4M ./target/debug/ds
    4M ./target/release
    4M ./target/release/deps
    4M ./target/release/deps/ds-d307492d3cacef39
    4M ./target/release/ds
  996K ./target/debug/deps/libdiskspace-c8b9796891c5526c.rlib
  996K ./target/debug/libdiskspace.rlib
  996K ./target/debug/deps/libds-5149253a6d6a1a22.rlib
  996K ./target/debug/libds.rlib
   14K ./.git/hooks
   13K .
   12K ./.README.md.swp
    6K ./src
    4K ./.git/hooks/pre-rebase.sample
    3K ./.git/hooks/update.sample
```

This command can help find old browser cache files, a misplaced ISO image or a sudden increase in assets of a game.  

# Examples
To list all entries

```
$ ds -a
```

To reverse the sort

```
$ ds -r
```

To see any skipped files or directories and the error

```
$ ds -v
```

To search multiple directories

```
$ ds /home /local
```

On windows

```
> ds \Users \temp
```

To highlight the sizes in green

```
$ ds -c green
```

To turn off color

```
$ ds -c none
```

# Installation
From rust

```
$ cargo install diskspace
```

