use std::str::FromStr;

use winnow::{ModalResult, Parser, error::StrContext, stream::AsChar, token::take_till};

use crate::{error::MalformedReference, roles::SphinxType};

/// Describes a JavaScript role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `js:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq)]
pub enum JsRole {
    /// This directive sets the module name for object declarations that follow after
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/javascript.html#directive-js-module)
    Module,

    /// Describes a JavaScript Function or method
    /// see also [the shinx docs](https://www.sphinx-doc.org/en/master/usage/domains/javascript.html#directive-js-function)
    Function,

    /// An alias for [`JsRole::Function`]
    Method,

    /// Describes a javaScript constructor that creates an object
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/javascript.html#directive-js-class)
    Class,

    /// Describes a global variable or constant.
    /// see also [the sphinx docs](https://www.sphinx-doc.org/en/master/usage/domains/javascript.html#directive-js-data)
    Data,
}
impl FromStr for JsRole {
    type Err = MalformedReference;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "module" => Ok(JsRole::Module),
            "data" => Ok(JsRole::Data),
            "function" => Ok(JsRole::Function),
            "method" => Ok(JsRole::Method),
            "class" => Ok(JsRole::Class),

            _ => Err(MalformedReference::InvalidRole(s.to_string())),
        }
    }
}

impl TryFrom<&str> for JsRole {
    type Error = MalformedReference;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

/// Parses a cpp role as defined in [`JsRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn js_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., |c| AsChar::is_space(c) || AsChar::is_newline(c))
        .context(StrContext::Label("Js Role"))
        .parse_to()
        .parse_next(input)?;
    Ok(SphinxType::JavaScript(role))
}

#[cfg(test)]
mod test {

    use crate::error::MalformedReference;

    use super::*;
    #[test]
    fn test_sphinx_role_parsing_std_err() {
        assert!(JsRole::try_from("asdf").is_err());
        assert!(JsRole::try_from("doc").is_err());
        assert!(JsRole::try_from("").is_err());
        assert!(JsRole::try_from("::::").is_err());
        assert!(JsRole::try_from(" label").is_err());
        assert!(JsRole::try_from(" asdf").is_err());
    }
    #[test]
    fn test_sphinx_type_parsing_js() -> Result<(), MalformedReference> {
        assert_eq!(JsRole::try_from("module")?, JsRole::Module);
        assert_eq!(JsRole::try_from("function")?, JsRole::Function);
        assert_eq!(JsRole::try_from("method")?, JsRole::Method);
        assert_eq!(JsRole::try_from("class")?, JsRole::Class);
        assert_eq!(JsRole::try_from("data")?, JsRole::Data);
        Ok(())
    }
}
