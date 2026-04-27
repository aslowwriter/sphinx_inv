use crate::{
    priority::SphinxPriority,
    roles::{SphinxType, c_role, cpp_role, js_role, math_role, py_role, rst_role, std_role},
};
use winnow::{
    ModalResult, Parser,
    ascii::{digit1, space1, till_line_ending},
    combinator::{alt, dispatch, fail, opt, preceded, repeat_till, terminated, trace},
    stream::AsChar,
    token::take_while,
};

#[derive(Debug)]
pub struct SphinxReference {
    pub name: String,
    // type is a reserved keyword
    pub sphinx_type: SphinxType,
    pub priority: SphinxPriority,
    pub location: String,
    pub display_name: String,
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
        "std" => std_role,
        "py" => py_role,
        "c" => c_role,
        "rst" => rst_role,
        "cpp" => cpp_role,
        "js" => js_role,
        "math" => math_role,
        _ => fail
    }
    .parse_next(input)
}

fn priority(input: &mut &str) -> ModalResult<i32> {
    let (sign, num) = trace(
        "priority",
        preceded(space1, (opt('-'), digit1.parse_to::<i32>())),
    )
    .parse_next(input)?;
    if sign.is_some() { Ok(-num) } else { Ok(num) }
}

fn uri<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    trace("uri", preceded(space1, non_space)).parse_next(input)
}

fn display_name<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    trace("display_name", preceded(space1, till_line_ending)).parse_next(input)
}

fn name_domain_role(input: &mut &str) -> ModalResult<(String, SphinxType)> {
    let (mut prefix_vec, role): (String, SphinxType) = trace(
        "name_domain_role",
        repeat_till(0.., alt((word, non_word)), role_domain),
    )
    .parse_next(input)?;
    // the last space was separating the title and the domain, so we pop that off
    let _ = prefix_vec.pop();
    Ok((prefix_vec, role))
}

///# Errors
pub fn reference(input: &mut &str) -> ModalResult<SphinxReference> {
    let (name, sphinx_type) = name_domain_role.parse_next(input)?;
    let prio = trace("priority", priority)
        .try_map(SphinxPriority::try_from)
        .parse_next(input)?;
    let location = trace("location", uri).parse_next(input)?;
    let dispname = trace("display_name", display_name).parse_next(input)?;

    let display_name = if dispname == "-" {
        name.clone()
    } else {
        dispname.to_string()
    };
    let location = location.replace('$', &name);
    Ok(SphinxReference {
        name: name.clone(),
        sphinx_type,
        priority: prio,
        location,
        display_name,
    })
}

#[cfg(test)]
mod test {

    use crate::roles::{PyRole, RstRole, StdRole};

    use super::*;

    #[test]
    fn test_hard_dummy_record() -> ModalResult<()> {
        let mut input = "asdfasdf :foo std::endl :: _bar_baz: something- : hello std:label 1 library/stdtypes.html asdf";

        let sphinx_ref = reference(&mut input)?;

        assert_eq!(
            sphinx_ref.name,
            "asdfasdf :foo std::endl :: _bar_baz: something- : hello".to_string()
        );
        assert_eq!(sphinx_ref.sphinx_type, SphinxType::Std(StdRole::Label));
        assert_eq!(sphinx_ref.priority, SphinxPriority::Standard);
        assert_eq!(sphinx_ref.location, "library/stdtypes.html");
        assert_eq!(sphinx_ref.display_name, "asdf");

        assert_eq!(input, "");
        Ok(())
    }
    #[test]
    fn test_parse_example_record_with_rst_directive() -> ModalResult<()> {
        let mut input = "str.join rst:directive:option 1 library/stdtypes.html#$ -";

        let sphinx_ref = reference(&mut input)?;
        assert_eq!(sphinx_ref.name, "str.join".to_string());
        assert_eq!(
            sphinx_ref.sphinx_type,
            SphinxType::ReStructuredText(RstRole::Option)
        );
        assert_eq!(sphinx_ref.priority, SphinxPriority::Standard);
        assert_eq!(sphinx_ref.location, "library/stdtypes.html#str.join");
        assert_eq!(sphinx_ref.display_name, "str.join");

        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_parse_example_record_with_newline() {
        let mut input = "str.join\n py:method 1 library/stdtypes.html#$ -";

        let result = reference(&mut input);
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_example_record() -> ModalResult<()> {
        let mut input = "str.join py:method 1 library/stdtypes.html#$ -";

        let sphinx_ref = reference(&mut input)?;
        assert_eq!(sphinx_ref.name, "str.join".to_string());
        assert_eq!(sphinx_ref.sphinx_type, SphinxType::Python(PyRole::Method));
        assert_eq!(sphinx_ref.priority, SphinxPriority::Standard);
        assert_eq!(sphinx_ref.location, "library/stdtypes.html#str.join");
        assert_eq!(sphinx_ref.display_name, "str.join");

        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_lkd_hard_line_with_rst_directive() -> ModalResult<()> {
        let mut input = "accel/qaic/aic080:qualcomm cloud ai 80 (aic080) rst:directive:option -1 accel/qaic/aic080.html#qualcomm-cloud-ai-80-aic080 Qualcomm Cloud AI 80 (AIC080)";

        let sphinx_ref = reference(&mut input)?;
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

        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_lkd_hard_line() -> ModalResult<()> {
        let mut input = "accel/qaic/aic080:qualcomm cloud ai 80 (aic080) std:label -1 accel/qaic/aic080.html#qualcomm-cloud-ai-80-aic080 Qualcomm Cloud AI 80 (AIC080)";

        let sphinx_ref = reference(&mut input)?;
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

        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_cmake_example() {
        let mut input = "command:add_compile_definitions cmake:command 1 command/add_compile_definitions.html#$ -\nstr.join py:method 1 library/stdtypes.html#$ -";
        let result = reference(&mut input);

        assert!(result.is_err());
    }
}
