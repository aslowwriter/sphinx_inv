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
//! The Sphinx inventory format doesn't have a formal specification.
//! What follows are just the rules that we (and others) have
//! inferred from files we've seen in the wild. We try to be as correct as possible.
//! That said, we can't be guaranteed to be correct. If you find any errors, or have a valid file we
//! can't parse please open an issue!
//!
//! Currently only v2 of the Sphinx Inventory file format is supported.
//!
//! ## Usage
//!
//! The main entry points of this create are the the [`InventoryHeader`] and [`SphinxReference`] data
//! structs and the [`SphinxInventoryReader`] and [`SphinxInventoryWriter`]
//! structs to handle with them.
//!
//! The [`SphinxInventoryReader`] and [`SphinxInventoryWriter`] can work with any struct that
//! implements [`std::io::Read`] and [`std::io::Write`] respectively. These are internally buffered
//! so you do not have to wrap them yourself.
//!
//! In previous version we had separate structs for the zlib and plain-text versions, but these
//! have now all been combined into one reader and one writer. The reader will detect which is
//! necessary based on the description in the header. For example if the header metnions:
//!
//! `# The remainder of this file is compressed using zlib`
//!
//! it will automatically decompress the body, whereas if it says
//!
//! `# The remainder of this file is compressed using plain-text`
//!
//! it will use plain text reading. The writer will write the correct header along with the body in
//! the correct format based on the [`WriteFormat`] provided at finalisation.
//!
//! In the following examples we will use the plain text versions and
//! the [`std::io::Cursor`] to make it easier to display the results, but the code should work
//! basically unchanged by switching to a [`std::fs::File`].
//!
//! ## Examples
//!
//!
//! ```
//! # use pretty_assertions::assert_eq;
//! # use sphinx_inv::*;
//! # use std::fs::File;
//! # use std::io::{Cursor, Read, Write};
//! #
//! let header = InventoryHeader::new("Sphinx Inv", "0.2.0", "plain-text");
//! let join_reference = SphinxReference::new(
//!     "str.join",
//!     SphinxType::Python(PyRole::Method),
//!     SphinxPriority::Standard,
//!     "library/stdtypes.html#$",
//!     "-",
//! );
//! let lower_reference = SphinxReference::new(
//!     "str.lower",
//!     SphinxType::Python(PyRole::Method),
//!     SphinxPriority::Standard,
//!     "library/stdtypes.html#$",
//!     "-",
//! );
//!
//! let mut buffer = Vec::new();
//!
//! let mut cursor = Cursor::new(buffer);
//!
//! // the capacity is just to preallocate the internal buffer, it can be anything
//! let mut writer = SphinxInventoryWriter::from_header(header.clone(), 2);
//!
//! // add the references to the writer
//! writer.add_reference(join_reference.clone());
//! writer.add_reference(lower_reference.clone());
//!
//! // add_reference on it's own only adds it to the internal buffer
//! // nothing actually happens until you call [`SphinxInventoryWriter::finalize`]
//! writer
//!     .finalize(&mut cursor, &WriteFormat::Plain, true)
//!     .unwrap();
//!
//! let written = String::from_utf8(cursor.into_inner()).unwrap();
//!
//! assert_eq!(
//!     &written,
//!     "# Sphinx inventory version 2
//! ## Project: Sphinx Inv
//! ## Version: 0.2.0
//! ## The remainder of this file is compressed using plain-text.
//! str.join py:method 1 library/stdtypes.html#$ -
//! str.lower py:method 1 library/stdtypes.html#$ -
//! "
//! );
//!
//! let mut cursor = Cursor::new(written);
//!
//! let mut reader = SphinxInventoryReader::from_reader(cursor).unwrap();
//!
//! assert_eq!(&header, reader.header());
//!
//! assert_eq!(reader.next().unwrap().unwrap(), join_reference);
//! assert_eq!(reader.next().unwrap().unwrap(), lower_reference);
//! ```
//!
//!
//! ## Format Description
//!
//! As noted by Skinn et al. currently, a inventory file (in the v2 format) has 2 parts:
//!
//! - the header
//! - the body
//!
//! ### Header description
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
//! [^*]: technically it matter doesn't as long as the byte offsets are the same since the Sphinx
//! implementation just skips a known amount of bytes, but this is a impl detail so
//! we recommend following the format.
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

mod error;
mod header;
mod priority;
mod readers;
mod reference;
mod roles;
mod writers;

/// The main error type returned by this crate
pub use error::SphinxInvError;

/// Error type when parsing either the header or a record fails.
pub use error::SphinxParseError;

/// Error type when there is not enough input from the underlying reader
/// to properly parse the header
pub use error::MissingHeaderComponent;

/// Struct for handling the metadata of an inventory such as project name and version
pub use header::InventoryHeader;

/// The main entrypoint to this crate, used to read and parse sphinx reference data
pub use readers::SphinxInventoryReader;

/// The main data struct of this crate with the necessary information to link to external
pub use reference::SphinxReference;

/// The main entrypoint to this crate, used to write and format sphinx reference data
pub use writers::{SphinxInventoryWriter, WriteFormat};

/// type used to parse `{domain}:{roles}` information provided by Sphinx used to disembguate
/// between object types and names between different languages
pub use roles::SphinxType;

/// The search priority of the associated object used by Sphinx
pub use priority::SphinxPriority;

/// The various known domains and roles used in sphinx inventory files
pub use roles::{CRole, CmakeRole, CppRole, JsRole, MathRole, PyRole, RstRole, SipRole, StdRole};
