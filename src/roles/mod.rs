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
use std::str::FromStr;

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
