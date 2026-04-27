use crate::error::MalformedReference;

#[derive(Debug, PartialEq)]
pub enum SphinxPriority {
    Omit,
    Standard,
    High,
    Low,
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
