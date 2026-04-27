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
    MalformedHeader(#[from] MalformedHeader),

    #[error("Malformed Header")]
    MalformedReference(#[from] MalformedReference),

    #[error("Unsupported inventory version: {0}")]
    UnsupportedInventoryVersion(u8),

    #[error("Unsupported compression method: {0}")]
    UnsupportedCompressionMethod(String),
}

#[derive(Error, Debug)]
pub enum MissingHeaderComponent {
    #[error("inventory format version")]
    InvVersion,

    #[error("project name")]
    ProjectName,

    #[error("project version")]
    ProjectVersion,

    #[error("compression description")]
    CompressionDescription,
}

#[derive(Error, Debug)]
pub enum MalformedHeader {
    #[error("Input was missing the following header component: {0}")]
    IncompleteHeader(MissingHeaderComponent),

    #[error("Could not parse header line: {0}")]
    ParseError(String),

    #[error("Unsupported inventory version: {0}")]
    UnsupportedInventoryVersion(u8),

    #[error("Unsupported compression method: {0}")]
    UnsupportedCompressionMethod(String),

    #[error("IO error")]
    IoError(#[from] std::io::Error),
}

#[derive(Error, Debug, PartialEq)]
pub enum MalformedReference {
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
