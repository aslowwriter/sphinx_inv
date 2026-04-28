mod c_role;
mod cpp_role;
mod js_role;
mod math_role;
mod py_role;
mod rst_role;
mod std_role;

pub(crate) use c_role::c_role;
pub(crate) use cpp_role::cpp_role;
pub(crate) use js_role::js_role;
pub(crate) use math_role::math_role;
pub(crate) use py_role::py_role;
pub(crate) use rst_role::rst_role;
pub(crate) use std_role::std_role;

pub use c_role::CRole;
pub use cpp_role::CppRole;
pub use js_role::JsRole;
pub use math_role::MathRole;
pub use py_role::PyRole;
pub use rst_role::RstRole;
pub use std_role::StdRole;

use crate::error::MalformedReference;
use std::{fmt::Display, str::FromStr};

#[derive(Debug, PartialEq)]
pub enum SphinxType {
    Std(StdRole),
    C(CRole),
    Python(PyRole),
    Cpp(CppRole),
    JavaScript(JsRole),
    Mathematics(MathRole),
    ReStructuredText(RstRole),
}

impl Display for SphinxType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&match self {
            SphinxType::Std(std_role) => format!("std:{std_role}"),
            SphinxType::C(crole) => format!("c:{crole}"),
            SphinxType::Python(py_role) => format!("py:{py_role}"),
            SphinxType::Cpp(cpp_role) => format!("cpp:{cpp_role}"),
            SphinxType::JavaScript(js_role) => format!("js:{js_role}"),
            SphinxType::Mathematics(math_role) => format!("math:{math_role}"),
            SphinxType::ReStructuredText(rst_role) => format!("rst:{rst_role}"),
        })
    }
}

impl TryFrom<&str> for SphinxType {
    type Error = MalformedReference;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.split_once(':') {
            Some((domain, role)) => match domain {
                "std" => Ok(SphinxType::Std(StdRole::from_str(role)?)),
                "c" => Ok(SphinxType::C(CRole::from_str(role)?)),
                "cpp" => Ok(SphinxType::Cpp(CppRole::from_str(role)?)),
                "py" => Ok(SphinxType::Python(PyRole::from_str(role)?)),
                "js" => Ok(SphinxType::JavaScript(JsRole::from_str(role)?)),
                "math" => Ok(SphinxType::Mathematics(MathRole::from_str(role)?)),
                "rst" => Ok(SphinxType::ReStructuredText(RstRole::from_str(role)?)),
                _ => Err(MalformedReference::InvalidDomain(domain.to_string())),
            },
            None => Err(MalformedReference::MalformedDomainField(value.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        CRole, CppRole, JsRole, MathRole, PyRole, RstRole,
        roles::{SphinxType, StdRole},
    };

    #[test]
    fn display_js_role() {
        assert_eq!(
            format!("{}", SphinxType::JavaScript(JsRole::Module)),
            "js:module"
        );
        assert_eq!(
            format!("{}", SphinxType::JavaScript(JsRole::Function)),
            "js:function"
        );
        assert_eq!(
            format!("{}", SphinxType::JavaScript(JsRole::Method)),
            "js:method"
        );
        assert_eq!(
            format!("{}", SphinxType::JavaScript(JsRole::Class)),
            "js:class"
        );
        assert_eq!(
            format!("{}", SphinxType::JavaScript(JsRole::Data)),
            "js:data"
        );
    }
    #[test]
    fn display_py_role() {
        assert_eq!(format!("{}", SphinxType::Python(PyRole::Class)), "py:class");
        assert_eq!(
            format!("{}", SphinxType::Python(PyRole::Property)),
            "py:property"
        );
        assert_eq!(
            format!("{}", SphinxType::Python(PyRole::Module)),
            "py:module"
        );
        assert_eq!(
            format!("{}", SphinxType::Python(PyRole::Method)),
            "py:method"
        );
        assert_eq!(
            format!("{}", SphinxType::Python(PyRole::Function)),
            "py:function"
        );
        assert_eq!(format!("{}", SphinxType::Python(PyRole::Type)), "py:type");
        assert_eq!(
            format!("{}", SphinxType::Python(PyRole::Exception)),
            "py:exception"
        );
        assert_eq!(format!("{}", SphinxType::Python(PyRole::Data)), "py:data");
        assert_eq!(
            format!("{}", SphinxType::Python(PyRole::Attribute)),
            "py:attribute"
        );
    }
    #[test]
    fn display_cpp_role() {
        assert_eq!(format!("{}", SphinxType::Cpp(CppRole::Type)), "cpp:type");
        assert_eq!(
            format!("{}", SphinxType::Cpp(CppRole::Function)),
            "cpp:function"
        );
        assert_eq!(format!("{}", SphinxType::Cpp(CppRole::Class)), "cpp:class");
        assert_eq!(
            format!("{}", SphinxType::Cpp(CppRole::FunctionParam)),
            "cpp:functionParam"
        );
        assert_eq!(
            format!("{}", SphinxType::Cpp(CppRole::Member)),
            "cpp:member"
        );
        assert_eq!(
            format!("{}", SphinxType::Cpp(CppRole::TemplateParam)),
            "cpp:templateParam"
        );
    }
    #[test]
    fn display_c_role() {
        assert_eq!(
            format!("{}", SphinxType::C(CRole::Enumerator)),
            "c:enumerator"
        );
        assert_eq!(format!("{}", SphinxType::C(CRole::Enum)), "c:enum");
        assert_eq!(format!("{}", SphinxType::C(CRole::Function)), "c:function");
        assert_eq!(
            format!("{}", SphinxType::C(CRole::FunctionParam)),
            "c:functionParam"
        );
        assert_eq!(format!("{}", SphinxType::C(CRole::Member)), "c:member");
        assert_eq!(format!("{}", SphinxType::C(CRole::Macro)), "c:macro");
        assert_eq!(format!("{}", SphinxType::C(CRole::Var)), "c:var");
        assert_eq!(format!("{}", SphinxType::C(CRole::Type)), "c:type");
        assert_eq!(format!("{}", SphinxType::C(CRole::Struct)), "c:struct");
        assert_eq!(format!("{}", SphinxType::C(CRole::Union)), "c:union");
    }
    #[test]
    fn display_rst_role() {
        assert_eq!(
            format!("{}", SphinxType::ReStructuredText(RstRole::Directive)),
            "rst:directive"
        );
        assert_eq!(
            format!("{}", SphinxType::ReStructuredText(RstRole::Option)),
            "rst:directive:option"
        );
    }
    #[test]
    fn display_math_role() {
        assert_eq!(
            format!("{}", SphinxType::Mathematics(MathRole::Numref)),
            "math:numref"
        );
    }
    #[test]
    fn display_std_type() {
        assert_eq!(format!("{}", SphinxType::Std(StdRole::Doc)), "std:doc");
        assert_eq!(format!("{}", SphinxType::Std(StdRole::Term)), "std:term");
        assert_eq!(format!("{}", SphinxType::Std(StdRole::Label)), "std:label");
        assert_eq!(
            format!("{}", SphinxType::Std(StdRole::Cmdoption)),
            "std:cmdoption"
        );
        assert_eq!(
            format!("{}", SphinxType::Std(StdRole::Pdbcommand)),
            "std:pdbcommand"
        );
        assert_eq!(format!("{}", SphinxType::Std(StdRole::Token)), "std:token");
        assert_eq!(
            format!("{}", SphinxType::Std(StdRole::Opcode)),
            "std:opcode"
        );
        assert_eq!(
            format!("{}", SphinxType::Std(StdRole::MonitoringEvent)),
            "std:monitoring-event"
        );
        assert_eq!(
            format!("{}", SphinxType::Std(StdRole::Envvar)),
            "std:envvar"
        );
    }
}
