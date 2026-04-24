# Changelog

## Version 0.5.0

### Breaking Changes

- Updated to use the Rust 2024 edition in generated code.
- Made the black box mode unconditional and dropped the `--black-box` flag,
  since we always want to execute the given expression.
- Replaced the crates used to support the `--perf` option on Linux, which has
  changed the names of the values for that option.

### Fixes

- Fixed warning about `unnecessary parentheses around closure body`.
- Use `std::hint::black_box` instead of the one from `criterion`.

### Miscellaneous

- Updated `criterion` to version 0.5.
