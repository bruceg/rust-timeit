[![crates.io](https://img.shields.io/crates/v/rust-timeit.svg)](https://crates.io/crates/rust-timeit)
[![license](https://img.shields.io/badge/license-Apache2.0-blue.svg)](https://github.com/bruceg/rust-timeit/blob/main/LICENSE)

rust-timeit
============

This package provides a simple way of timing small bits of Rust code. It
essentially serves as a command-line wrapper for
[criterion](https://bheisler.github.io/criterion.rs/book/index.html),
allowing for quickly running micro-benchmark tasks.

It takes its inspiration, and some of its form, from [Python's timeit
library](https://docs.python.org/3/library/timeit.html).

Installation
------------

To install the stable version, install Rust and run `cargo install
rust-timeit`.

To install from source, run `cargo install --path .` from the main
directory.

Example
-------

Which way of creating a zero-length string is fastest?

```sh
rust-timeit --perf instructions --setup 'let empty = String::new()' 'String::new()' 'String::from("")' 'empty.clone()' '"".to_owned()'
```

(Hint: `String::new()` is fastest)

License
-------

Copyright [2021] Bruce Guenter

Licensed under the Apache License, Version 2.0 (the "License"); you may
not use this file except in compliance with the License.  You may obtain
a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
