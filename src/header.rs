use std::fmt::Display;
use std::io::BufRead;

use winnow::ascii::space0;
use winnow::combinator::{eof, terminated};
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{take_till, take_while};
use winnow::{
    Result as WinnowResult,
    combinator::{preceded, trace},
    error::{StrContext, StrContextValue},
    token::{rest, take_until},
};

use crate::error::{MissingHeaderComponent, SphinxInvError, SphinxParseError};

#[derive(Debug, Clone, PartialEq)]
pub struct InventoryHeader {
    /// the display name of the project this inventory refers to (can contain internal whitespace)
    pub project_name: String,

    /// the version of the project this inventory refers to (should be without a leading v)
    pub project_version: String,

    /// a field for storing and checking the inventory version (should always be 2)
    pub inventory_version: u8,

    /// a field for storing and checking the body compression method (should always be `zlib`)
    pub compression_method_description: String,
}

impl InventoryHeader {
    pub fn new(name: &str, version: &str) -> Self {
        InventoryHeader {
            project_name: name.to_string(),
            project_version: version.to_string(),
            inventory_version: 2,
            compression_method_description: "zlib".to_string(),
        }
    }
}

impl Display for InventoryHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "# Sphinx inventory version {}\n",
            self.inventory_version
        ))?;
        f.write_str(&format!("# Project: {}\n", self.project_name))?;
        f.write_str(&format!("# Version: {}\n", self.project_version))?;
        f.write_str("# The remainder of this file is compressed using zlib.\n")?;
        Ok(())
    }
}

/// Parses the inventory file version from the ascii header part of an inventory file
fn parse_inventory_file_version(buffer: &mut &[u8]) -> WinnowResult<u8> {
    // sphinx itself requires that the first line is exactly
    // # Sphinx inventory version 2
    // but we can be a little more flexible
    trace(
        "inventory version",
        terminated(
            preceded(
                trace("prefix", take_till(1.., AsChar::is_dec_digit)),
                trace("version", take_while(1.., AsChar::is_dec_digit)),
            ),
            (space0, eof)
                .context(StrContext::Label("unexpected extra input"))
                .context(StrContext::Expected(StrContextValue::Description(
                    "line ending",
                ))),
        ),
    )
    .parse_to()
    .parse_next(buffer)
}

/// Parses the project name from the ascii header part of an inventory file
fn parse_project_name(buffer: &mut &[u8]) -> WinnowResult<String> {
    // https://github.com/sphinx-doc/sphinx/blob/ac3f74a3e0fbb326f73989a16dfa369e072064ca/sphinx/util/inventory.py#L126
    // sphinx just takes the first 11 bytes, but we do things slightly different so we can have
    // better error reporting

    trace(
        "project name line",
        preceded(
            trace("project name line prefix", "# Project: ")
                .context(StrContext::Label("project name line prefix"))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "# Project: ",
                ))),
            trace("name", rest),
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
        "project version line",
        preceded(
            trace("project version line prefix", "# Version: ")
                .context(StrContext::Label("project version line prefix"))
                .context(StrContext::Expected(StrContextValue::StringLiteral(
                    "# Version: ",
                ))),
            trace("version", rest),
        ),
    )
    .parse_to()
    .map(|s: String| s.trim().to_owned())
    .parse_next(buffer)
}

fn parse_compression_method(buffer: &mut &[u8]) -> WinnowResult<String> {
    // this is how sphinx itself does it even if it's a bit silly
    trace(
        "compression description",
        terminated(preceded(take_until(0.., "zlib"), "zlib"), rest)
            .context(StrContext::Label("compression method"))
            .context(StrContext::Expected(StrContextValue::StringLiteral("zlib"))),
    )
    .parse_to()
    .verify(|c: &str| !c.is_empty())
    .parse_next(buffer)
}

/// Parses the ascii header part of an inventory file.
///
/// This must look like so:
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
pub fn parse_header<R: BufRead>(reader: &mut R) -> Result<InventoryHeader, SphinxInvError> {
    let mut lines_iter = reader.lines();

    // Currently the API requires that we pass a buffer to the parsing function,
    // and we need the original buffer to display the error so this is required to be split up
    // This will hopefully be addressed in a future iteration of the API
    let inventory_version_line = lines_iter.next().ok_or(SphinxInvError::IncompleteHeader(
        MissingHeaderComponent::InvVersion,
    ))??;

    let inventory_version = parse_inventory_file_version
        .parse(inventory_version_line.as_bytes())
        .map_err(|e| SphinxParseError::from_byte_parse(&e, 1))?;

    let project_name_line = lines_iter.next().ok_or(SphinxInvError::IncompleteHeader(
        MissingHeaderComponent::ProjectName,
    ))??;

    let project_name = parse_project_name
        .parse(project_name_line.as_bytes())
        .map_err(|e| SphinxParseError::from_byte_parse(&e, 2))?;

    let project_version_line = lines_iter.next().ok_or(SphinxInvError::IncompleteHeader(
        MissingHeaderComponent::ProjectVersion,
    ))??;

    let project_version = parse_project_version
        .parse(project_version_line.as_bytes())
        .map_err(|e| SphinxParseError::from_byte_parse(&e, 3))?;

    let compression_method_description_line = lines_iter.next().ok_or(
        SphinxInvError::IncompleteHeader(MissingHeaderComponent::CompressionDescription),
    )??;

    let compression_method_description = parse_compression_method
        .parse(compression_method_description_line.as_bytes())
        .map_err(|e| SphinxParseError::from_byte_parse(&e, 4))?;

    let header = InventoryHeader {
        project_name,
        project_version,
        inventory_version,
        compression_method_description,
    };

    Ok(header)
}

// fn fmt_context_error(input: &str, err: &ContextError) -> String {
//     format!("failed to parse line: {input} because of the following error: {err:#?}")
// }

#[cfg(test)]
mod test {
    use crate::{error::SphinxInvError, header::MissingHeaderComponent};
    use std::io::{BufReader, Cursor};

    use crate::{InventoryHeader, error::SphinxParseError, header::parse_header};

    #[test]
    fn test_numpy_header() -> Result<(), SphinxInvError> {
        let header = "# Sphinx inventory version 2
# Project: NumPy
# Version: 2.3
# The remainder of this file is compressed using zlib.
"
        .as_bytes();

        let mut reader = BufReader::new(Cursor::new(header));

        let header = parse_header(&mut reader)?;

        assert_eq!(header.inventory_version, 2);
        assert_eq!(header.project_name, "NumPy".to_string());
        assert_eq!(header.project_version, "2.3".to_string());
        assert_eq!(header.compression_method_description, "zlib".to_string());
        Ok(())
    }

    #[test]
    fn missing_compression_line() {
        let mut header = "# Sphinx inventory version 3
# Project: asdf
# Version: asdf"
            .as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::IncompleteHeader(
                MissingHeaderComponent::CompressionDescription
            ))
        );
    }
    #[test]
    fn missing_project_line() {
        let mut header = "# Sphinx inventory version 3".as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::IncompleteHeader(
                MissingHeaderComponent::ProjectName
            ))
        );
    }
    #[test]
    fn invalid_compression_descriptor() {
        let mut header = "# Sphinx inventory version 3
# Project: asdfasdf
# Version: ll.3
# The remainder of this file is compressed using my butt."
            .as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::ParseError(SphinxParseError::from_str(
                "# The remainder of this file is compressed using my butt.",
                "invalid compression method\nexpected `zlib`",
                0,
                4
            )))
        );
    }

    #[test]
    fn missing_project_line_only() {
        let mut header = "# Sphinx inventory version 2
# Version: 2.3
# The remainder of this file is compressed using gzip."
            .as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::ParseError(SphinxParseError::from_str(
                "# Version: 2.3",
                "invalid project name line prefix\nexpected `# Project: `",
                0,
                2
            )))
        );
    }
    #[test]
    fn missing_version_line() {
        let mut header = "# Sphinx inventory version 2
# Project: NumPy"
            .as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::IncompleteHeader(
                MissingHeaderComponent::ProjectVersion
            ))
        );
    }
    #[test]
    fn test_no_zlib_header() {
        let mut header = "# Sphinx inventory version 2
# Project: NumPy
# Version: 2.3"
            .as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::IncompleteHeader(
                MissingHeaderComponent::CompressionDescription
            ))
        );
    }

    #[test]
    fn missing_project() {
        let mut header = "".as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::IncompleteHeader(
                MissingHeaderComponent::InvVersion
            ))
        );
    }
    #[test]
    fn empty_buffer() {
        let mut header = "".as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::IncompleteHeader(
                MissingHeaderComponent::InvVersion
            ))
        );
    }
    #[test]
    fn test_unknown_inv_version() {
        let mut header = "# Sphinx inventory version 3.14".as_bytes();

        let result = parse_header(&mut header);
        assert_eq!(
            result,
            Err(SphinxInvError::ParseError(SphinxParseError::from_str(
                "# Sphinx inventory version 3.14",
                "invalid unexpected extra input\nexpected line ending",
                28,
                1
            )))
        );
    }

    #[test]
    fn new_header() {
        assert_eq!(
            InventoryHeader {
                project_name: "foo".to_string(),
                project_version: "0.24.24".to_string(),
                inventory_version: 2,
                compression_method_description: "zlib".to_string()
            },
            InventoryHeader::new("foo", "0.24.24")
        );
    }
}
