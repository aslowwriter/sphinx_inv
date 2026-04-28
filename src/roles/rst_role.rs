use std::{fmt::Display, str::FromStr};

use winnow::{ModalResult, Parser, error::StrContext, stream::AsChar, token::take_till};

use crate::roles::{MalformedReference, SphinxType};

/// Describes a RST role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `rst:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq)]
pub enum RstRole {
    /// Describes a reStructuredText directive.
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/restructuredtext.html#directive-rst-directive)
    Directive,

    /// Describes an option for a reStructuredText directive
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/restructuredtext.html#directive-rst-directive-option)
    Option,
}

impl Display for RstRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            RstRole::Directive => "directive",
            RstRole::Option => "directive:option",
        })
    }
}

/// Parses a cpp role as defined in [`RstRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn rst_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., |c| AsChar::is_space(c) || AsChar::is_newline(c))
        .context(StrContext::Label("rst role"))
        .parse_to::<RstRole>()
        .parse_next(input)?;
    Ok(SphinxType::ReStructuredText(role))
}
impl FromStr for RstRole {
    type Err = MalformedReference;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "directive:option" => Ok(RstRole::Option),
            "directive" => Ok(RstRole::Directive),

            _ => Err(MalformedReference::InvalidRole(s.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::roles::{RstRole, SphinxType, rst_role};

    #[test]
    fn rst_parsing() {
        let mut line = "directive";
        assert_eq!(
            rst_role(&mut line),
            Ok(SphinxType::ReStructuredText(RstRole::Directive))
        );

        line = "directive:option";
        assert_eq!(
            rst_role(&mut line),
            Ok(SphinxType::ReStructuredText(RstRole::Option))
        );
    }
}
