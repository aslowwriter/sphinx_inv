use std::fmt::Display;

use crate::{
    error::SphinxParseError,
    priority::SphinxPriority,
    roles::{
        SphinxType, c_role, cmake_role, cpp_role, http_role, js_role, math_role, py_role, rst_role,
        sip_role, std_role,
    },
};
use winnow::{
    ModalResult, Parser,
    ascii::{space1, till_line_ending},
    combinator::{alt, cut_err, dispatch, fail, preceded, repeat_till, terminated, trace},
    error::{StrContext, StrContextValue},
    stream::AsChar,
    token::take_while,
};

#[derive(Debug, PartialEq)]
pub struct SphinxReference {
    pub name: String,
    // type is a reserved keyword
    pub sphinx_type: SphinxType,
    pub priority: SphinxPriority,
    pub location: String,
    pub display_name: String,
}

impl SphinxReference {
    pub fn new(
        name: String,
        sphinx_type: SphinxType,
        priority: Option<SphinxPriority>,
        location: String,
        display_name: Option<String>,
    ) -> Self {
        Self {
            name,
            sphinx_type,
            priority: priority.unwrap_or(SphinxPriority::Standard),
            location,
            display_name: display_name.unwrap_or("-".to_string()),
        }
    }
}

impl Display for SphinxReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} {} {} {} {}",
            self.name, self.sphinx_type, self.priority, self.location, self.display_name
        ))
    }
}

fn word<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    take_while(1.., |c| {
        (AsChar::is_alphanum(c) || c == '_') && !AsChar::is_newline(c)
    })
    .parse_next(input)
}

fn non_space<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    take_while(1.., |c| !AsChar::is_space(c) && !AsChar::is_newline(c)).parse_next(input)
}

fn non_word<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    take_while(1.., |c| {
        !(AsChar::is_alphanum(c) || c == '_' || AsChar::is_newline(c))
    })
    .parse_next(input)
}

fn domain<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    trace("domain", word).parse_next(input)
}

fn role_domain(input: &mut &str) -> ModalResult<SphinxType> {
    dispatch! {terminated(domain,':');
        "std" => cut_err(std_role),
        "py" => cut_err(py_role),
        "c" => cut_err(c_role),
        "rst" => cut_err(rst_role),
        "cpp" => cut_err(cpp_role),
        "js" => cut_err(js_role),
        "math" => cut_err(math_role),
        "sip" => cut_err(sip_role),
        "http" => cut_err(http_role),
        "cmake" => cut_err(cmake_role),
        _ => fail.context(StrContext::Label("unknown domain"))
            .context(StrContext::Expected(StrContextValue::StringLiteral("std")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("py")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("c")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("rst")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("cpp")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("js")))
            .context(StrContext::Expected(StrContextValue::StringLiteral("math")))
    }
    .parse_next(input)
}

fn priority(input: &mut &str) -> ModalResult<SphinxPriority> {
    preceded(space1, alt(("-1", "1", "0", "2")))
        .parse_to()
        .parse_next(input)
}

fn uri<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    trace("uri", preceded(space1, non_space)).parse_next(input)
}

fn display_name<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    trace("display_name", preceded(space1, till_line_ending)).parse_next(input)
}

fn name_domain_role(input: &mut &str) -> ModalResult<(String, SphinxType)> {
    // this is a bit nasty, but it's necessary to make sure we parse at least one word
    // the first word is not allowed to be the role and there are some cases where this one
    // contains a : which trips up the parser, so we take the first word a bit more liberally
    let (first_word, (mut prefix_vec, role)): (&str, (String, SphinxType)) = trace(
        "name_domain_role",
        (
            non_space,
            repeat_till(0.., alt((word, non_word)), role_domain),
        ),
    )
    .parse_next(input)?;
    // the last space was separating the title and the domain, so we pop that off
    let _ = prefix_vec.pop();
    Ok((format!("{first_word}{prefix_vec}"), role))
}

pub fn parse_reference(line: &str, line_num: usize) -> Result<SphinxReference, SphinxParseError> {
    let ((name, sphinx_type), prio, loc, dispname) = reference
        .parse(line)
        .map_err(|error| SphinxParseError::from_str_parse(&error, line_num))?;

    // let display_name = if dispname == "-" {
    //     name.clone()
    // } else {
    //     dispname.to_string()
    // };
    // let location = loc.replace('$', &name);

    Ok(SphinxReference {
        name,
        sphinx_type,
        priority: prio,
        location: loc.to_string(),
        display_name: dispname.to_string(),
    })
}

fn reference<'a>(
    input: &mut &'a str,
) -> ModalResult<((String, SphinxType), SphinxPriority, &'a str, &'a str)> {
    (name_domain_role, priority, uri, display_name).parse_next(input)
}

#[cfg(test)]
mod test {

    use crate::{
        CRole,
        error::SphinxParseError,
        roles::{PyRole, RstRole, StdRole},
    };

    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_hard_dummy_record() -> Result<(), SphinxParseError> {
        // TODO:
        // for the error reporting I had to disallow strings that conform to `(\w+):` but I'm
        // undecided on whether I want to keep this behaviour. Revisit this once I'm done adding
        // domains. it might also be useful to see some other nasty stuff from cmake or whatever
        let input = "asdfasdf :foo std ::endl :: _bar_baz : something- : hello std:label 1 library/stdtypes.html asdf";

        let sphinx_ref = parse_reference(input, 0)?;

        assert_eq!(
            sphinx_ref.name,
            "asdfasdf :foo std ::endl :: _bar_baz : something- : hello".to_string()
        );
        assert_eq!(sphinx_ref.sphinx_type, SphinxType::Std(StdRole::Label));
        assert_eq!(sphinx_ref.priority, SphinxPriority::Standard);
        assert_eq!(sphinx_ref.location, "library/stdtypes.html");
        assert_eq!(sphinx_ref.display_name, "asdf");

        Ok(())
    }
    #[test]
    fn test_parse_example_record_with_rst_directive() -> Result<(), SphinxParseError> {
        let input = "str.join rst:directive:option 1 library/stdtypes.html#$ -".to_string();

        let sphinx_ref = parse_reference(&input, 0)?;
        assert_eq!(sphinx_ref.name, "str.join".to_string());
        assert_eq!(
            sphinx_ref.sphinx_type,
            SphinxType::ReStructuredText(RstRole::Option)
        );
        assert_eq!(sphinx_ref.priority, SphinxPriority::Standard);
        assert_eq!(sphinx_ref.location, "library/stdtypes.html#$");
        assert_eq!(sphinx_ref.display_name, "-");

        Ok(())
    }

    #[test]
    fn type_parse_unknown_domain_err() {
        let header = "str.join asdf:method 1 library/stdtypes.html#$ -".to_string();
        let result = parse_reference(&header, 0);
        assert_eq!(
            result,
            Err(SphinxParseError::from_str(
                "str.join asdf:method 1 library/stdtypes.html#$ -",
                // "invalid unknown domain\nexpected `std`, `py`, `c`, `rst`, `cpp`, `js`, `math`",
                "",
                48,
                0
            ))
        );
    }
    #[test]

    fn type_parse_py_role_err() {
        let header = "str.join py:asdf 1 library/stdtypes.html#$ -".to_string();
        let result = parse_reference(&header, 0);
        assert_eq!(
            result,
            Err(SphinxParseError::from_str(
                "str.join py:asdf 1 library/stdtypes.html#$ -",
                "invalid python role\nexpected `attribute`, `data`, `exception`, `function`, `method`, `module`, `property`, `class`",
                12,
                0
            ))
        );
    }

    #[test]
    fn test_parse_example_record_with_newline() {
        let mut input = "str.join\n py:method 1 library/stdtypes.html#$ -";

        let result = reference(&mut input);
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_example_record() -> Result<(), SphinxParseError> {
        let input = "str.join py:method 1 library/stdtypes.html#$ -".to_string();

        let sphinx_ref = parse_reference(&input, 0)?;
        assert_eq!(sphinx_ref.name, "str.join".to_string());
        assert_eq!(sphinx_ref.sphinx_type, SphinxType::Python(PyRole::Method));
        assert_eq!(sphinx_ref.priority, SphinxPriority::Standard);
        assert_eq!(sphinx_ref.location, "library/stdtypes.html#$");
        assert_eq!(sphinx_ref.display_name, "-");

        Ok(())
    }

    #[test]
    fn test_lkd_hard_line_with_rst_directive() -> Result<(), SphinxParseError> {
        let input = "accel/qaic/aic080:qualcomm cloud ai 80 (aic080) rst:directive:option -1 accel/qaic/aic080.html#qualcomm-cloud-ai-80-aic080 Qualcomm Cloud AI 80 (AIC080)".to_string();

        let sphinx_ref = parse_reference(&input, 0)?;
        assert_eq!(
            sphinx_ref.sphinx_type,
            SphinxType::ReStructuredText(RstRole::Option)
        );
        assert_eq!(sphinx_ref.priority, SphinxPriority::Omit);
        assert_eq!(
            sphinx_ref.location,
            "accel/qaic/aic080.html#qualcomm-cloud-ai-80-aic080"
        );
        assert_eq!(sphinx_ref.display_name, "Qualcomm Cloud AI 80 (AIC080)");

        Ok(())
    }

    #[test]
    fn test_lkd_hard_line() -> Result<(), SphinxParseError> {
        let input = "accel/qaic/aic080:qualcomm cloud ai 80 (aic080) std:label -1 accel/qaic/aic080.html#qualcomm-cloud-ai-80-aic080 Qualcomm Cloud AI 80 (AIC080)".to_string();

        let sphinx_ref = parse_reference(&input, 0)?;
        assert_eq!(
            sphinx_ref.name,
            "accel/qaic/aic080:qualcomm cloud ai 80 (aic080)".to_string()
        );
        assert_eq!(sphinx_ref.sphinx_type, SphinxType::Std(StdRole::Label));
        assert_eq!(sphinx_ref.priority, SphinxPriority::Omit);
        assert_eq!(
            sphinx_ref.location,
            "accel/qaic/aic080.html#qualcomm-cloud-ai-80-aic080"
        );
        assert_eq!(sphinx_ref.display_name, "Qualcomm Cloud AI 80 (AIC080)");

        Ok(())
    }

    #[test]
    fn new_reference() {
        assert_eq!(
            SphinxReference {
                name: "foo".to_string(),
                sphinx_type: SphinxType::C(CRole::Macro),
                priority: SphinxPriority::Standard,
                location: "foo/bar".to_string(),
                display_name: "-".to_string()
            },
            SphinxReference::new(
                "foo".to_string(),
                SphinxType::C(CRole::Macro),
                None,
                "foo/bar".to_string(),
                None
            )
        );
    }

    // These are tests that we took from inventory files in the wild that we put here so we don't
    // havd to keep entire inventory files around for testing
    // as such we just want them to parse, and don't actually check the output
    #[test]
    fn parsing_external_py() -> Result<(), SphinxParseError> {
        let examples = vec![
            "or_ar.vecm.select_order.data py:parameter 2 generated/statsmodels.tsa.vector_ar.vecm.select_order.html#$ -",
            "right_x cpp:enumerator 1 cpp/reference/panda3d.core.InputDevice#_CPPv4N4Axis7right_xE -",
            "TextureStage::Mode cpp:enum 1 cpp/reference/panda3d.core.TextureStage#_CPPv4N12TextureStage4ModeE -",
            "typo3-documentation std:title -1 Home/WikiLanding.html#wikilanding TYPO3 Documentation",
            "pySPACE.resources.dataset_defs.stream.StreamDataset.project2d py:staticmethod 1 api/generated/pySPACE.resources.dataset_defs.stream.html#$ -",
            "PyQt5.QtXmlPatterns.QXmlNodeModelIndex.NodeKind.Attribute sip:member 0 api/qtxmlpatterns/qxmlnodemodelindex.html##NodeKind-Attribute Attribute",
            "PyQt5.QtXmlPatterns.QXmlResultItems sip:class 0 api/qtxmlpatterns/qxmlresultitems.html QXmlResultItems",
            "PyQt5.QtXml sip:module 0 api/qtxml/qtxml-module.html QtXml",
            "PyQt5.QtWinExtras.QtWin.WindowFlip3DPolicy sip:enum 0 api/qtwinextras/qtwin.html##WindowFlip3DPolicy WindowFlip3DPolicy",
            "PyQt5.QtWidgets.QTextEdit.undoAvailable sip:signal 0 api/qtwidgets/qtextedit.html##undoAvailable undoAvailable",
            "PyQt5.QtWidgets.QStyleOptionTabWidgetFrame.tabBarRect sip:attribute 0 api/qtwidgets/qstyleoptiontabwidgetframe.html##tabBarRect tabBarRect",
            "pandas.api.typing.aliases.JoinHow py:type 1 reference/aliases.html#$ -",
            "SAPT2+3(CCD)DMP2 TOTAL ENERGY std:psivar 1 glossary_psivariables.html#psivar-20 -",
            "optking.v1.optparams.OptParams.fix_val_near_pi py:pydantic_field 1 optking.html#$ -",
            "coverage_ignore_functions std:confval 1 usage/extensions/coverage.html#confval-$ -",
            "/d2l/api/lp/(version)/orgstructure/recyclebin/ http:get 1 res/orgunit.html#get--d2l-api-lp-(version)-orgstructure-recyclebin- -",
            "/d2l/api/le/(version)/lti/link/(orgUnitId) http:post 1 res/lti.html#post--d2l-api-le-(version)-lti-link-(orgUnitId) -",
            "/d2l/api/le/(version)/lti/link/(orgUnitId)/(linkId)/sharing/ http:delete 1 res/lti.html#delete--d2l-api-le-(version)-lti-link-(orgUnitId)-(linkId)-sharing- -",
            "/d2l/api/le/(version)/lti/tp/(tpId) http:put 1 res/lti.html#put--d2l-api-le-(version)-lti-tp-(tpId) -",
            "psi4.driver.driver_nbody.ManyBodyComputer py:pydantic_model 1 nbody.html#$ -",
            "build-finished std:event 1 extdev/event_callbacks.html#event-$ -",
            "psi4.driver.driver_nbody.ManyBodyComputer.set_molecule py:pydantic_validator 1 nbody.html#$ -",
            "pySPACE.missions.support.windower.Windower._load_window_spec py:classmethod 1 api/generated/pySPACE.missions.support.windower.html#$ -",
            "--input std:option 1 tools/cgfx2json.html#cmdoption-cgfx2json$ -",
            "variable:ExternalData_NO_SYMLINKS cmake:variable 1 module/ExternalData.html#$ -",
            "prop_tgt:XCODE_SCHEME_THREAD_SANITIZER_STOP cmake:prop_tgt 1 prop_tgt/XCODE_SCHEME_THREAD_SANITIZER_STOP.html#$ -",
            "module:CSharpUtilities cmake:module 1 module/CSharpUtilities.html#$ -",
            "policy:CMP0187 cmake:policy 1 policy/CMP0187.html#$ -",
            "command:qt4_generate_moc cmake:command 1 module/FindQt4.html#$ -",
            "/{db} http:head 1 api/database/common.html#head--db -",
            "/{db}/_design/{ddoc} http:copy 1 api/ddoc/common.html#copy--db-_design-ddoc -",
            "/{db}/_design/{ddoc}/_rewrite/{path} http:any 1 api/ddoc/rewrites.html#any--db-_design-ddoc-_rewrite-path -",
            "/{db}/_local/{docid} http:copy 1 api/local.html#copy--db-_local-docid -",
            "prop_gbl:CMAKE_ROLE cmake:prop_gbl 1 prop_gbl/CMAKE_ROLE.html#$ -",
            "prop_dir:CACHE_VARIABLES cmake:prop_dir 1 prop_dir/CACHE_VARIABLES.html#$ -",
            "manual:cpack(1) cmake:manual 1 manual/cpack.1.html#$ -",
            "prop_test:RUN_SERIAL cmake:prop_test 1 prop_test/RUN_SERIAL.html#$ -",
            "genex:VERSION_GREATER cmake:genex 1 manual/cmake-generator-expressions.7.html#$ -",
            "generator:CodeLite cmake:generator 1 generator/CodeLite.html#$ -",
            "envvar:RC cmake:envvar 1 envvar/RC.html#$ -",
            "cpack_gen:CPack DEB Generator cmake:cpack_gen 1 cpack_gen/deb.html#$ -",
            "prop_sf:Fortran_FORMAT cmake:prop_sf 1 prop_sf/Fortran_FORMAT.html#$ -",
            "Data::@data cpp:union 1 usage/domains/cpp.html#_CPPv4N4DataUt4_dataE Data::[anonymous]",
            "td::Iterator cpp:concept 1 usage/domains/cpp.html#_CPPv4I0ENSt8IteratorE -",
            "pyvista.plotting.opts.RepresentationType py:enum 1 api/plotting/_autosummary/pyvista.plotting.opts.RepresentationType.html#$ -",
            "prop_cache:MODIFIED cmake:prop_cache 1 prop_cache/MODIFIED.html#$ -",
            "guide:tutorial/In-Depth CMake Target Commands cmake:guide 1 guide/tutorial/In-Depth%20CMake%20Target%20Commands.html#$ -",
            "gevent._interfaces.IWatcher py:interface 1 api/gevent.hub.html#$ -",
            // "translations/zh_tw/admin-guide/readme:linux內核6.x版本 <http://kernel.org/> std:label -1 translations/zh_TW/admin-guide/README.html#linux6-x-http-kernel-org Linux內核6.x版本 <http://kernel.org/>",
        ];

        for ex in examples {
            let _ = parse_reference(ex, 0)?;
        }
        Ok(())
    }
}
