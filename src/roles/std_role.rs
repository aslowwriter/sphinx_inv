use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    combinator::trace,
    error::{ContextError, StrContext},
    stream::AsChar,
    token::take_till,
};

use crate::roles::SphinxType;

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

impl Display for StdRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            StdRole::Doc => "doc",
            StdRole::Label => "label",
            StdRole::Term => "term",
            StdRole::Cmdoption => "cmdoption",
            StdRole::Pdbcommand => "pdbcommand",
            StdRole::Token => "token",
            StdRole::Opcode => "opcode",
            StdRole::MonitoringEvent => "monitoring-event",
            StdRole::Envvar => "envvar",
        })
    }
}

impl FromStr for StdRole {
    type Err = ContextError;

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

            _ => Err(ContextError::new()),
        }
    }
}

/// Parses a c role as defined in [`StdRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn std_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = trace(
        "std_role",
        take_till(0.., AsChar::is_space).context(StrContext::Label("std role")),
    )
    .parse_to::<StdRole>()
    .parse_next(input)?;
    Ok(SphinxType::Std(role))
}

#[cfg(test)]
mod test {
    use std::str::FromStr;
    use winnow::{ModalResult, Result, error::ContextError};

    use crate::roles::{SphinxType, StdRole, std_role::std_role};

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
    fn test_sphinx_role_parsing_std() -> Result<(), ContextError> {
        assert_eq!(StdRole::from_str("label")?, StdRole::Label);
        assert_eq!(StdRole::from_str("doc")?, StdRole::Doc);
        assert_eq!(StdRole::from_str("term")?, StdRole::Term);
        assert_eq!(StdRole::from_str("token")?, StdRole::Token);
        assert_eq!(StdRole::from_str("cmdoption")?, StdRole::Cmdoption);
        assert_eq!(StdRole::from_str("pdbcommand")?, StdRole::Pdbcommand);
        assert_eq!(StdRole::from_str("opcode")?, StdRole::Opcode);
        assert_eq!(
            StdRole::from_str("monitoring-event")?,
            StdRole::MonitoringEvent
        );
        assert_eq!(StdRole::from_str("envvar")?, StdRole::Envvar);
        Ok(())
    }
}
