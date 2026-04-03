pub mod error;

mod header;
mod priority;
mod reference;
mod roles;

pub use header::InventoryHeader;

use flate2::bufread::ZlibDecoder;
pub use reference::SphinxReference;
use std::{fs::File, io::Read, path::Path};
use winnow::Parser;

use crate::{error::SphinxInvError, header::parse_header, reference::references};

fn decompress_remaining_zlib_data(buffer: &mut &[u8]) -> Result<String, SphinxInvError> {
    let mut decoder = ZlibDecoder::new(&buffer[..]);

    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;

    let decompressed_text = String::from_utf8(decompressed_data)?;
    Ok(decompressed_text)
}

/// # Errors
/// Returns errors on parse errors
pub fn parse_inventory(
    buffer: &mut &[u8],
) -> std::result::Result<Vec<SphinxReference>, SphinxInvError> {
    let header = parse_header(buffer).map_err(|_| SphinxInvError::MalformedHeader)?;

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

    let decompressed = decompress_remaining_zlib_data(buffer)?;

    let refs = references
        .parse_next(&mut decompressed.as_ref())
        .map_err(|e| SphinxInvError::ParseError(format!("{e:?}")))?;

    Ok(refs)
}

/// Parse a Sphinx Inventory file on disk. This will load, decompress and parse the data
/// # Errors
/// returns errors on IO errors or parse errors
pub fn parse_inventory_file(path: &Path) -> Result<Vec<SphinxReference>, SphinxInvError> {
    let mut file = File::open(path)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)?;
    parse_inventory(&mut buf.as_slice())
}
