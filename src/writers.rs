use std::io::Write;

use flate2::{Compression, write::ZlibEncoder};

use crate::{InventoryHeader, SphinxReference};

#[derive(Debug)]
pub struct PlainTextSphinxInventoryWriter<'a, 'b> {
    header: &'a InventoryHeader,
    buffer: Vec<&'b SphinxReference>,
}

impl<'a, 'b> PlainTextSphinxInventoryWriter<'a, 'b> {
    #[must_use]
    pub fn from_header(header: &'a InventoryHeader, capacity: usize) -> Self {
        Self {
            header,
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn add_reference(&mut self, reference: &'b SphinxReference) {
        self.buffer.push(reference);
    }

    pub fn finalize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        writer.write_all(format!("{}", self.header).as_bytes())?;
        for reference in self.buffer {
            writer.write_all(format!("{reference}\n").as_bytes())?;
        }
        Ok(())
    }
}

pub struct SphinxInventoryWriter<'a, 'b> {
    header: &'a InventoryHeader,
    buffer: Vec<&'b SphinxReference>,
}

impl<'a, 'b> SphinxInventoryWriter<'a, 'b> {
    pub fn from_header(header: &'a InventoryHeader, capacity: usize) -> Self {
        Self {
            header,
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn add_reference(&mut self, reference: &'b SphinxReference) {
        self.buffer.push(reference);
    }

    pub fn finalize<W: Write>(self, writer: &mut W) -> Result<&mut W, std::io::Error> {
        writer.write_all(format!("{}", self.header).as_bytes())?;
        writer.flush()?;
        let mut zlib_writer = ZlibEncoder::new(writer, Compression::fast());
        for reference in self.buffer {
            zlib_writer.write_all(format!("{reference}\n").as_bytes())?;
        }
        zlib_writer.finish()
    }
}

#[cfg(test)]
mod test {
    #![allow(clippy::unwrap_used)]
    use pretty_assertions::assert_eq;
    use std::io::Cursor;

    use crate::{
        InventoryHeader, SphinxInventoryReader, SphinxReference,
        error::SphinxInvError,
        roles::PyRole,
        writers::{PlainTextSphinxInventoryWriter, SphinxInventoryWriter},
    };

    #[test]
    fn write_simple_plain_text_inventory() -> Result<(), SphinxInvError> {
        let mut write_buffer = Vec::new();
        let expected = String::from(
            "# Sphinx inventory version 2
# Project: foo
# Version: 0.24.24
# The remainder of this file is compressed using zlib.
str.join py:method 1 library/stdtypes.html#str.join str.join
str.lower py:method 1 library/stdtypes.html#str.lower str.lower
",
        );
        let header = InventoryHeader {
            project_name: "foo".to_string(),
            project_version: "0.24.24".to_string(),
            inventory_version: 2,
            compression_method_description: "zlib".to_string(),
        };

        let str_lower_ref = SphinxReference {
            name: "str.lower".to_string(),
            sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
            priority: crate::priority::SphinxPriority::Standard,
            location: "library/stdtypes.html#str.lower".to_string(),
            display_name: "str.lower".to_string(),
        };

        let str_join_ref = SphinxReference {
            name: "str.join".to_string(),
            sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
            priority: crate::priority::SphinxPriority::Standard,
            location: "library/stdtypes.html#str.join".to_string(),
            display_name: "str.join".to_string(),
        };

        let mut writer = PlainTextSphinxInventoryWriter::from_header(&header, 2);

        writer.add_reference(&str_join_ref);
        writer.add_reference(&str_lower_ref);

        let mut cursor = Cursor::new(&mut write_buffer);

        writer.finalize(&mut cursor)?;

        assert_eq!(String::from_utf8(write_buffer)?, expected);
        Ok(())
    }

    #[test]
    fn write_read_round_trip() -> Result<(), SphinxInvError> {
        let buffer = Vec::new();
        let mut cursor = Cursor::new(buffer);
        let header = InventoryHeader {
            project_name: "foo".to_string(),
            project_version: "0.24.24".to_string(),
            inventory_version: 2,
            compression_method_description: "zlib".to_string(),
        };

        let str_lower_ref = SphinxReference {
            name: "str.lower".to_string(),
            sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
            priority: crate::priority::SphinxPriority::Standard,
            location: "library/stdtypes.html#str.lower".to_string(),
            display_name: "str.lower".to_string(),
        };

        let str_join_ref = SphinxReference {
            name: "str.join".to_string(),
            sphinx_type: crate::roles::SphinxType::Python(PyRole::Method),
            priority: crate::priority::SphinxPriority::Standard,
            location: "library/stdtypes.html#str.join".to_string(),
            display_name: "str.join".to_string(),
        };
        let mut writer = SphinxInventoryWriter::from_header(&header, 2);

        writer.add_reference(&str_join_ref);
        writer.add_reference(&str_lower_ref);
        writer.finalize(&mut cursor)?;

        cursor.set_position(0);

        let mut reader = SphinxInventoryReader::from_reader(cursor)?;

        assert_eq!(reader.header(), &header);

        assert_eq!(reader.next().unwrap()?, str_join_ref);

        assert_eq!(reader.next().unwrap()?, str_lower_ref);

        Ok(())
    }
}
