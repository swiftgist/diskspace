The motivation of this project is to learn Rust.  After reading half of two different books, the program is useful but primitive.  My next goals are further exploring unit tests and command line arguments.

# Filesystem Full?
Linux users are familiar with the `df` and `du` commands, respectively disk free and disk usage.  When a filesystem is full (or nearly so), `df` will confirm that available space is zero.  The `du` command with options can be more helpful.  For example, 
```
du -sm * | sort -n 
```
will list all files and directories.  However, hidden files and directories are not included.  Using 
```
du -sm .??* *| sort -n
```
 will include hidden files and directories without including the parent directory.  Additionally, knowing the largest directory simply means traversing any of the subdirectories and repeating the command.  Tracking down the largest directories and files allows one to make the decision to remove them or accept that more space is necessary.

Explaining the above to new Linux users such as how shell expansion works, that directories such as `.a` will be skipped and the large numbers are all mebibytes will be unwelcome. 

# DiskSpace
The program `ds` returns the 20 largest directories and files from the current directory.  The output is human friendly using suffixes.  For example, the output of the src directory is

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

This command can help find browser cache files, a misplaced ISO image or a sudden increase in a game's asset files.

# Buidling
Since this is Rust, the command is

```
$ cargo build
```

