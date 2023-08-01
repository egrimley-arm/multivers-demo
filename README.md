# Multivers-demo

Cargo allows a project to use two different versions of the same
crate, but not if the crate includes a native library because in this
case you would be likely to have identically named libraries
containing identically named symbols, both of which would confuse the
native linker. If the crate correctly uses the `links` field in its
`Cargo.toml` then the conflict will be detected by Cargo. Otherwise
there might be linker errors, or run-time errors, or perhaps, by luck,
everything will work.

This crate has a single trivial program that directly uses two
versions of the same library; usually the two versions of the same
library would be indirect dependencies, of course. The two versions
are in subdirectories called `verslib-v1` and `verslib-v2` and they
are identical apart from the `version` in `Cargo.toml` and the value
returned by the C function `verslib_version`.

To avoid the two C libraries colliding at link time the `build.rs` in
`verslib` renames the C symbols before linking. See the comments in
that file.
