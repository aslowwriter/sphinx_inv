use std::io::Write;

use flate2::{Compression, write::ZlibEncoder};

use crate::{InventoryHeader, SphinxReference};

#[derive(Debug)]
pub struct PlainTextSphinxInventoryWriter<'a> {
    header: &'a InventoryHeader,
    buffer: Vec<&'a SphinxReference>,
    minimize: bool,
}

impl<'a> PlainTextSphinxInventoryWriter<'a> {
    #[must_use]
    pub fn from_header(header: &'a InventoryHeader, capacity: usize, minimize: bool) -> Self {
        Self {
            header,
            buffer: Vec::with_capacity(capacity),
            minimize,
        }
    }

    pub fn add_reference(&mut self, reference: &'a SphinxReference) {
        self.buffer.push(reference);
    }

    pub fn finalize<W: Write>(self, writer: &mut W) -> Result<(), std::io::Error> {
        writer.write_all(format!("{}", self.header).as_bytes())?;
        for reference in self.buffer {
            if self.minimize {
                writer.write_all(format!("{}\n", reference.fmt_minified()).as_bytes())?;
            } else {
                writer.write_all(format!("{}\n", reference.fmt_expanded()).as_bytes())?;
            }
        }
        Ok(())
    }
}

pub struct SphinxInventoryWriter<'a> {
    header: &'a InventoryHeader,
    buffer: Vec<&'a SphinxReference>,
    minimize: bool,
}

impl<'a> SphinxInventoryWriter<'a> {
    pub fn from_header(header: &'a InventoryHeader, capacity: usize, minimize: bool) -> Self {
        Self {
            header,
            buffer: Vec::with_capacity(capacity),
            minimize,
        }
    }

    pub fn add_reference(&mut self, reference: &'a SphinxReference) {
        self.buffer.push(reference);
    }

    pub fn finalize<W: Write>(self, writer: &mut W) -> Result<&mut W, std::io::Error> {
        writer.write_all(format!("{}", self.header).as_bytes())?;
        writer.flush()?;
        let mut zlib_writer = ZlibEncoder::new(writer, Compression::fast());
        for reference in self.buffer {
            if self.minimize {
                zlib_writer.write_all(format!("{}\n", reference.fmt_minified()).as_bytes())?;
            } else {
                zlib_writer.write_all(format!("{}\n", reference.fmt_expanded()).as_bytes())?;
            }
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
        InventoryHeader, SphinxInventoryReader, SphinxReference, SphinxType,
        error::SphinxInvError,
        priority::SphinxPriority,
        roles::PyRole,
        writers::{PlainTextSphinxInventoryWriter, SphinxInventoryWriter},
    };

    #[test]
    fn write_simple_plain_text_inventory_minified() -> Result<(), SphinxInvError> {
        let mut write_buffer = Vec::new();
        let expected = String::from(
            "# Sphinx inventory version 2
# Project: foo
# Version: 0.24.24
# The remainder of this file is compressed using zlib.
str.join py:method 1 library/stdtypes.html#$ -
str.lower py:method 1 library/stdtypes.html#$ -
",
        );
        let header = InventoryHeader {
            project_name: "foo".to_string(),
            project_version: "0.24.24".to_string(),
            inventory_version: 2,
            compression_method_description: "zlib".to_string(),
        };

        let str_lower_ref = SphinxReference::new(
            "str.lower",
            SphinxType::Python(PyRole::Method),
            SphinxPriority::Standard,
            "library/stdtypes.html#$",
            "-",
        );

        let str_join_ref = SphinxReference::new(
            "str.join",
            SphinxType::Python(PyRole::Method),
            SphinxPriority::Standard,
            "library/stdtypes.html#$",
            "-",
        );

        let mut writer = PlainTextSphinxInventoryWriter::from_header(&header, 2, true);

        writer.add_reference(&str_join_ref);
        writer.add_reference(&str_lower_ref);

        let mut cursor = Cursor::new(&mut write_buffer);

        writer.finalize(&mut cursor)?;

        assert_eq!(String::from_utf8(write_buffer).unwrap(), expected);
        Ok(())
    }
    #[test]
    fn write_simple_plain_text_inventory_expanded() -> Result<(), SphinxInvError> {
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

        let mut writer = PlainTextSphinxInventoryWriter::from_header(&header, 2, false);

        writer.add_reference(&str_join_ref);
        writer.add_reference(&str_lower_ref);

        let mut cursor = Cursor::new(&mut write_buffer);

        writer.finalize(&mut cursor)?;

        assert_eq!(String::from_utf8(write_buffer).unwrap(), expected);
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
        let mut writer = SphinxInventoryWriter::from_header(&header, 2, true);

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
