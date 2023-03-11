# Changelog

All notable changes to geo-uri-rs will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.1] - 2023-03-11

### Changed

* Bumped dependency on `derive_builder` crate
* Use `assert_eq` for float tests; drop dev depend on `float_eq` crate

### Fixed

* Fix doclinks in README
* Fix docs.rs metadata section name in `Cargo.toml`

## [0.2.0] - 2022-10-01

### Added

* Add support for converting from/to `Url` structs (#1)
* Add support for (de)serializing via serde (#2)

### Fixed

* Fix documentation and comment types and improve examples
* Make the crate adhere to the [Rust API guidelines](https://rust-lang.github.io/api-guidelines/)

## [0.1.1] - 2022-09-30

### Added

* Update examples in `README.md`
* Add some more fields to `Cargo.toml`

### Fixed

* Fix some small errors in the documentation

[Unreleased]: https://git.luon.net/paul/geo-uri-rs/compare/v0.2.1...HEAD
[0.2.1]: https://git.luon.net/paul/geo-uri-rs/compare/v0.2.0..v0.2.1
[0.2.0]: https://git.luon.net/paul/geo-uri-rs/compare/v0.1.1..v0.2.0
[0.1.1]: https://git.luon.net/paul/geo-uri-rs/commits/tag/v0.1.1
