use std::str::FromStr;

use winnow::{ModalResult, Parser, Result, error::StrContext, stream::AsChar, token::take_till};

use crate::{error::MalformedReference, roles::SphinxType};

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
impl FromStr for PyRole {
    type Err = MalformedReference;

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

            _ => Err(MalformedReference::InvalidRole(s.to_string())),
        }
    }
}

impl TryFrom<&str> for PyRole {
    type Error = MalformedReference;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

/// Parses a py role as defined in [`PyRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn py_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(0.., |c| AsChar::is_space(c) || AsChar::is_newline(c))
        .context(StrContext::Label("Py Role"))
        .parse_to()
        .parse_next(input)?;
    Ok(SphinxType::Python(role))
}

#[cfg(test)]
mod test {

    use winnow::ModalResult;

    use crate::error::MalformedReference;
    use crate::roles::PyRole;
    use crate::roles::SphinxType;
    use crate::roles::py_role::Result;
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
        assert!(PyRole::try_from("asdf").is_err());
        assert!(PyRole::try_from("doc").is_err());
        assert!(PyRole::try_from("").is_err());
        assert!(PyRole::try_from("::::").is_err());
        assert!(PyRole::try_from(" label").is_err());
        assert!(PyRole::try_from(" asdf").is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_py() -> Result<(), MalformedReference> {
        assert_eq!(PyRole::try_from("attribute")?, PyRole::Attribute);
        assert_eq!(PyRole::try_from("data")?, PyRole::Data);
        assert_eq!(PyRole::try_from("exception")?, PyRole::Exception);
        assert_eq!(PyRole::try_from("function")?, PyRole::Function);
        assert_eq!(PyRole::try_from("method")?, PyRole::Method);
        assert_eq!(PyRole::try_from("module")?, PyRole::Module);
        assert_eq!(PyRole::try_from("property")?, PyRole::Property);
        assert_eq!(PyRole::try_from("class")?, PyRole::Class);
        Ok(())
    }
}
