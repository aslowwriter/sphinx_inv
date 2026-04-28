use std::fmt::Display;

use crate::error::MalformedReference;

#[derive(Debug, PartialEq)]
pub enum SphinxPriority {
    Omit,
    Standard,
    High,
    Low,
}

impl Display for SphinxPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SphinxPriority::Omit => "-1",
            SphinxPriority::Standard => "1",
            SphinxPriority::High => "0",
            SphinxPriority::Low => "2",
        })
    }
}
impl TryFrom<i32> for SphinxPriority {
    type Error = MalformedReference;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        match value {
            -1 => Ok(SphinxPriority::Omit),
            1 => Ok(SphinxPriority::Standard),
            0 => Ok(SphinxPriority::High),
            2 => Ok(SphinxPriority::Low),
            _ => Err(MalformedReference::InvalidRowPriority(value.to_string())),
        }
    }
}
impl TryFrom<&str> for SphinxPriority {
    type Error = MalformedReference;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "-1" => Ok(SphinxPriority::Omit),
            "1" => Ok(SphinxPriority::Standard),
            "0" => Ok(SphinxPriority::High),
            "2" => Ok(SphinxPriority::Low),
            _ => Err(MalformedReference::InvalidRowPriority(value.to_string())),
        }
    }
}
