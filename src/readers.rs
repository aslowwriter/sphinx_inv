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

#[derive(Debug)]
pub struct SphinxInventoryReader<R: Read> {
    header: InventoryHeader,
    // yes we double buffer here, which is necessary to make sure
    // we don't loose any input from the first buffer when we make the zlib decoder
    // if we just call .into_inner we'll loose part (don't ask how I know that).
    inner: InnerReader<R>,
    current_line: usize, // just for reporting
}

#[derive(Debug)]
pub enum InnerReader<R: Read> {
    Plain(Lines<BufReader<R>>),
    Zlib(Lines<BufReader<ZlibDecoder<BufReader<R>>>>),
}

impl<R: std::io::Read> Iterator for InnerReader<R> {
    type Item = Result<String, io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            InnerReader::Plain(lines) => lines.next(),
            InnerReader::Zlib(lines) => lines.next(),
        }
    }
}

impl<R: Read> PartialEq for SphinxInventoryReader<R> {
    fn eq(&self, other: &Self) -> bool {
        self.header == other.header
            && std::mem::discriminant(&self.inner) == std::mem::discriminant(&other.inner)
            && self.current_line == other.current_line
    }
}

impl<R: Read> SphinxInventoryReader<R> {
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
        let new_reader = if header.compression_method_description.contains("zlib") {
            Ok(InnerReader::Zlib(
                BufReader::new(ZlibDecoder::new(buffered_header_reader)).lines(),
            ))
        } else if header.compression_method_description.contains("plain-text") {
            Ok(InnerReader::Plain(buffered_header_reader.lines()))
        } else {
            Err(SphinxInvError::UnsupportedCompressionMethod(
                header.compression_method_description.clone(),
            ))
        }?;

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

    if !header.compression_method_description.contains("zlib")
        && !header.compression_method_description.contains("plain-text")
    {
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

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use std::io::Cursor;

    use pretty_assertions::assert_eq;

    use crate::{
        InventoryHeader, SphinxReference, SphinxType,
        error::{SphinxInvError, SphinxParseError},
        priority::SphinxPriority,
        readers::SphinxInventoryReader,
        roles::PyRole,
    };

    #[test]
    fn plain_text_reader_errors() -> Result<(), SphinxInvError> {
        let buffer = r"# Sphinx inventory file 2
# Project: <project display name>
# Version: <project version without preceding v>
# The remainder of this file is compressed using plain-text.
str.join py:macro 1 library/stdtypes.html#$ -
str.lower py:method 24 library/stdtypes.html#$ -
str.lower py:method 1 library/stdtypes.html#$ -
";
        let reader = Cursor::new(buffer);

        let mut inv_reader = SphinxInventoryReader::from_reader(reader)?;

        assert_eq!(
            *inv_reader.header(),
            InventoryHeader {
                project_name: "<project display name>".to_string(),
                project_version: "<project version without preceding v>".to_string(),
                inventory_version: 2,
                compression_method_description: "plain-text".to_string()
            }
        );

        assert!(inv_reader.next().unwrap().is_err());
        assert!(inv_reader.next().unwrap().is_err());

        let str_lower_ref = SphinxReference::new(
            "str.lower",
            SphinxType::Python(PyRole::Method),
            SphinxPriority::Standard,
            "library/stdtypes.html#str.lower",
            "str.lower",
        );

        assert_eq!(inv_reader.next().unwrap().unwrap(), str_lower_ref);

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

        let result = SphinxInventoryReader::from_reader(reader);
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
# The remainder of this file is compressed using plain-text.
str.join py:method 1 library/stdtypes.html#$ -
str.lower py:method 1 library/stdtypes.html#$ -
";
        let reader = Cursor::new(buffer);

        let mut inv_reader = SphinxInventoryReader::from_reader(reader)?;

        assert_eq!(
            *inv_reader.header(),
            InventoryHeader {
                project_name: "<project display name>".to_string(),
                project_version: "<project version without preceding v>".to_string(),
                inventory_version: 2,
                compression_method_description: "plain-text".to_string()
            }
        );

        let str_lower_ref = SphinxReference::new(
            "str.lower",
            SphinxType::Python(PyRole::Method),
            SphinxPriority::Standard,
            "library/stdtypes.html#str.lower",
            "str.lower",
        );

        let str_join_ref = SphinxReference::new(
            "str.join",
            SphinxType::Python(PyRole::Method),
            SphinxPriority::Standard,
            "library/stdtypes.html#str.join",
            "-",
        );
        assert_eq!(inv_reader.next().unwrap().unwrap(), str_join_ref);

        assert_eq!(inv_reader.next().unwrap().unwrap(), str_lower_ref);

        assert!(inv_reader.next().is_none());

        Ok(())
    }
    #[test]
    fn alternating_errors() -> Result<(), SphinxInvError> {
        let buffer = r"# Sphinx inventory file 2
# Project: <project display name>
# Version: <project version without preceding v>
# The remainder of this file is compressed using plain-text.
str.join py:method 1 library/stdtypes.html#$ -
str.join asdf:method 1 library/stdtypes.html#$ -
str.upper py:method 1 library/stdtypes.html#$ -
str.upper py:macro 1 library/stdtypes.html#$ -
str.lower py:method 1 library/stdtypes.html#$ -
str.lower asdf:method 1 library/stdtypes.html#$ -
";
        let reader = Cursor::new(buffer);

        let mut inv_reader = SphinxInventoryReader::from_reader(reader)?;

        assert_eq!(
            *inv_reader.header(),
            InventoryHeader {
                project_name: "<project display name>".to_string(),
                project_version: "<project version without preceding v>".to_string(),
                inventory_version: 2,
                compression_method_description: "plain-text".to_string()
            }
        );
        let str_upper_ref = SphinxReference::new(
            "str.upper",
            SphinxType::Python(PyRole::Method),
            SphinxPriority::Standard,
            "library/stdtypes.html#str.upper",
            "str.upper",
        );
        let str_lower_ref = SphinxReference::new(
            "str.lower",
            SphinxType::Python(PyRole::Method),
            SphinxPriority::Standard,
            "library/stdtypes.html#str.lower",
            "str.lower",
        );

        let str_join_ref = SphinxReference::new(
            "str.join",
            SphinxType::Python(PyRole::Method),
            SphinxPriority::Standard,
            "library/stdtypes.html#str.join",
            "-",
        );

        assert_eq!(inv_reader.next().unwrap().unwrap(), str_join_ref);

        assert_eq!(
            inv_reader.next(),
            Some(Err(SphinxParseError::from_str(
                "str.join asdf:method 1 library/stdtypes.html#$ -",
                "invalid missing domain:role\nexpected `std`, `py`, `c`, `rst`, `cpp`, `js`, `math`",
                48,
                6
            )
            .into()))
        );

        assert_eq!(inv_reader.next().unwrap().unwrap(), str_upper_ref);

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

        assert_eq!(inv_reader.next().unwrap().unwrap(), str_lower_ref);

        assert_eq!(
            inv_reader.next(),
            Some(Err(SphinxParseError::from_str(
                "str.lower asdf:method 1 library/stdtypes.html#$ -",
                "invalid missing domain:role\nexpected `std`, `py`, `c`, `rst`, `cpp`, `js`, `math`",
                49,
                10
            )
            .into()))
        );

        assert!(inv_reader.next().is_none());

        Ok(())
    }
}
