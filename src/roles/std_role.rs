use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    combinator::trace,
    error::{ContextError, StrContext},
    stream::AsChar,
    token::take_till,
};

use crate::roles::SphinxType;

#[derive(Debug, PartialEq, Clone)]
pub enum StdRole {
    /// Link to the specified document; [see more](https://www.sphinx-doc.org/en/master/usage/referencing.html#role-doc)
    Doc,

    /// A label, usually for an equasion or figure. [see more](https://www.sphinx-doc.org/en/master/usage/referencing.html#role-doc)
    Label,

    /// Reference to a term in a glossary. [see more](https://www.sphinx-doc.org/en/master/usage/referencing.html#role-term)
    Term,

    /// Describes a command line argument or switch. [see more](https://www.sphinx-doc.org/en/master/usage/domains/standard.html#directive-option)
    Option,

    /// Deprecated alias to [`StdRole::Option`]
    Cmdoption,

    /// A command you can use in a pdb session. [see more](https://docs.python.org/3/library/pdb.html#debugger-commands)
    Pdbcommand,

    /// The name of a grammar token [see more](https://www.sphinx-doc.org/en/master/usage/referencing.html#role-token)
    Token,

    /// numeric byte code referring to a python operation [see more](https://docs.python.org/3/library/dis.html#dis.Instruction.opcode)
    Opcode,

    /// I can't actually find any docs for this or where it came
    /// from so... TBC?
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
            StdRole::Cmdoption | StdRole::Option => "cmdoption",
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
