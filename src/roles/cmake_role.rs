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
    Command,
    CpackGen,
    Envvar,
    Generator,
    Genex,
    Guide,
    Manual,
    Module,
    Policy,
    PropCache,
    PropDir,
    PropGbl,
    PropInst,
    PropSf,
    PropTest,
    PropTgt,
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
