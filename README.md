# sphinx_inv

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![codecov](https://codecov.io/gh/aslowwriter/sphinx_inv/branch/main/graph/badge.svg)](https://codecov.io/gh/savente93/sphinx_inv)
[![crates.io](https://img.shields.io/crates/v/sphinx_inv)](https://crates.io/crates/sphinx_inv)
[![Docs.rs](https://docs.rs/sphinx_inv/badge.svg)](https://docs.rs/sphinx_inv)


A rust library to parse Sphinx `objects.inv` files.

Sphinx the documentation generator will output a file called `objects.inv` containing data so that other websites can link to it easily. This library is made to read, parse, and write files like that. It was initially developed for use in [snakedown](https://github.com/aslowwriter/snakedown) but should be useful for anyone wanting to interact with these files.

## Usage

To use the library simply add it to your project:

```
cargo add sphinx_inv
```

The main entry points of this create are the the [`InventoryHeader`] and [`SphinxReference`] data
structs and the [`SphinxInventoryReader`] and [`SphinxInventoryWriter`]
structs to handle with them.

The [`SphinxInventoryReader`] and [`SphinxInventoryWriter`] can work with any struct that
immplements [`std::io::Read`] and [`std::io::Write`] respectively. These are internally buffered
so you do not have to wrap them yourself.

When interacting with real `objects.inv` files in the wild you will most likely use the base
reader and writer struct, but both also have a `PlainText` variant. The only difference is that
the plain text versions don't encode/decode the data in zlib like the files do. This is mostly
useful for debugging/testing. In the following examples we will use the plain text versions and
the [`std::io::Cursor`] to make it easier to display the results, but the code should work
basically unchanged by switching to a [`std::fs::File`] and the base readers and writers.

## Example

```rust
use sphinx_inv::*;
use std::fs::File;
use std::io::{Read, Write, Cursor};
use pretty_assertions::assert_eq;

let header = InventoryHeader::new("Sphinx Inv", "0.2.0");

let join_reference = SphinxReference::new(
    "str.join".to_string(),
    SphinxType::Python(PyRole::Method),
    None,
    "library/stdtypes.html#$".to_string(),
    None);

let lower_reference = SphinxReference::new(
    "str.lower".to_string(),
    SphinxType::Python(PyRole::Method),
    None,
    "library/stdtypes.html#$".to_string(),
    None);

let mut buffer = Vec::new();

let mut cursor = Cursor::new(buffer);

// the capacity is just to preallocate the internal buffer, it can be anything
let mut writer = PlainTextSphinxInventoryWriter::from_header(&header, 2);


// add the references to the writer
writer.add_reference(&join_reference);
writer.add_reference(&lower_reference);

// add_reference on it's own only adds it to the internal buffer
// nothing actually happens until you call [`SphinxInventoryWriter::finalize`]
writer.finalize(&mut cursor).unwrap();

let written = String::from_utf8(cursor.into_inner()).unwrap();

assert_eq!(&written, "# Sphinx inventory version 2
# Project: Sphinx Inv
# Version: 0.2.0
# The remainder of this file is compressed using zlib.
str.join py:method 1 library/stdtypes.html#$ -
str.lower py:method 1 library/stdtypes.html#$ -
");

let mut cursor = Cursor::new( written);

let mut reader = PlainTextSphinxInventoryReader::from_reader(cursor).unwrap();

assert_eq!(&header, reader.header());

assert_eq!(reader.next().unwrap().unwrap(), join_reference);
assert_eq!(reader.next().unwrap().unwrap(), lower_reference);
```


## FAQ

### Q: I have a file that isn't parsing!

Since this is written in a compiled language we can't easily install extensions like python can, therefore it is very possible that you have a valid file that isn't parsing correctly. Because there is a lot of extensions out there and it is unclear how many of them are still actively used we limited ourselves to a few of the bigger projects (like `http`, `sip` provided by PyQt, and `cmake`). If you have one that we don't support yet, please [open an issue](https://github.com/aslowwriter/sphinx_inv/issues/new), we'd love to fix it!

Regardless of whether you found a bug, or a domain/role you'd like us to support, if possible please include the file that has the failing lines, if not provide at least an example line. If there are any links to documentation of the domain and roles available we'd love those as well!

### Q: What's the status of the project?

Currently the project is mostly "done." That means that it does what I need it to do for now, so it may not see regular updates. However, I'm happy to take bug reports and feature requests. The project is still maintained, but we'd rather wait to have actual usecases we can address properly rather than implement a bunch of features nobody is interested in.


## Acknowledgements

- Thank you to Brian Skinn et al. for all the research they did into the format
  which they documented in the [sphobjinv package](https://sphobjinv.readthedocs.io/en/stable/syntax.html)
  They have been invaluable in writing this library.
- Thank you to `@BurntSushi` for writing the `csv` crate which has been a great example
  to follow when designing the API

## Template

This repo was initially setup using [`cargo-generate`](https://github.com/cargo-generate/cargo-generate) and [this template](https://github.com/aslowwriter/rust-template)
