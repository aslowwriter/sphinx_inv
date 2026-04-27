use std::str::FromStr;

use winnow::{ModalResult, Parser, combinator::trace, stream::AsChar, token::take_till};

use crate::roles::{MalformedReference, SphinxType};

#[derive(Debug, PartialEq)]
pub enum StdRole {
    Doc,

    Label,

    Term,

    Cmdoption,

    Pdbcommand,

    Token,

    Opcode,

    MonitoringEvent,

    /// Describes an environment variable that the documented code
    /// or program uses or defines
    Envvar,
}

impl FromStr for StdRole {
    type Err = MalformedReference;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "doc" => Ok(StdRole::Doc),
            "label" => Ok(StdRole::Label),
            "term" => Ok(StdRole::Term),
            "cmdoption" => Ok(StdRole::Cmdoption),
            "pdbcommand" => Ok(StdRole::Pdbcommand),
            "opcode" => Ok(StdRole::Opcode),
            "token" => Ok(StdRole::Token),
            "monitoring-event" => Ok(StdRole::MonitoringEvent),
            "envvar" => Ok(StdRole::Envvar),

            _ => Err(MalformedReference::InvalidRole(s.to_string())),
        }
    }
}

impl TryFrom<&str> for StdRole {
    type Error = MalformedReference;
    fn try_from(value: &str) -> std::result::Result<StdRole, Self::Error> {
        StdRole::from_str(value)
    }
}

impl From<StdRole> for SphinxType {
    fn from(value: StdRole) -> Self {
        SphinxType::Std(value)
    }
}

/// Parses a c role as defined in [`StdRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn std_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = trace(
        "std role",
        take_till(0.., |c| AsChar::is_space(c) || AsChar::is_newline(c)),
    )
    .parse_to::<StdRole>()
    .parse_next(input)?;
    Ok(SphinxType::Std(role))
}

#[cfg(test)]
mod test {
    use winnow::{ModalResult, Result};

    use crate::{
        error::MalformedReference,
        roles::{SphinxType, StdRole, std_role::std_role},
    };

    #[test]
    fn test_sphinx_type_parsing_std_err() {
        let mut line = "larel ";
        assert!(std_role(&mut line).is_err());

        line = " ";
        assert!(std_role(&mut line).is_err());

        line = ":::";
        assert!(std_role(&mut line).is_err());

        line = "";
        assert!(std_role(&mut line).is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_std() -> ModalResult<()> {
        let mut line = "label ";
        assert_eq!(std_role(&mut line)?, SphinxType::Std(StdRole::Label));

        line = "doc";
        assert_eq!(std_role(&mut line)?, SphinxType::Std(StdRole::Doc));

        Ok(())
    }

    #[test]
    fn test_sphinx_role_parsing_std() -> Result<(), MalformedReference> {
        assert_eq!(
            SphinxType::try_from("std:doc")?,
            SphinxType::Std(StdRole::Doc)
        );
        assert_eq!(StdRole::try_from("label")?, StdRole::Label);
        assert_eq!(StdRole::try_from("term")?, StdRole::Term);
        assert_eq!(StdRole::try_from("token")?, StdRole::Token);
        assert_eq!(StdRole::try_from("cmdoption")?, StdRole::Cmdoption);
        assert_eq!(StdRole::try_from("pdbcommand")?, StdRole::Pdbcommand);
        assert_eq!(StdRole::try_from("opcode")?, StdRole::Opcode);
        assert_eq!(
            StdRole::try_from("monitoring-event")?,
            StdRole::MonitoringEvent
        );
        assert_eq!(StdRole::try_from("envvar")?, StdRole::Envvar);
        Ok(())
    }
}
