use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    error::{ContextError, StrContext},
    stream::AsChar,
    token::take_till,
};

use crate::roles::SphinxType;

/// Describes a sip role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `sip:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq)]
pub enum SipRole {
    Method,
    Member,
    Class,
    Attribute,
    Enum,
    Module,
    Signal,
}
impl Display for SipRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            SipRole::Method => "method",
            SipRole::Member => "member",
            SipRole::Class => "class",
            SipRole::Signal => "signal",
            SipRole::Module => "module",
            SipRole::Attribute => "attribute",
            SipRole::Enum => "enum",
        })
    }
}

impl FromStr for SipRole {
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "method" => Ok(SipRole::Method),
            "member" => Ok(SipRole::Member),
            "class" => Ok(SipRole::Class),
            "signal" => Ok(SipRole::Signal),
            "module" => Ok(SipRole::Module),
            "attribute" => Ok(SipRole::Attribute),
            "enum" => Ok(SipRole::Enum),

            // this is only really necessary to communicate with the parser
            // so we don't have to communicate more than "it failed"
            // as this should never happen
            _ => Err(ContextError::new()),
        }
    }
}

/// may not contain whitespace but may contain other colons
pub(crate) fn sip_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., AsChar::is_space)
        .context(StrContext::Label("sip role"))
        .parse_to()
        .parse_next(input)?;
    Ok(SphinxType::Sip(role))
}

#[cfg(test)]
mod test {

    use crate::roles::sip_role::{SipRole, sip_role};
    use winnow::ModalResult;

    use crate::roles::SphinxType;

    #[test]
    fn invalid_sip_role() {
        let mut input = "asdfasdf";

        let result = sip_role(&mut input);
        assert!(result.is_err());
    }
    #[test]
    fn c_role_enum() -> ModalResult<()> {
        let mut input = "method";

        let role = sip_role(&mut input)?;

        assert_eq!(role, SphinxType::Sip(SipRole::Method));
        Ok(())
    }
}
