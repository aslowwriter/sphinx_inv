use std::str::FromStr;

use winnow::{ModalResult, Parser, error::StrContext, stream::AsChar, token::take_till};

use crate::{error::MalformedReference, roles::SphinxType};

/// Describes a C role that has been observed in the wild, i.e. one of the known
/// inventory file declared at least one line with the type `c:{role}`
/// if you would like one added please open a feature request
#[derive(Debug, PartialEq)]
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

impl FromStr for CRole {
    type Err = MalformedReference;

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

            _ => Err(MalformedReference::InvalidRole(s.to_string())),
        }
    }
}

/// Parses a c role as defined in [`CRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn c_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., |c| AsChar::is_space(c) || AsChar::is_newline(c))
        .context(StrContext::Label("c role"))
        .parse_to()
        .parse_next(input)?;
    Ok(SphinxType::C(role))
}

#[cfg(test)]
mod test {

    use winnow::ModalResult;

    use crate::{
        roles::SphinxType,
        roles::{CRole, c_role::c_role},
    };

    #[test]
    fn invalid_c_role() {
        let mut input = "asdfasdf";

        let result = c_role(&mut input);
        assert!(result.is_err());
    }
    #[test]
    fn c_role_enum() -> ModalResult<()> {
        let mut input = "enum";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Enum));
        Ok(())
    }
    #[test]
    fn c_role_enumerator() -> ModalResult<()> {
        let mut input = "enumerator";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Enumerator));
        Ok(())
    }
    #[test]
    fn c_role_function() -> ModalResult<()> {
        let mut input = "function";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Function));
        Ok(())
    }
    #[test]
    fn c_role_function_param() -> ModalResult<()> {
        let mut input = "functionParam";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::FunctionParam));
        Ok(())
    }
    #[test]
    fn c_role_member() -> ModalResult<()> {
        let mut input = "member";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Member));
        Ok(())
    }
    #[test]
    fn c_role_macro() -> ModalResult<()> {
        let mut input = "macro";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Macro));
        Ok(())
    }
    #[test]
    fn c_role_var() -> ModalResult<()> {
        let mut input = "var";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Var));
        Ok(())
    }
    #[test]
    fn c_role_type() -> ModalResult<()> {
        let mut input = "type";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Type));
        Ok(())
    }
    #[test]
    fn c_role_struct() -> ModalResult<()> {
        let mut input = "struct";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Struct));
        Ok(())
    }
    #[test]
    fn c_role_union() -> ModalResult<()> {
        let mut input = "union";

        let role = c_role(&mut input)?;

        assert_eq!(role, SphinxType::C(CRole::Union));
        Ok(())
    }
}
