use std::fmt::Display;

use thiserror::Error;
use winnow::error::{ContextError, ParseError};

#[derive(Error, Debug)]
pub enum SphinxInvError {
    /// Errors that originate while trying to read/write to the underlying Reader/Writer
    #[error("IO error")]
    IoError(#[from] std::io::Error),

    /// Errors while parsing Sphinx input (can be either during header or record parsing)
    #[error("Parse error: {0}")]
    ParseError(#[from] SphinxParseError),

    /// Errors when there is not enough input to correctly parse a header, note that
    /// this is different from failing to parse malformed header lines
    /// for that see [`SphinxInvError::ParseError`]
    /// This error only occurs when the reader does not provide enough output
    #[error("Input was missing the following header component: {0}")]
    IncompleteHeader(MissingHeaderComponent),

    /// When the inventory version is not one that we can parse (e.g. Sphinx inventory v1)
    #[error("Unsupported inventory version: {0}")]
    UnsupportedInventoryVersion(u8),

    /// When the declared compression of the body is one that is not supported
    /// currently `zlib` is the only supported one
    /// note that this does NOT look at the body itself to try and figure out what
    /// compression was used. this is entirely based on what the header line mentions
    /// (in accordance with sphinx itself)
    #[error("Unsupported compression method: {0}")]
    UnsupportedCompressionMethod(String),
}

/// This error occurs when the underlying reader does not provide enough
/// input to parse a header correctly
/// Note that this is purely based on how many lines have been read from the reader
/// and the order in which they were, and does not occur when said lines cannot be
/// parsed correctly, for that see [`SphinxInvError::ParseError`]
#[derive(Error, Debug, PartialEq)]
pub enum MissingHeaderComponent {
    /// There was no input to be read from the reader
    #[error("inventory format version")]
    InvVersion,

    /// There was not enough input to be read from the reader
    /// next line expected was: `# Project: <project name>`
    #[error("project name")]
    ProjectName,

    /// There was not enough input to be read from the reader
    /// next line expected was: `# Version: <project name>`
    #[error("project version")]
    ProjectVersion,

    /// There was not enough input to be read from the reader
    /// next line expected was: `# The remainder of this file is compressed with zlib.`
    #[error("compression description")]
    CompressionDescription,
}

#[derive(Error, Debug, PartialEq)]
pub struct SphinxParseError {
    pub input: String,
    pub message: String,
    pub location: usize,
    pub line_num: usize,
}

impl SphinxParseError {
    /// A convenience function to create parse errors, mostly used for testing.
    pub fn from_str(line: &str, message: &str, location: usize, line_num: usize) -> Self {
        Self {
            input: line.to_string(),
            message: message.to_string(),
            location,
            line_num,
        }
    }
}

/// Default implementation for parse errors
/// might also offer one with e.g. annotate snippets in the future
impl Display for SphinxParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Parsing error on line {0}: \n", self.line_num))?;
        f.write_str(&format!("{}\n", &self.input))?;
        f.write_str(&format!("{}^\n", " ".repeat(self.location)))?;
        f.write_str(&self.message)
    }
}

impl SphinxParseError {
    pub fn from_byte_parse(error: &ParseError<&[u8], ContextError>, line_num: usize) -> Self {
        // The default renderer for `ContextError` is still used but that can be
        // customized as well to better fit our needs.
        let message = error.inner().to_string();
        let byte_buf = (*error.input()).to_owned();
        let line = match String::from_utf8(byte_buf.clone()) {
            Ok(s) => s,
            Err(_) => format!("{byte_buf:?}"),
        };

        // Assume the error span is only for the first `char`.
        let location = error.offset();
        Self {
            input: line,
            message,
            location,
            line_num,
        }
    }
    // Avoiding `From` so `winnow` types don't become part of our public API
    pub fn from_str_parse(error: &ParseError<&str, ContextError>, line_num: usize) -> Self {
        // The default renderer for `ContextError` is still used for now but that can be
        // customized as well to better fit our needs.
        let message = error.inner().to_string();
        let line = *error.input();
        // Assume the error span is only for the first `char`.
        let location = error.offset();
        Self {
            input: line.to_string(),
            message,
            location,
            line_num,
        }
    }
}

// This is behind a cfg(test) because the semantics of one IoError being equal to another
// doesn't technically make sense, and soI don't want to expose this to the user, but it is
// really useful for testing
#[cfg(test)]
impl PartialEq for SphinxInvError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::IoError(_), Self::IoError(_)) => true,
            (Self::ParseError(l0), Self::ParseError(r0)) => l0 == r0,
            (Self::UnsupportedInventoryVersion(l0), Self::UnsupportedInventoryVersion(r0)) => {
                l0 == r0
            }
            (Self::UnsupportedCompressionMethod(l0), Self::UnsupportedCompressionMethod(r0)) => {
                l0 == r0
            }
            (Self::IncompleteHeader(l0), Self::IncompleteHeader(r0)) => l0 == r0,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::error::SphinxParseError;

    #[test]
    fn parse_error_render() {
        let err =
            SphinxParseError::from_str("foo bar baz", "baz is deprecated, please use soup", 10, 0);
        assert_eq!(
            err.to_string(),
            "Parsing error on line 0: \nfoo bar baz\n          ^\nbaz is deprecated, please use soup"
        );
    }
}
