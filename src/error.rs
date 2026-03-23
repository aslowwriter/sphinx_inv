use std::string::FromUtf8Error;

use thiserror::Error;

use crate::header::SphinxInvVersion;

#[derive(Error, Debug)]
pub enum SphinxInvError {
    #[error("IO error")]
    IoError(#[from] std::io::Error),

    #[error("invalid UTF-8")]
    Utf8ParseError(#[from] FromUtf8Error),

    #[error("Failed to parse line: {0}")]
    ParseError(#[from] RecordParseError),

    #[error("Failed to parse header: {0}")]
    InvalidHeaderError(#[from] InvalidHeaderError),
}

#[derive(Error, Debug)]
pub enum RecordParseError {
    #[error("Invalid Domain: {0}")]
    InvalidDomain(String),

    #[error("Invalid priority: {0}")]
    InvalidRowPriority(String),

    #[error("Invalid role: {0}")]
    InvalidRole(#[from] strum::ParseError),

    #[error("Malformed domain field: {0}")]
    MalformedDomainField(String),

    #[error("Malformed record: {0}")]
    MalformedRecord(String),
}

#[derive(Error, Debug)]
pub enum InvalidHeaderError {
    #[error("Invalid Sphinx Version")]
    InvalidSphinxVerison(String),

    #[error("Unsupported Sphinx Version")]
    UnsupportedSphinxVersion(SphinxInvVersion),

    #[error("IO error")]
    IoError(#[from] std::io::Error),

    #[error("Invalid compression method")]
    InvalidCompressionMethod(String),
}
