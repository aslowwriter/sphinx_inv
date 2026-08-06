use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    error::{ContextError, StrContext},
    stream::AsChar,
    token::take_till,
};

use crate::SphinxType;

#[derive(Debug, PartialEq, Clone)]
pub enum HttpRole {
    Get,
    Delete,
    Put,
    Post,
    Copy,
    Head,
    Any,
}

impl Display for HttpRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            HttpRole::Get => "get",
            HttpRole::Delete => "delete",
            HttpRole::Put => "put",
            HttpRole::Post => "post",
            HttpRole::Copy => "copy",
            HttpRole::Head => "head",
            HttpRole::Any => "any",
        })
    }
}

/// Parses a http role as defined in [`HttpRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn http_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., AsChar::is_space)
        .context(StrContext::Label("http role"))
        .parse_to::<HttpRole>()
        .context(StrContext::Label("expected valid http role"))
        .parse_next(input)?;
    Ok(SphinxType::Http(role))
}
impl FromStr for HttpRole {
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "get" => Ok(HttpRole::Get),
            "delete" => Ok(HttpRole::Delete),
            "put" => Ok(HttpRole::Put),
            "post" => Ok(HttpRole::Post),
            "head" => Ok(HttpRole::Head),
            "copy" => Ok(HttpRole::Copy),
            "any" => Ok(HttpRole::Any),

            _ => Err(ContextError::new()),
        }
    }
}
