use flate2::bufread::ZlibDecoder;
use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use crate::{
    error::{RecordParseError, SphinxInvError},
    header::parse_sphinx_inv_header,
    types::ExternalSphinxRef,
};

fn decompress_remaining_zlib_data<R: Read>(
    reader: &mut BufReader<R>,
) -> Result<String, SphinxInvError> {
    // Read the rest of the file and decompress it using zlib
    let mut compressed_data = Vec::new();
    reader.read_to_end(&mut compressed_data)?;

    let mut decoder = ZlibDecoder::new(&compressed_data[..]);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data)?;
    let decompressed_text = String::from_utf8(decompressed_data)?;
    Ok(decompressed_text)
}

pub fn parse_objects_inv<R: Read>(
    mut reader: BufReader<R>,
) -> Result<Vec<ExternalSphinxRef>, SphinxInvError> {
    let _header = parse_sphinx_inv_header(&mut reader)?;

    let decompressed = decompress_remaining_zlib_data(&mut reader)?;

    let refs = decompressed
        .lines()
        .map(ExternalSphinxRef::try_from)
        .collect::<Result<Vec<ExternalSphinxRef>, RecordParseError>>()?;

    Ok(refs)
}

pub fn parse_objects_inv_file(path: &Path) -> Result<Vec<ExternalSphinxRef>, SphinxInvError> {
    let file = File::open(path)?;
    let reader = BufReader::new(&file);
    parse_objects_inv(reader)
}

#[cfg(test)]
mod test {
    use assert_fs::TempDir;
    use std::fs::File;
    use std::io::{BufReader, Write};
    use std::path::PathBuf;

    use crate::error::SphinxInvError;
    use crate::header::SphinxInvVersion;
    use crate::inv_file::{
        decompress_remaining_zlib_data, parse_objects_inv_file, parse_sphinx_inv_header,
    };
    use crate::types::ExternalSphinxRef;

    fn write_test_header(header: &str) -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.join("header_file.inv");
        let mut file = File::create(&path).unwrap();
        file.write_all(header.as_bytes()).unwrap();

        (temp_dir, path)
    }

    #[test]
    fn test_numpy_header() -> Result<(), SphinxInvError> {
        let header = "# Sphinx inventory version 2
# Project: NumPy
# Version: 2.3
# The remainder of this file is compressed using zlib.";

        let (_temp_dir, header_path) = write_test_header(header);
        let file = File::open(header_path)?;
        let mut reader = BufReader::new(&file);
        let header = parse_sphinx_inv_header(&mut reader)?;
        assert_eq!(header.sphinx_version, SphinxInvVersion::V2);
        assert_eq!(header.project_name, "NumPy".to_string());
        assert_eq!(header.project_version, "2.3".to_string());
        Ok(())
    }

    #[test]
    fn test_garbange_header() -> Result<(), SphinxInvError> {
        let header = "# Sphinx inventory version 3.14...
# Project: asdfasdf
# Version: ll.3
# The remainder of this file is compressed using my butt.";

        let (_temp_dir, header_path) = write_test_header(header);
        let file = File::open(header_path)?;
        let mut reader = BufReader::new(&file);
        let result = parse_sphinx_inv_header(&mut reader);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_short_header() -> Result<(), SphinxInvError> {
        let header = "# Sphinx inventory version 2
# Project: NumPy
# Version: 2.3";

        let (_temp_dir, header_path) = write_test_header(header);
        let file = File::open(header_path)?;
        let mut reader = BufReader::new(&file);
        let result = parse_sphinx_inv_header(&mut reader);
        assert!(result.is_err());
        Ok(())
    }
    #[test]
    fn test_no_zlib_header() -> Result<(), SphinxInvError> {
        let header = "# Sphinx inventory version 2
# Project: NumPy
# Version: 2.3
# The remainder of this file is compressed using gzip.";

        let (_temp_dir, header_path) = write_test_header(header);
        let file = File::open(header_path)?;
        let mut reader = BufReader::new(&file);
        let result = parse_sphinx_inv_header(&mut reader);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_old_inv_version() -> Result<(), SphinxInvError> {
        let header = "# Sphinx inventory version 1";

        let (_temp_dir, header_path) = write_test_header(header);
        let file = File::open(header_path)?;
        let mut reader = BufReader::new(&file);
        let result = parse_sphinx_inv_header(&mut reader);
        assert!(result.is_err(),);
        Ok(())
    }

    #[test]
    fn test_unknown_inv_version() -> Result<(), SphinxInvError> {
        let header = "# Sphinx inventory version 3.14";

        let (_temp_dir, header_path) = write_test_header(header);
        let file = File::open(header_path)?;
        let mut reader = BufReader::new(&file);
        let header = parse_sphinx_inv_header(&mut reader);
        assert!(header.is_err());
        Ok(())
    }
    #[test]
    fn test_garbage_inv_version() -> Result<(), SphinxInvError> {
        let header = "# asdfasdfsadf";

        let (_temp_dir, header_path) = write_test_header(header);
        let file = File::open(header_path)?;
        let mut reader = BufReader::new(&file);
        let header = parse_sphinx_inv_header(&mut reader);
        assert!(header.is_err());
        Ok(())
    }

    #[test]
    fn load_numpy_file_manually() -> Result<(), SphinxInvError> {
        let filename = "tests/sphinx_objects/numpy.inv";
        let file = File::open(filename)?;
        let mut reader = BufReader::new(&file);

        let header = parse_sphinx_inv_header(&mut reader)?;
        assert_eq!(header.sphinx_version, SphinxInvVersion::V2);
        assert_eq!(header.project_name, "NumPy".to_string());
        assert_eq!(header.project_version, "2.3".to_string());

        let decompressed = decompress_remaining_zlib_data(&mut reader)?;

        for line in decompressed.lines() {
            let sphinx_ref = ExternalSphinxRef::try_from(line);
            assert!(sphinx_ref.is_ok(), "failed to parse line: {line}");
        }

        Ok(())
    }

    #[test]
    fn load_numpy_file_pub_func() -> Result<(), SphinxInvError> {
        let filename = PathBuf::from("tests/sphinx_objects/numpy.inv");
        let _ = parse_objects_inv_file(&filename)?;

        Ok(())
    }
}
