use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    error::{ContextError, StrContext},
    stream::AsChar,
    token::take_till,
};

use crate::roles::SphinxType;

/// Describes a C++ role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `cpp:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq)]
pub enum CppRole {
    /// a C++ enum definition
    /// cee also [the C++ reference](https://en.cppreference.com/cpp/language/enum)
    Enum,

    /// A C++ which seems to be the same as a
    Enumerator,

    /// Describes a C++ class
    /// Sphinx considers this to be equivalent to `cpp:struct`
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/cpp.html#directive-cpp-class)
    Class,

    Union,

    Concept,

    /// Describes a C++ function, which may also be a method on a class
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/cpp.html#directive-cpp-function)
    Function,

    /// Describes a C++ function parameter
    /// No documentation seems to be available for this yet
    FunctionParam,

    /// Describes a C++ Variable or member declaration
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/cpp.html#directive-cpp-member)
    Member,

    /// Describes a C+! template parameter
    /// No documentation seems to be available for this yet
    TemplateParam,

    /// Desxcribes a C++ type declaration or alias
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/cpp.html#directive-cpp-member)
    Type,
}

impl Display for CppRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CppRole::Enumerator => "enumerator",
            CppRole::Enum => "enum",
            CppRole::Class => "class",
            CppRole::Function => "function",
            CppRole::FunctionParam => "functionParam",
            CppRole::Member => "member",
            CppRole::TemplateParam => "templateParam",
            CppRole::Type => "type",
            CppRole::Union => "union",
            CppRole::Concept => "concept",
        })
    }
}

impl FromStr for CppRole {
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "class" => Ok(CppRole::Class),
            "enumerator" => Ok(CppRole::Enumerator),
            "enum" => Ok(CppRole::Enum),
            "function" => Ok(CppRole::Function),
            "functionParam" => Ok(CppRole::FunctionParam),
            "member" => Ok(CppRole::Member),
            "templateParam" => Ok(CppRole::TemplateParam),
            "concept" => Ok(CppRole::Concept),
            "union" => Ok(CppRole::Union),
            "type" => Ok(CppRole::Type),

            // this is only really necessary to communicate with the parser
            // so we don't have to communicate more than "it failed"
            // as this should never happen
            _ => Err(ContextError::new()),
        }
    }
}

/// Parses a cpp role as defined in [`CppRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn cpp_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., AsChar::is_space)
        .context(StrContext::Label("Cpp Role"))
        .parse_to()
        .parse_next(input)?;
    Ok(SphinxType::Cpp(role))
}
