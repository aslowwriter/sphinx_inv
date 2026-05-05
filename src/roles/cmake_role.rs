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
    Variable,
    PropTgt,
    Policy,
    Module,
    Command,
    PropGbl,
    PropDir,
    PropTest,
    Genex,
    Generator,
    Envvar,
    CpackGen,
    Manual,
    PropSf,
    PropCache,
    Guide,
    PropInst,
}

impl Display for CmakeRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            CmakeRole::Variable => "variable",
            CmakeRole::Guide => "guide",
            CmakeRole::PropTgt => "prop_tgt",
            CmakeRole::PropInst => "prop_inst",
            CmakeRole::Module => "module",
            CmakeRole::Policy => "policy",
            CmakeRole::Command => "command",
            CmakeRole::PropGbl => "prop_gbl",
            CmakeRole::PropTest => "prop_test",
            CmakeRole::Genex => "genex",
            CmakeRole::Generator => "generator",
            CmakeRole::Envvar => "envvar",
            CmakeRole::CpackGen => "cpack_gen",
            CmakeRole::PropDir => "prop_dir",
            CmakeRole::Manual => "manual",
            CmakeRole::PropSf => "prop_sf",
            CmakeRole::PropCache => "prop_cache",
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
            "variable" => Ok(CmakeRole::Variable),
            "prop_tgt" => Ok(CmakeRole::PropTgt),
            "module" => Ok(CmakeRole::Module),
            "policy" => Ok(CmakeRole::Policy),
            "command" => Ok(CmakeRole::Command),
            "prop_gbl" => Ok(CmakeRole::PropGbl),
            "prop_dir" => Ok(CmakeRole::PropDir),
            "manual" => Ok(CmakeRole::Manual),
            "prop_test" => Ok(CmakeRole::PropTest),
            "genex" => Ok(CmakeRole::Genex),
            "prop_sf" => Ok(CmakeRole::PropSf),
            "generator" => Ok(CmakeRole::Generator),
            "envvar" => Ok(CmakeRole::Envvar),
            "cpack_gen" => Ok(CmakeRole::CpackGen),
            "prop_cache" => Ok(CmakeRole::PropCache),
            "prop_inst" => Ok(CmakeRole::PropInst),
            "guide" => Ok(CmakeRole::Guide),

            _ => Err(ContextError::new()),
        }
    }
}
