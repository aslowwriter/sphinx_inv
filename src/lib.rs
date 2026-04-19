//! A library to parse (and maybe one day write) Sphinx inventory files
//! for referencing other documentation pages that use Sphinx.
//!
//! In contraty to Sphinx itself this library parses the data using a
//! combinator parser, instead of a regex, which has better performance
//! and better error reporting.
//!
//! This library was originally made for use in [ `snakedown` ]( https://crates.io/crates/snakedown )
//! but an effort has been made to make it more generally useful.
//!
//! ### Disclaimer
//! The Sphinx inventory format doesn't have a formal specification
//! instead what follows is just the rules that we (and other) have
//! inferred from files we've seen in the wild. We try to be as correct as possible.
//! That said, we can't be guaranteed to be correct. If you find any errors, please open an issue!
//!
//! Currently only v2 is supported.
//!
//! ## Usage
//!
//! The main point of entry for this crate is the [`readers::SphinxInventoryReader`] which you can use to
//! read and parse inventory data from any source that implements [`std::io::Read`]. These will
//! produce [`InventoryHeader`] structs and an iterator over [`SphinxReference`]
//!
//! ## Examples
//!
//! ### Reading
//!
//! You can construct a reader by using the [`readers::SphinxInventoryReader::from_reader`] like so:
//! (Currently we use a non-existent file because the writing functionality has not been developed
//! yet this will be rectified asap, but for now the following doctests are skipped)
//!
//! ```ignore
//! # use sphinx_inv::SphinxInventoryReader;
//! # use std::fs::File;
//! # use std::path::Pathbuf;
//! #
//! let path = PathBuf::from("objects.inv");
//! let mut file = File::open(path)?;
//! let reader = SphinxInventoryReader::from_reader(file).unwrap();
//!
//! println!("{:?}", reader.header());
//!
//! for reference in reader {
//!     println!("{}", reference.unwrap());
//! }
//! ```
//!
//! If you have a local file you can also use the convenience wrapper
//!
//! if you have the data already in memory you can parse that instead: [`readers::SphinxInventoryReader::from_path`]
//! ```ignore
//! # use sphinx_inv::SphinxInventoryReader;
//! # use std::path::PathBuf;
//! #
//! let path = PathBuf::from("objects.inv");
//! let reader = SphinxInventoryReader::from_path(&path).unwrap();
//!
//! println!("{:?}", reader.header());
//!
//! for reference in reader {
//!     println!("{:?}", reference.unwrap());
//! }
//! ```
//!
//! Note that this will consume the buffer, and afterwards it should be left empty.
//!
//!
//! ## Format Description
//!
//! ### General format description
//! As noted by Skinn et al. currently, a inventory file (in the v2 format) has 2 parts:
//! the header and the body.
//!
//! The header needs to be of the following format:
//! ```txt
//! # Sphinx inventory version 2
//! # Project: <project name>
//! # Version: <full version number>
//! # The remainder of this file is compressed using zlib.
//! ```
//!
//! #### Caveats:
//! 1. The first line has to match exactly
//! 2. version number should not contain a leading `v`
//! 3. currently zlib is the only compression method that Sphinx supports.
//! 4. Though it is not specified, it is expected that the text mentioned above is in ascii.
//!    The project name can contain unicode, but the text in the example must match exactly[^*].
//! 5. While Sphinx itself allows for userdefinable domains and roles, this is not possible for this
//!    library due to being complied. However we have made an attempt to include as many domains and
//!    roles we found out in the wild. If you are missing any, please submit a feature request or pull
//!    request to add it!
//!
//! For more indepth explanation of the format, please see
//! [spobjinv](https://sphobjinv.readthedocs.io/en/stable/syntax.html)
//!
//! [^*]: technically it doesn't as long as the byte offsets are the same since the Sphinx
//! implementation just skips a known amount of bytes, but this is a impl detail so
//! we recommend that the format is still followed
//!
//!
//!### Body format
//!
//! The remaining body of the file after the header must be compressed with zlib.
//! In the decompressed data each line should have the following format:
//!
//! ```txt
//! {name} {domain}:{role} {priority} {uri} {dispname}
//! ```
//!
//! Specifically it must match this regex:
//! `(.+?)\s+(\S+)\s+(-?\d+)\s+?(\S*)\s+(.*)`
//!
//! For example:
//! ```txt
//! str.join py:method 1 library/stdtypes.html#$ -
//! ```
//!
//! ## Special Thanks
//!
//! - Thank you to Brian Skinn et al. for all the research they did into the format
//!   which they documented in the
//!   [sphobjinv package](https://sphobjinv.readthedocs.io/en/stable/syntax.html)
//!   They have been invaluable in writing this library.
//! - Thank you to `BurntSushi` for writing the `csv` crate which has been a great example
//!   to follow when designing the API

pub mod error;
pub mod readers;

mod header;
mod priority;
mod reference;
mod roles;

pub use header::InventoryHeader;
pub use readers::SphinxInventoryReader;
pub use reference::SphinxReference;
