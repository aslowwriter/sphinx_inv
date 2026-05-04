#[cfg(test)]
use std::io::Read;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::Path,
};

use flate2::read::ZlibDecoder;

use crate::{
    InventoryHeader, SphinxReference, error::SphinxInvError, header::parse_header,
    reference::parse_reference,
};
pub struct SphinxInventoryReader<R: std::io::Read> {
    header: InventoryHeader,
    // yes we double buffer here, which is necessary to make sure
    // we don't loose any input from the first buffer when we make the zlib decoder
    // if we just call .into_inner we'll loose part (don't ask how I know that).
    inner: Lines<BufReader<ZlibDecoder<BufReader<R>>>>,
    current_line: usize, // just for reporting
}

impl<R: std::io::Read> SphinxInventoryReader<R> {
    /// Construct a [`SphinxInventoryReader`] that wraps a impl [`std::io::Read`]
    /// Note that constructing this struct WILL cause reads immediately. Upon creation
    /// we will try to read and parse the header lines from the reader. This must succeed otherwise
    /// an Err will be returned. Subsequent reads will return parsed body lines.
    /// # Errors
    /// This function can return Err when:
    /// - An unsupported version format is mentinoed in the header (i.e. anything other than 2
    ///   currently)
    /// - the body is compressed with anything besides zlib, or the last header line does not
    ///   mention zlib
    /// - On any IO error while reading from the readaer
    pub fn from_reader(reader: R) -> Result<SphinxInventoryReader<R>, SphinxInvError> {
        let mut buffered_header_reader = BufReader::new(reader);
        let header = read_header(&mut buffered_header_reader)?;
        let new_reader = BufReader::new(ZlibDecoder::new(buffered_header_reader)).lines();

        Ok(SphinxInventoryReader {
            header,
            inner: new_reader,
            // 4 is to account for header lines
            current_line: 4,
        })
    }

    pub fn current_line(&self) -> usize {
        self.current_line
    }

    pub fn header(&self) -> &InventoryHeader {
        &self.header
    }
}

impl SphinxInventoryReader<File> {
    /// Construct a [`SphinxInventoryReader`] by reading the data from a [`std::path::Path`]
    /// # Errors
    /// This function can return Err when:
    /// - An unsupported version format is mentinoed in the header (i.e. anything other than 2
    ///   currently)
    /// - the body is compressed with anything besides zlib, or the last header line does not
    ///   mention zlib
    /// - On any IO error while reading from the readaer
    pub fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<SphinxInventoryReader<File>, SphinxInvError> {
        SphinxInventoryReader::from_reader(File::open(path)?)
    }
}

impl<R: std::io::Read> Iterator for SphinxInventoryReader<R> {
    type Item = Result<SphinxReference, SphinxInvError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_line += 1;
        let next = self.inner.next();
        parse_line(next, self.current_line)
    }
}

fn read_header<R: BufRead>(mut reader: &mut R) -> Result<InventoryHeader, SphinxInvError> {
    let header = parse_header(&mut reader)?;
    if header.inventory_version != 2 {
        return Err(SphinxInvError::UnsupportedInventoryVersion(
            header.inventory_version,
        ));
    }

    if !header.compression_method_description.contains("zlib") {
        return Err(SphinxInvError::UnsupportedCompressionMethod(
            header.compression_method_description,
        ));
    }

    Ok(header)
}

fn parse_line(
    maybe_line: Option<Result<String, io::Error>>,
    num_line: usize,
) -> Option<Result<SphinxReference, SphinxInvError>> {
    // Maybe it's mabeline
    match maybe_line {
        Some(read_line) => match read_line {
            Ok(line) => Some(parse_reference(&line, num_line).map_err(SphinxInvError::ParseError)),
            Err(err) => Some(Err(SphinxInvError::IoError(err))),
        },
        None => None,
    }
}

#[derive(Debug)]
pub struct PlainTextSphinxInventoryReader<R: std::io::Read> {
    header: InventoryHeader,
    inner: Lines<BufReader<R>>,
    current_line: usize, // just for reporting
}

#[cfg(test)]
impl<R: Read> PartialEq for PlainTextSphinxInventoryReader<R> {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
    }
}

impl<R: std::io::Read> Iterator for PlainTextSphinxInventoryReader<R> {
    type Item = Result<SphinxReference, SphinxInvError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_line += 1;
        parse_line(self.inner.next(), self.current_line)
    }
}
impl<R: std::io::Read> PlainTextSphinxInventoryReader<R> {
    /// Construct a [`SphinxInventoryReader`] that wraps a impl [`std::io::Read`]
    /// Note that constructing this struct WILL cause reads immediately. Upon creation
    /// we will try to read and parse the header lines from the reader. This must succeed otherwise
    /// an Err will be returned. Subsequent reads will return parsed body lines.
    /// # Errors
    /// This function can return Err when:
    /// - An unsupported version format is mentinoed in the header (i.e. anything other than 2
    ///   currently)
    /// - the body is compressed with anything besides zlib, or the last header line does not
    ///   mention zlib
    /// - On any IO error while reading from the readaer
    pub fn from_reader(reader: R) -> Result<PlainTextSphinxInventoryReader<R>, SphinxInvError> {
        let mut buffered_reader = BufReader::new(reader);
        let header = read_header(&mut buffered_reader)?;

        Ok(PlainTextSphinxInventoryReader {
            header,
            inner: buffered_reader.lines(),
            // 4 is to account for header lines
            current_line: 4,
        })
    }

    pub fn header(&self) -> &InventoryHeader {
        &self.header
    }
}

impl PlainTextSphinxInventoryReader<File> {
    /// Construct a [`SphinxInventoryReader`] by reading the data from a [`std::path::Path`]
    /// # Errors
    /// This function can return Err when:
    /// - An unsupported version format is mentinoed in the header (i.e. anything other than 2
    ///   currently)
    /// - the body is compressed with anything besides zlib, or the last header line does not
    ///   mention zlib
    /// - On any IO error while reading from the readaer
    pub fn from_path<P: AsRef<Path>>(
        path: P,
    ) -> Result<SphinxInventoryReader<File>, SphinxInvError> {
        SphinxInventoryReader::from_reader(File::open(path)?)
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use crate::{
        InventoryHeader, SphinxReference,
        error::{SphinxInvError, SphinxParseError},
        readers::PlainTextSphinxInventoryReader,
        roles::PyRole,
    };

    #[test]
    fn plain_text_reader_errors() -> Result<(), SphinxInvError> {
        let buffer = r"# Sphinx inventory file 2
# Project: <project display name>
# Version: <project version without preceding v>
# The remainder of this file is compressed using zlib.
str.join py:macro 1 library/stdtypes.html#$ -
str.lower py:method 24 library/stdtypes.html#$ -
str.lower py:method 1 library/stdtypes.html#$ -
";
        let reader = Cursor::new(buffer);

        let mut inv_reader = PlainTextSphinxInventoryReader::from_reader(reader)?;

        assert_eq!(
            *inv_reader.header(),
            InventoryHeader {
                project_name: "<project display name>".to_string(),
                project_version: "<project version without preceding v>".to_string(),
                inventory_version: 2,
                compression_method_description: "zlib".to_string()
            }
        );

        assert!(inv_reader.next().unwrap().is_err());
        assert!(inv_reader.next().unwrap().is_err());

        assert_eq!(
            inv_reader.next().unwrap().unwrap(),
            SphinxReference {
                name: "str.lower".to_string(),
                sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
                priority: crate::priority::SphinxPriority::Standard,
                location: "library/stdtypes.html#$".to_string(),
                display_name: "-".to_string()
            }
        );

        assert!(inv_reader.next().is_none());

        Ok(())
    }
    #[test]
    fn unsupported_inv_version() {
        let buffer = "# Sphinx inventory version 255
# Project: foo
# Version: bar
# zlib
"
        .as_bytes();
        let reader = Cursor::new(buffer);

        let result = PlainTextSphinxInventoryReader::from_reader(reader);
        assert_eq!(
            result,
            Err(SphinxInvError::UnsupportedInventoryVersion(255))
        );
    }
    #[test]
    fn plain_text_reader() -> Result<(), SphinxInvError> {
        let buffer = r"# Sphinx inventory file 2
# Project: <project display name>
# Version: <project version without preceding v>
# The remainder of this file is compressed using zlib.
str.join py:method 1 library/stdtypes.html#$ -
str.lower py:method 1 library/stdtypes.html#$ -
";
        let reader = Cursor::new(buffer);

        let mut inv_reader = PlainTextSphinxInventoryReader::from_reader(reader)?;

        assert_eq!(
            *inv_reader.header(),
            InventoryHeader {
                project_name: "<project display name>".to_string(),
                project_version: "<project version without preceding v>".to_string(),
                inventory_version: 2,
                compression_method_description: "zlib".to_string()
            }
        );

        assert_eq!(
            inv_reader.next().unwrap().unwrap(),
            SphinxReference {
                name: "str.join".to_string(),
                sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
                priority: crate::priority::SphinxPriority::Standard,
                location: "library/stdtypes.html#$".to_string(),
                display_name: "-".to_string()
            }
        );

        assert_eq!(
            inv_reader.next().unwrap().unwrap(),
            SphinxReference {
                name: "str.lower".to_string(),
                sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
                priority: crate::priority::SphinxPriority::Standard,
                location: "library/stdtypes.html#$".to_string(),
                display_name: "-".to_string()
            }
        );

        assert!(inv_reader.next().is_none());

        Ok(())
    }
    #[test]
    fn alternating_errors() -> Result<(), SphinxInvError> {
        let buffer = r"# Sphinx inventory file 2
# Project: <project display name>
# Version: <project version without preceding v>
# The remainder of this file is compressed using zlib.
str.join py:method 1 library/stdtypes.html#$ -
str.join asdf:method 1 library/stdtypes.html#$ -
str.upper py:method 1 library/stdtypes.html#$ -
str.upper py:macro 1 library/stdtypes.html#$ -
str.lower py:method 1 library/stdtypes.html#$ -
str.lower asdf:method 1 library/stdtypes.html#$ -
";
        let reader = Cursor::new(buffer);

        let mut inv_reader = PlainTextSphinxInventoryReader::from_reader(reader)?;

        assert_eq!(
            *inv_reader.header(),
            InventoryHeader {
                project_name: "<project display name>".to_string(),
                project_version: "<project version without preceding v>".to_string(),
                inventory_version: 2,
                compression_method_description: "zlib".to_string()
            }
        );

        assert_eq!(
            inv_reader.next().unwrap().unwrap(),
            SphinxReference {
                name: "str.join".to_string(),
                sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
                priority: crate::priority::SphinxPriority::Standard,
                location: "library/stdtypes.html#$".to_string(),
                display_name: "-".to_string()
            }
        );

        assert_eq!(
            inv_reader.next(),
            Some(Err(SphinxParseError::from_str(
                "str.join asdf:method 1 library/stdtypes.html#$ -",
                "invalid unknown domain\nexpected `std`, `py`, `c`, `rst`, `cpp`, `js`, `math`",
                14,
                6
            )
            .into()))
        );

        assert_eq!(
            inv_reader.next().unwrap().unwrap(),
            SphinxReference {
                name: "str.upper".to_string(),
                sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
                priority: crate::priority::SphinxPriority::Standard,
                location: "library/stdtypes.html#$".to_string(),
                display_name: "-".to_string()
            }
        );

        assert_eq!(
            inv_reader.next(),
            Some(Err(SphinxParseError::from_str(
                "str.upper py:macro 1 library/stdtypes.html#$ -",
                "invalid python role\nexpected `attribute`, `data`, `exception`, `function`, `method`, `module`, `property`, `class`",
                13,
                8
            )
            .into()))
        );

        assert_eq!(
            inv_reader.next().unwrap().unwrap(),
            SphinxReference {
                name: "str.lower".to_string(),
                sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
                priority: crate::priority::SphinxPriority::Standard,
                location: "library/stdtypes.html#$".to_string(),
                display_name: "-".to_string()
            }
        );

        assert_eq!(
            inv_reader.next(),
            Some(Err(SphinxParseError::from_str(
                "str.lower asdf:method 1 library/stdtypes.html#$ -",
                "invalid unknown domain\nexpected `std`, `py`, `c`, `rst`, `cpp`, `js`, `math`",
                15,
                10
            )
            .into()))
        );

        assert!(inv_reader.next().is_none());

        Ok(())
    }
}
