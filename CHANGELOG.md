# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.1](https://github.com/aslowwriter/sphinx_inv/compare/v0.3.0...v0.3.1) - 2026-06-08

### Added

- add hyperfine benchmark

### Other

- *(deps)* bump codecov/codecov-action from 6 to 7

## [0.3.0](https://github.com/aslowwriter/sphinx_inv/compare/v0.2.0...v0.3.0) - 2026-05-07

### Fixed

- handle index entry
- handle minification/expansion of references

## [0.2.0](https://github.com/aslowwriter/sphinx_inv/compare/v0.1.0...v0.2.0) - 2026-05-06

### Added

- add http domain
- add sip domain
- add cmake domain
- add more python roles
- add rst:role
- simplify error types
- add writer structs
- add a plaintext reader
- implement public reader struct
- implement parsing using winnow

### Fixed

- add some missing cpp roles
- add missing js role
- move missing domain err msg to correct parser
- update github handles

### Other

- update readme
- centralize domain:role parsing tests
- add top level crate documentation
- *(deps)* bump codecov/codecov-action from 5 to 6
