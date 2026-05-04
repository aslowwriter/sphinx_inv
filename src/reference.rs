use std::fmt::Display;

use crate::{
    error::SphinxParseError,
    priority::SphinxPriority,
    roles::{SphinxType, c_role, cpp_role, js_role, math_role, py_role, rst_role, std_role},
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
        _ => cut_err(fail).context(StrContext::Label("unknown domain")).context(StrContext::Expected(StrContextValue::StringLiteral("std"))) .context(StrContext::Expected(StrContextValue::StringLiteral("py"))).context(StrContext::Expected(StrContextValue::StringLiteral("c"))).context(StrContext::Expected(StrContextValue::StringLiteral("rst"))).context(StrContext::Expected(StrContextValue::StringLiteral("cpp"))).context(StrContext::Expected(StrContextValue::StringLiteral("js"))).context(StrContext::Expected(StrContextValue::StringLiteral("math")))
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
                "invalid unknown domain\nexpected `std`, `py`, `c`, `rst`, `cpp`, `js`, `math`",
                14,
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
    fn test_cmake_example() {
        let mut input = "command:add_compile_definitions cmake:command 1 command/add_compile_definitions.html#$ -\nstr.join py:method 1 library/stdtypes.html#$ -";
        let result = reference(&mut input);

        assert!(result.is_err());
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
}
