use std::{fmt::Display, str::FromStr};

use winnow::{
    ModalResult, Parser,
    error::{ContextError, StrContext},
    stream::AsChar,
    token::take_till,
};

use crate::SphinxType;

#[derive(Debug, PartialEq)]
pub enum CmakeRole {
    /// A cmake command in a cmake file like `set()`
    /// or `add_executable()` not to be confused with a command line
    /// command like `cmake build` see [the cmake docs](https://cmake.org/cmake/help/latest/manual/cmake-commands.7.html#cmake-commands-7) for more info
    Command,

    /// Cpack generators create packages out of artifacts such as installers
    /// [see more](https://cmake.org/cmake/help/latest/module/CPack.html#module:CPack)
    CpackGen,

    /// A environment variable that cmake can use, e.g. `CMAKE_CONFIG_DIR` [ see more]( https://cmake.org/cmake/help/latest/manual/cmake-env-variables.7.html ) for more
    Envvar,

    /// A cmake Generator is responsible for writing the input files for a native build system.
    /// [see more](https://cmake.org/cmake/help/latest/manual/cmake-generators.7.html)
    Generator,

    /// A cmake generator expression [see more](https://cmake.org/cmake/help/latest/guide/tutorial/Miscellaneous%20Features.html#exercise-2-generator-expressions)
    Genex,

    /// refers to a gide page such as the tutorial in the documentation
    Guide,

    /// Refers to a man page such as `cmake(1)`
    Manual,

    /// A cmake module designed for reuse. [see more](https://cmake.org/cmake/help/latest/manual/cmake-modules.7.html#manual:cmake-modules(7))
    Module,

    /// policies introduce behavior changes while preserving compatibility for existing
    /// project releases. [see more](https://cmake.org/cmake/help/latest/manual/cmake-policies.7.html)
    Policy,

    /// A cache property [see more](https://cmake.org/cmake/help/latest/manual/cmake-properties.7.html#properties-on-cache-entries)
    PropCache,

    /// A directory property [see more](https://cmake.org/cmake/help/latest/manual/cmake-properties.7.html#properties-on-directories)
    PropDir,

    /// A global cmake property [see more](https://cmake.org/cmake/help/latest/manual/cmake-properties.7.html#properties-of-global-scope)
    PropGbl,

    /// Properties on installed files [see more](https://cmake.org/cmake/help/latest/manual/cmake-properties.7.html#properties-on-installed-files)
    PropInst,

    /// Properties on source files [see more](https://cmake.org/cmake/help/latest/manual/cmake-properties.7.html#properties-on-source-files)
    PropSf,

    /// Properties on tests [see more](https://cmake.org/cmake/help/latest/manual/cmake-properties.7.html#properties-on-tests)
    PropTest,

    /// A target property [see more](https://cmake.org/cmake/help/latest/manual/cmake-properties.7.html#properties-on-targets)
    PropTgt,

    /// A cmake variable. can be set by cmake itself or by project code
    /// [see more](https://cmake.org/cmake/help/latest/manual/cmake-variables.7.html)
    Variable,
}

impl Display for CmakeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CmakeRole::Command => "command",
            CmakeRole::CpackGen => "cpack_gen",
            CmakeRole::Envvar => "envvar",
            CmakeRole::Generator => "generator",
            CmakeRole::Genex => "genex",
            CmakeRole::Guide => "guide",
            CmakeRole::Manual => "manual",
            CmakeRole::Module => "module",
            CmakeRole::Policy => "policy",
            CmakeRole::PropCache => "prop_cache",
            CmakeRole::PropDir => "prop_dir",
            CmakeRole::PropGbl => "prop_gbl",
            CmakeRole::PropInst => "prop_inst",
            CmakeRole::PropSf => "prop_sf",
            CmakeRole::PropTest => "prop_test",
            CmakeRole::PropTgt => "prop_tgt",
            CmakeRole::Variable => "variable",
        })
    }
}

/// Parses a cpp role as defined in [`HttpRole`]
/// may not contain whitespace but may contain other colons
pub(crate) fn cmake_role(input: &mut &str) -> ModalResult<SphinxType> {
    let role = take_till(1.., AsChar::is_space)
        .context(StrContext::Label("cmake role"))
        .parse_to::<CmakeRole>()
        .parse_next(input)?;
    Ok(SphinxType::Cmake(role))
}
impl FromStr for CmakeRole {
    type Err = ContextError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "command" => Ok(CmakeRole::Command),
            "cpack_gen" => Ok(CmakeRole::CpackGen),
            "envvar" => Ok(CmakeRole::Envvar),
            "generator" => Ok(CmakeRole::Generator),
            "genex" => Ok(CmakeRole::Genex),
            "guide" => Ok(CmakeRole::Guide),
            "manual" => Ok(CmakeRole::Manual),
            "module" => Ok(CmakeRole::Module),
            "policy" => Ok(CmakeRole::Policy),
            "prop_cache" => Ok(CmakeRole::PropCache),
            "prop_dir" => Ok(CmakeRole::PropDir),
            "prop_gbl" => Ok(CmakeRole::PropGbl),
            "prop_inst" => Ok(CmakeRole::PropInst),
            "prop_sf" => Ok(CmakeRole::PropSf),
            "prop_test" => Ok(CmakeRole::PropTest),
            "prop_tgt" => Ok(CmakeRole::PropTgt),
            "variable" => Ok(CmakeRole::Variable),

            _ => Err(ContextError::new()),
        }
    }
}
