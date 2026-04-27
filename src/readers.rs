use std::io::{BufRead, BufReader, Lines};

use flate2::read::ZlibDecoder;

use crate::{
    InventoryHeader, SphinxReference, error::SphinxInvError, header::parse_header,
    reference::reference,
};
pub struct SphinxInventoryReader<R: std::io::Read> {
    header: InventoryHeader,
    inner: Lines<BufReader<ZlibDecoder<R>>>,
    current_line: usize, // just for reporting
}

impl<R: std::io::Read> SphinxInventoryReader<R> {
    /**
    Construct a [`SphinxInventoryReader`] that wraps a impl [`std::io::Read`]
    # Errors
    This function can return Err when:
    - An unsupported version format is mentinoed in the header (i.e. anything other than 2
      currently)
    - the body is compressed with anything besides zlib, or the last header line does not
      mention zlib
    - On any IO error while reading from the readaer
    */
    pub fn from_reader(reader: R) -> Result<SphinxInventoryReader<R>, SphinxInvError> {
        let mut buffered_header_reader = BufReader::new(reader);
        let header = parse_header(&mut buffered_header_reader)?;
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

        let new_reader =
            BufReader::new(ZlibDecoder::new(buffered_header_reader.into_inner())).lines();

        Ok(SphinxInventoryReader {
            header,
            inner: new_reader,
            current_line: 0,
        })
    }

    pub fn header(&self) -> &InventoryHeader {
        &self.header
    }
}

impl<R: std::io::Read> Iterator for SphinxInventoryReader<R> {
    type Item = Result<SphinxReference, SphinxInvError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_line += 1;
        match self.inner.next() {
            Some(read_line) => match read_line {
                Ok(line) => Some(reference(&mut line.as_str()).map_err(|_| {
                    SphinxInvError::ParseError(format!("error parsing line {0}", self.current_line))
                })),
                // we'll use the error for better error reporting in a later API iteration
                Err(_err) => Some(Err(SphinxInvError::ParseError(format!(
                    "error reading line {0}",
                    self.current_line
                )))),
            },
            None => None,
        }
    }
}
