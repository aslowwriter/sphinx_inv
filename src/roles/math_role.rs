use std::{fmt::Display, str::FromStr};

use winnow::{ModalResult, Parser, Result, error::StrContext, stream::AsChar, token::take_till};

use crate::{error::MalformedReference, roles::SphinxType};

/// Describes a Mathematics role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `math:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq)]
pub enum MathRole {
    /// Role for cross-referencing equations defined by math directive via their label
    /// can also refer to figures
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/mathematics.html)
    Numref,
}
impl Display for MathRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            MathRole::Numref => "numref",
        })
    }
}
impl FromStr for MathRole {
    type Err = MalformedReference;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "numref" => Ok(MathRole::Numref),

            _ => Err(MalformedReference::InvalidRole(s.to_string())),
        }
    }
}
impl TryFrom<&str> for MathRole {
    type Error = MalformedReference;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

/// Parses a math role as defined in [`MathRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn math_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., |c| AsChar::is_space(c) || AsChar::is_newline(c))
        .context(StrContext::Label("Math Role"))
        .parse_to()
        .parse_next(input)?;
    Ok(SphinxType::Mathematics(role))
}

#[cfg(test)]
mod test {

    use super::*;
    #[test]
    fn test_sphinx_type_parsing_std_err() {
        let mut line = "larel ";
        assert!(math_role(&mut line).is_err());

        line = " ";
        assert!(math_role(&mut line).is_err());

        line = ":::";
        assert!(math_role(&mut line).is_err());

        line = "";
        assert!(math_role(&mut line).is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_std() -> ModalResult<()> {
        let mut line = "numref ";
        assert_eq!(
            math_role(&mut line)?,
            SphinxType::Mathematics(MathRole::Numref)
        );

        Ok(())
    }
    #[test]
    fn test_sphinx_role_parsing_std_err() {
        assert!(MathRole::try_from("asdf").is_err());
        assert!(MathRole::try_from("doc").is_err());
        assert!(MathRole::try_from("").is_err());
        assert!(MathRole::try_from("::::").is_err());
        assert!(MathRole::try_from(" label").is_err());
        assert!(MathRole::try_from(" asdf").is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_math() -> Result<(), MalformedReference> {
        assert_eq!(MathRole::try_from("numref")?, MathRole::Numref);
        Ok(())
    }

    #[test]
    fn render() {}
}
