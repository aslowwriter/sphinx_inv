use winnow::{
    Result as WinnowResult,
    ascii::{line_ending, space0},
    combinator::{delimited, terminated, trace},
    stream::AsChar,
    token::{take, take_till, take_while},
};
use winnow::{ascii::till_line_ending, prelude::*};

#[derive(Debug)]
pub struct InventoryHeader {
    pub project_name: String,
    pub project_version: String,
    pub inventory_version: u8,
    pub compression_method_description: String,
}

/// Parses the inventory file version from the ascii header part of an inventory file
fn parse_inventory_file_version(buffer: &mut &[u8]) -> WinnowResult<u8> {
    // sphinx itself requires that the first line is exactly
    // # Sphinx inventory version 2
    // but we can be a little more flexible
    trace(
        "inventory version",
        delimited(
            trace("prefix", take_till(1.., AsChar::is_dec_digit)),
            trace("version", take_while(1.., AsChar::is_dec_digit)),
            trace("trim", (space0, line_ending)),
        ),
    )
    .parse_to()
    .parse_next(buffer)
}

/// Parses the project name from the ascii header part of an inventory file
fn parse_project_name(buffer: &mut &[u8]) -> WinnowResult<String> {
    // this is how sphinx itself does it
    // https://github.com/sphinx-doc/sphinx/blob/ac3f74a3e0fbb326f73989a16dfa369e072064ca/sphinx/util/inventory.py#L126
    trace(
        "project name",
        delimited(
            trace("prefix", take(11usize)),
            trace("name", till_line_ending),
            line_ending,
        ),
    )
    .parse_to()
    .map(|s: String| s.trim().to_owned())
    .parse_next(buffer)
}

/// Parses the version of the project the inventory file refers to
/// from the ascii header part of an inventory file
fn parse_project_version(buffer: &mut &[u8]) -> WinnowResult<String> {
    // this is how sphinx itself does it
    // https://github.com/sphinx-doc/sphinx/blob/ac3f74a3e0fbb326f73989a16dfa369e072064ca/sphinx/util/inventory.py#L126
    trace(
        "project version",
        delimited(take(11usize), till_line_ending, line_ending),
    )
    .parse_to()
    .map(|s: String| s.trim().to_owned())
    .parse_next(buffer)
}

fn parse_compression_method_line(buffer: &mut &[u8]) -> WinnowResult<String> {
    // this is how sphinx itself does it even if it's a bit silly
    trace(
        "compression description",
        terminated(till_line_ending, line_ending),
    )
    .parse_to()
    .verify(|c: &str| !c.is_empty())
    .parse_next(buffer)
}

/// Parses the ascii header part of an inventory file. This must look like so:
/// ```txt
/// # Sphinx inventory file 2
/// # Project: <project display name>
/// # Version: <project version without preceding v>
/// # The rest of this file is comppressed using zlib.
/// ```
///
/// Sphinx itself has some slightly hardcoded rules and we attempt to be
/// slightly more flexible.
///
/// # Errors
/// This function only returns errors on parse errors. This function will not return
/// errors on things such as unknown or unsupported sphinx versions. The caller is
/// responsible for checking those
///
pub fn parse_header(buffer: &mut &[u8]) -> WinnowResult<InventoryHeader> {
    let inventory_version = parse_inventory_file_version(buffer)?;
    let project_name = parse_project_name(buffer)?;
    let project_version = parse_project_version(buffer)?;
    let compression_method_description = parse_compression_method_line(buffer)?;

    let header = InventoryHeader {
        project_name,
        project_version,
        inventory_version,
        compression_method_description,
    };

    Ok(header)
}

#[cfg(test)]
mod test {
    use crate::header::parse_header;
    use winnow::Result;

    #[test]
    fn test_numpy_header() -> Result<()> {
        let mut header = "# Sphinx inventory version 2
# Project: NumPy
# Version: 2.3
# The remainder of this file is compressed using zlib.
"
        .as_bytes();

        let header = parse_header(&mut header)?;

        assert_eq!(header.inventory_version, 2);
        assert_eq!(header.project_name, "NumPy".to_string());
        assert_eq!(header.project_version, "2.3".to_string());
        assert_eq!(
            header.compression_method_description,
            "# The remainder of this file is compressed using zlib.".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_garbange_header() {
        let mut header = "# Sphinx inventory version 3.14...
# Project: asdfasdf
# Version: ll.3
# The remainder of this file is compressed using my butt."
            .as_bytes();

        let result = parse_header(&mut header);
        assert!(result.is_err());
    }

    #[test]
    fn test_incomplete_header() {
        let mut header = "# Sphinx inventory version 2
# Project: NumPy
# Version: 2.3"
            .as_bytes();

        let result = parse_header(&mut header);
        assert!(result.is_err());
    }
    #[test]
    fn test_no_zlib_header() {
        let mut header = "# Sphinx inventory version 2
# Project: NumPy
# Version: 2.3
# The remainder of this file is compressed using gzip."
            .as_bytes();

        let result = parse_header(&mut header);
        assert!(result.is_err());
    }

    #[test]
    fn test_unknown_inv_version() {
        let mut header = "# Sphinx inventory version 3.14".as_bytes();

        let result = parse_header(&mut header);
        assert!(result.is_err());
    }
}
