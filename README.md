# mtab-rs

A Rust crate for reading and writing the `/etc/mtab` file. Can be used to query the filesystems mounted on the system.

This crate automatically parses some of the mount options, and cointains functions for querying if, for example, a path is mounted on a particular filesystem.