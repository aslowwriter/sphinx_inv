use std::io::{BufWriter, Write};

use flate2::{Compression, write::ZlibEncoder};

use crate::{InventoryHeader, SphinxReference};

#[derive(Debug, PartialEq, Clone)]
pub enum WriteFormat {
    Plain,
    Zlib,
}

pub struct SphinxInventoryWriter {
    header: InventoryHeader,
    buffer: Vec<SphinxReference>,
}

impl SphinxInventoryWriter {
    pub fn from_header(header: InventoryHeader, capacity: usize) -> Self {
        Self {
            header,
            buffer: Vec::with_capacity(capacity),
        }
    }

    pub fn add_reference(&mut self, reference: SphinxReference) {
        self.buffer.push(reference);
    }

    pub fn finalize<W: Write>(
        mut self,
        writer: &mut W,
        format: &WriteFormat,
        minimize: bool,
    ) -> Result<(), std::io::Error> {
        let mut writer = BufWriter::new(writer);
        self.header.compression_method_description = match format {
            WriteFormat::Plain => "plain-text".to_string(),
            WriteFormat::Zlib => "zlib".to_string(),
        };
        writer.write_all(self.header.to_string().as_bytes())?;
        match format {
            WriteFormat::Plain => {
                if minimize {
                    for reference in self.buffer {
                        writeln!(writer, "{}", reference.fmt_minified())?;
                    }
                } else {
                    for reference in self.buffer {
                        writeln!(writer, "{}", reference.fmt_expanded())?;
                    }
                }
                Ok(())
            }
            WriteFormat::Zlib => {
                let mut zlib_writer = ZlibEncoder::new(writer, Compression::fast());
                if minimize {
                    for reference in self.buffer {
                        writeln!(zlib_writer, "{}", reference.fmt_minified())?;
                    }
                } else {
                    for reference in self.buffer {
                        writeln!(zlib_writer, "{}", reference.fmt_expanded())?;
                    }
                }
                zlib_writer.finish()?;
                Ok(())
            }
        }
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
        writers::{SphinxInventoryWriter, WriteFormat},
    };

    #[test]
    fn write_simple_plain_text_inventory_minified() -> Result<(), SphinxInvError> {
        let mut write_buffer = Vec::new();
        let expected = String::from(
            "# Sphinx inventory version 2
# Project: foo
# Version: 0.24.24
# The remainder of this file is compressed using plain-text.
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

        let mut writer = SphinxInventoryWriter::from_header(header, 2);

        writer.add_reference(str_join_ref);
        writer.add_reference(str_lower_ref);

        let mut cursor = Cursor::new(&mut write_buffer);

        writer.finalize(&mut cursor, &WriteFormat::Plain, true)?;

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
# The remainder of this file is compressed using plain-text.
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

        let mut writer = SphinxInventoryWriter::from_header(header, 2);

        writer.add_reference(str_join_ref);
        writer.add_reference(str_lower_ref);

        let mut cursor = Cursor::new(&mut write_buffer);

        writer.finalize(&mut cursor, &WriteFormat::Plain, false)?;

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
            compression_method_description: "plain-text".to_string(),
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
        let mut writer = SphinxInventoryWriter::from_header(header.clone(), 2);

        writer.add_reference(str_join_ref.clone());
        writer.add_reference(str_lower_ref.clone());
        writer.finalize(&mut cursor, &WriteFormat::Plain, true)?;

        cursor.set_position(0);

        let mut reader = SphinxInventoryReader::from_reader(cursor)?;

        assert_eq!(reader.header(), &header);

        assert_eq!(reader.next().unwrap()?, str_join_ref);

        assert_eq!(reader.next().unwrap()?, str_lower_ref);

        Ok(())
    }
}
