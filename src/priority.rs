use std::{fmt::Display, str::FromStr};

use winnow::error::ContextError;

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
impl FromStr for SphinxPriority {
    type Err = ContextError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "-1" => Ok(SphinxPriority::Omit),
            "1" => Ok(SphinxPriority::Standard),
            "0" => Ok(SphinxPriority::High),
            "2" => Ok(SphinxPriority::Low),
            _ => Err(ContextError::new()),
        }
    }
}
