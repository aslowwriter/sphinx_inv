use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    error::{ContextError, StrContext},
    stream::AsChar,
    token::take_till,
};

use crate::roles::SphinxType;

/// Describes a C role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `c:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq, Clone)]
pub enum CRole {
    /// Describes a C enumerator
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-enumerator)
    Enumerator,

    /// Describes a C enum
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-function)
    Enum,

    /// Describes a C function with a signature as written in C
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-function)
    Function,

    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-function)
    FunctionParam,

    /// A C variable, is an alias for [`CRole::Var`]
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-member)
    Member,

    /// Describes a C macro, i.e. a `#define` without the replacement text
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-macro)
    Macro,

    /// A C variable, equivalent to [`CRole::Member`]
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-var)
    Var,

    /// Describes a C type as defined by either a `typedef` or alias
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-type)
    Type,

    /// Describes a C struct
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-struct)
    Struct,

    /// Describes a C union
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/c.html#directive-c-union)
    Union,
}
impl Display for CRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CRole::Enumerator => "enumerator",
            CRole::Enum => "enum",
            CRole::Function => "function",
            CRole::FunctionParam => "functionParam",
            CRole::Member => "member",
            CRole::Macro => "macro",
            CRole::Var => "var",
            CRole::Type => "type",
            CRole::Struct => "struct",
            CRole::Union => "union",
        })
    }
}

impl FromStr for CRole {
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "enumerator" => Ok(CRole::Enumerator),
            "enum" => Ok(CRole::Enum),
            "function" => Ok(CRole::Function),
            "functionParam" => Ok(CRole::FunctionParam),
            "member" => Ok(CRole::Member),
            "macro" => Ok(CRole::Macro),
            "var" => Ok(CRole::Var),
            "type" => Ok(CRole::Type),
            "struct" => Ok(CRole::Struct),
            "union" => Ok(CRole::Union),

            // this is only really necessary to communicate with the parser
            // so we don't have to communicate more than "it failed"
            // as this should never happen
            _ => Err(ContextError::new()),
        }
    }
}

/// Parses a c role as defined in [`CRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn c_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., AsChar::is_space)
        .context(StrContext::Label("c role"))
        .parse_to()
        .parse_next(input)?;
    Ok(SphinxType::C(role))
}
