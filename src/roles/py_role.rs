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
#[derive(Debug, PartialEq, Clone)]
pub enum PyRole {
    /// documents a parameter to a function or method.
    /// [see more](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-function)
    Parameter,

    /// Describes an object data attribute
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-attribute)
    Attribute,

    /// An alias for [`PyRole::Attribute`]
    Attr,

    /// References a module-level python variable.
    /// `Type` should be used for type aliases and `Attribute` for class variables
    /// and instance attributes
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-data)
    Data,

    /// A python class describing an exception that can be thrown
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/python.html#directive-py-exception)
    Exception,

    /// A static method, aka one that does not take `self` as it's first argument
    /// see also [the python docs](https://docs.python.org/3/library/functions.html#staticmethod)
    Staticmethod,

    /// A python enum, aka a fixed collection of possible values.
    /// see also [the python docs](https://docs.python.org/3/library/enum.html#enum.Enum)
    Enum,

    /// An attribute of a pydantic model and it's associated validators
    /// see also [the pydantic docs](https://pydantic.dev/docs/validation/latest/concepts/fields/)
    PydanticField,

    /// A validator for a pydantic model field to validate its data
    /// see also [the pydantic docs](https://pydantic.dev/docs/validation/latest/concepts/validators/)
    PydanticValidator,

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

    /// A class method, aka a function on a class object rather than any particular instance of that
    /// class
    /// see also [the python docs](https://docs.python.org/3/library/functions.html#classmethod)
    Classmethod,

    /// A pydantic model, a data class designed to hold validated data
    /// see also [the pydantic docs](https://pydantic.dev/docs/validation/latest/concepts/models/)
    PydanticModel,

    /// python doesn't actually have interfaces, so I'm not sure what this means...
    /// I did find it in the wild though so I added it here.
    Interface,
}

impl Display for PyRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PyRole::Attribute => "attribute",
            PyRole::Attr => "attr",
            PyRole::Staticmethod => "staticmethod",
            PyRole::Parameter => "parameter",
            PyRole::Data => "data",
            PyRole::Exception => "exception",
            PyRole::Type => "type",
            PyRole::Function => "function",
            PyRole::Method => "method",
            PyRole::Module => "module",
            PyRole::Classmethod => "classmethod",
            PyRole::Property => "property",
            PyRole::Class => "class",
            PyRole::PydanticField => "pydantic_field",
            PyRole::PydanticValidator => "pydantic_validator",
            PyRole::PydanticModel => "pydantic_model",
            PyRole::Enum => "enum",
            PyRole::Interface => "interface",
        })
    }
}
impl FromStr for PyRole {
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "attr" => Ok(PyRole::Attr),
            "attribute" => Ok(PyRole::Attribute),
            "class" => Ok(PyRole::Class),
            "classmethod" => Ok(PyRole::Classmethod),
            "data" => Ok(PyRole::Data),
            "enum" => Ok(PyRole::Enum),
            "exception" => Ok(PyRole::Exception),
            "function" => Ok(PyRole::Function),
            "interface" => Ok(PyRole::Interface),
            "method" => Ok(PyRole::Method),
            "module" => Ok(PyRole::Module),
            "parameter" => Ok(PyRole::Parameter),
            "property" => Ok(PyRole::Property),
            "pydantic_field" => Ok(PyRole::PydanticField),
            "pydantic_model" => Ok(PyRole::PydanticModel),
            "pydantic_validator" => Ok(PyRole::PydanticValidator),
            "staticmethod" => Ok(PyRole::Staticmethod),
            "type" => Ok(PyRole::Type),

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
