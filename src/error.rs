use std::string::FromUtf8Error;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SphinxInvError {
    #[error("IO error")]
    IoError(#[from] std::io::Error),

    #[error("invalid UTF-8")]
    Utf8ParseError(#[from] FromUtf8Error),

    #[error("Failed to parse line: {0}")]
    ParseError(String),

    #[error("Malformed Header")]
    MalformedHeader,

    #[error("Unsupported inventory version: {0}")]
    UnsupportedInventoryVersion(u8),

    #[error("Unsupported compression method: {0}")]
    UnsupportedCompressionMethod(String),
}

#[derive(Error, Debug)]
pub enum RecordParseError {
    #[error("Invalid Domain: {0}")]
    InvalidDomain(String),

    #[error("Invalid priority: {0}")]
    InvalidRowPriority(String),

    #[error("Invalid role: {0}")]
    InvalidRole(String),

    #[error("Malformed domain field: {0}")]
    MalformedDomainField(String),

    #[error("Malformed type: {0}")]
    MalformedType(String),

    #[error("Malformed record: {0}")]
    MalformedRecord(String),
}
