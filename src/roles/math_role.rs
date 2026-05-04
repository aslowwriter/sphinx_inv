use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    error::{ContextError, StrContext},
    stream::AsChar,
    token::take_till,
};

use crate::roles::SphinxType;

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
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "numref" => Ok(MathRole::Numref),
            // this is only really necessary to communicate with the parser
            // so we don't have to communicate more than "it failed"
            // as this should never happen
            _ => Err(ContextError::new()),
        }
    }
}

/// Parses a math role as defined in [`MathRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn math_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., AsChar::is_space)
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
        assert!(MathRole::from_str("asdf").is_err());
        assert!(MathRole::from_str("doc").is_err());
        assert!(MathRole::from_str("").is_err());
        assert!(MathRole::from_str("::::").is_err());
        assert!(MathRole::from_str(" label").is_err());
        assert!(MathRole::from_str(" asdf").is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_math() -> Result<(), ContextError> {
        assert_eq!(MathRole::from_str("numref")?, MathRole::Numref);
        Ok(())
    }

    #[test]
    fn render() {}
}
