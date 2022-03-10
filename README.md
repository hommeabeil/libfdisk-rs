# libfdisk-rs
Rust wrappers for libfdisk. This is the new home for [this
repository](https://github.com/IBM/fdisk-lib).

For those who coming from the old repository, the version `0.1.3` is exactly the
same thing (no changes).

## Dependencies

In order to compile this project you will need to install those packages on your
system:
1. llvm
1. libfdisk-dev
1. clang

On Debian you can run:

```sh
apt install llvm libfdisk-dev clang
```