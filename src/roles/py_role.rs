use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    error::{ContextError, StrContext, StrContextValue},
    stream::AsChar,
    token::take_till,
};

use crate::roles::SphinxType;

/// Describes a Python role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `py:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq)]
pub enum PyRole {
    /// Describes an object data attribute
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-attribute)
    Attribute,

    /// References a module-level python variable.
    /// `Type` should be used for type aliases and `Attribute` for class variables
    /// and instance attributes
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-data)
    Data,

    /// A python class describing an exception that can be thrown
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-exception)
    Exception,

    /// Describes a type alias
    /// see also [the sphix docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-type)
    Type,

    /// A module-level Python function
    /// see also [sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-function)
    Function,

    /// A python module meaning a function that is defined on
    /// a class, which traditionally takes `self` as the first
    /// argument
    /// see also [sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-method)
    Method,

    /// A python module, usually corresponding to a file or
    /// directory with a __init__.py file
    /// see also [the sphinx docs](https://docs.python.org/3/tutorial/modules.html)
    Module,

    /// A property of an object such as `abstract`, `abstractmethod`,
    /// `classmethod` etc.
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-property)
    Property,

    /// A python class
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-method)
    Class,
}
impl Display for PyRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PyRole::Attribute => "attribute",
            PyRole::Data => "data",
            PyRole::Exception => "exception",
            PyRole::Type => "type",
            PyRole::Function => "function",
            PyRole::Method => "method",
            PyRole::Module => "module",
            PyRole::Property => "property",
            PyRole::Class => "class",
        })
    }
}
impl FromStr for PyRole {
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "attribute" => Ok(PyRole::Attribute),
            "data" => Ok(PyRole::Data),
            "exception" => Ok(PyRole::Exception),
            "function" => Ok(PyRole::Function),
            "method" => Ok(PyRole::Method),
            "module" => Ok(PyRole::Module),
            "property" => Ok(PyRole::Property),
            "class" => Ok(PyRole::Class),

            _ => Err(ContextError::new()),
        }
    }
}

/// Parses a py role as defined in [`PyRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn py_role(input: &mut &str) -> ModalResult<SphinxType> {
    take_till(0.., AsChar::is_space)
        .parse_to()
        .context(StrContext::Label("python role"))
        .context(StrContext::Expected(StrContextValue::StringLiteral(
            "attribute",
        )))
        .context(StrContext::Expected(StrContextValue::StringLiteral("data")))
        .context(StrContext::Expected(StrContextValue::StringLiteral(
            "exception",
        )))
        .context(StrContext::Expected(StrContextValue::StringLiteral(
            "function",
        )))
        .context(StrContext::Expected(StrContextValue::StringLiteral(
            "method",
        )))
        .context(StrContext::Expected(StrContextValue::StringLiteral(
            "module",
        )))
        .context(StrContext::Expected(StrContextValue::StringLiteral(
            "property",
        )))
        .context(StrContext::Expected(StrContextValue::StringLiteral(
            "class",
        )))
        .map(SphinxType::Python)
        .parse_next(input)
}

#[cfg(test)]
mod test {

    use std::str::FromStr;
    use winnow::ModalResult;
    use winnow::error::ContextError;

    use crate::roles::PyRole;
    use crate::roles::SphinxType;
    use crate::roles::py_role::py_role;

    #[test]
    fn test_sphinx_type_parsing_std_err() {
        let mut line = "larel ";
        assert!(py_role(&mut line).is_err());

        line = " ";
        assert!(py_role(&mut line).is_err());

        line = ":::";
        assert!(py_role(&mut line).is_err());

        line = "";
        assert!(py_role(&mut line).is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_std() -> ModalResult<()> {
        let mut line = "data ";
        assert_eq!(py_role(&mut line)?, SphinxType::Python(PyRole::Data));

        line = "class";
        assert_eq!(py_role(&mut line)?, SphinxType::Python(PyRole::Class));

        Ok(())
    }
    #[test]
    fn test_sphinx_role_parsing_std_err() {
        assert!(PyRole::from_str("asdf").is_err());
        assert!(PyRole::from_str("doc").is_err());
        assert!(PyRole::from_str("").is_err());
        assert!(PyRole::from_str("::::").is_err());
        assert!(PyRole::from_str(" label").is_err());
        assert!(PyRole::from_str(" asdf").is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_py() -> Result<(), ContextError> {
        assert_eq!(PyRole::from_str("attribute")?, PyRole::Attribute);
        assert_eq!(PyRole::from_str("data")?, PyRole::Data);
        assert_eq!(PyRole::from_str("exception")?, PyRole::Exception);
        assert_eq!(PyRole::from_str("function")?, PyRole::Function);
        assert_eq!(PyRole::from_str("method")?, PyRole::Method);
        assert_eq!(PyRole::from_str("module")?, PyRole::Module);
        assert_eq!(PyRole::from_str("property")?, PyRole::Property);
        assert_eq!(PyRole::from_str("class")?, PyRole::Class);
        Ok(())
    }
}
