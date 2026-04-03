use std::str::FromStr;

use winnow::{ModalResult, Parser, error::StrContext, stream::AsChar, token::take_till};

use crate::{error::RecordParseError, roles::SphinxType};

/// Describes a C++ role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `cpp:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq)]
pub enum CppRole {
    /// Describes a C++ class
    /// Sphinx considers this to be equivalent to `cpp:struct`
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/cpp.html#directive-cpp-class)
    Class,

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

impl FromStr for CppRole {
    type Err = RecordParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "class" => Ok(CppRole::Class),
            "function" => Ok(CppRole::Function),
            "functionParam" => Ok(CppRole::FunctionParam),
            "member" => Ok(CppRole::Member),
            "templateParam" => Ok(CppRole::TemplateParam),

            _ => Err(RecordParseError::InvalidRole(s.to_string())),
        }
    }
}

impl TryFrom<&str> for CppRole {
    type Error = RecordParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

/// Parses a cpp role as defined in [`CppRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn cpp_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., |c| AsChar::is_space(c) || AsChar::is_newline(c))
        .context(StrContext::Label("Cpp Role"))
        .parse_to()
        .parse_next(input)?;
    Ok(SphinxType::Cpp(role))
}

#[cfg(test)]
mod test {

    use crate::error::RecordParseError;

    use super::*;
    #[test]
    fn test_sphinx_role_parsing_std_err() {
        assert!(CppRole::try_from("asdf").is_err());
        assert!(CppRole::try_from("doc").is_err());
        assert!(CppRole::try_from("").is_err());
        assert!(CppRole::try_from("::::").is_err());
        assert!(CppRole::try_from(" label").is_err());
        assert!(CppRole::try_from(" asdf").is_err());
        assert!(CppRole::try_from("function Param").is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_cpp() -> Result<(), RecordParseError> {
        assert_eq!(CppRole::try_from("class")?, CppRole::Class);
        assert_eq!(CppRole::try_from("function")?, CppRole::Function);
        assert_eq!(CppRole::try_from("functionParam")?, CppRole::FunctionParam);
        assert_eq!(CppRole::try_from("templateParam")?, CppRole::TemplateParam);
        assert_eq!(CppRole::try_from("member")?, CppRole::Member);
        Ok(())
    }
}
