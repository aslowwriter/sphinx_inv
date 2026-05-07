use crate::{
    error::SphinxParseError,
    priority::SphinxPriority,
    roles::{SphinxType, role_domain},
};
use winnow::{
    ModalResult, Parser,
    ascii::till_line_ending,
    combinator::{alt, opt, preceded, repeat_till, trace},
    stream::AsChar,
    token::take_while,
};

// basically just a wrapper so the type system can keep track of whether it's minified or not for us
#[derive(Debug, PartialEq)]
enum ReferenceString {
    Minified(String),
    Expanded(String),
}

#[derive(Debug)]
pub struct SphinxReference {
    pub name: String,
    // type is a reserved keyword
    pub sphinx_type: SphinxType,
    pub priority: SphinxPriority,
    location: ReferenceString,
    display_name: ReferenceString,
}

impl PartialEq for SphinxReference {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.sphinx_type == other.sphinx_type
            && self.priority == other.priority
            && self.expanded_location() == other.expanded_location()
            && self.expanded_display_name() == other.expanded_display_name()
    }
}

impl SphinxReference {
    pub fn new(
        name: &str,
        sphinx_type: SphinxType,
        priority: SphinxPriority,
        location: &str,
        display_name: &str,
    ) -> Self {
        let loc = if location.ends_with("#$") {
            ReferenceString::Minified(location.to_string())
        } else {
            ReferenceString::Expanded(location.to_string())
        };

        let disp_name = if display_name == "-" {
            ReferenceString::Minified(display_name.to_string())
        } else {
            ReferenceString::Expanded(display_name.to_string())
        };
        Self {
            name: name.to_string(),
            sphinx_type,
            priority,
            location: loc,
            display_name: disp_name,
        }
    }

    pub fn expanded_location(&self) -> String {
        match &self.location {
            ReferenceString::Expanded(s) => s.clone(),
            ReferenceString::Minified(s) => s.replace('$', &self.name),
        }
    }

    pub fn expanded_display_name(&self) -> String {
        match &self.display_name {
            ReferenceString::Expanded(s) => s.clone(),
            ReferenceString::Minified(_) => self.name.clone(),
        }
    }
    pub fn minified_location(&self) -> String {
        match &self.location {
            ReferenceString::Minified(s) => s.clone(),
            ReferenceString::Expanded(s) => match s.split_once('#') {
                Some((prefix, _suffix)) => format!("{prefix}#$"),
                None => s.clone(),
            },
        }
    }

    pub fn minified_display_name(&self) -> String {
        match &self.display_name {
            ReferenceString::Minified(s) => s.clone(),
            ReferenceString::Expanded(_s) => "-".to_string(),
        }
    }

    pub fn fmt_expanded(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.name,
            self.sphinx_type,
            self.priority,
            self.expanded_location(),
            self.expanded_display_name()
        )
    }

    pub fn fmt_minified(&self) -> String {
        format!(
            "{} {} {} {} {}",
            self.name,
            self.sphinx_type,
            self.priority,
            self.minified_location(),
            self.minified_display_name()
        )
    }
}

pub(crate) fn word<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
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

fn priority(input: &mut &str) -> ModalResult<SphinxPriority> {
    preceded(" ", alt(("-1", "1", "0", "2")))
        .parse_to()
        .parse_next(input)
}

fn uri<'s>(input: &mut &'s str) -> ModalResult<Option<&'s str>> {
    trace("uri", preceded(" ", opt(non_space))).parse_next(input)
}

fn display_name<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    trace("display_name", preceded(" ", till_line_ending)).parse_next(input)
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
    let ((name, sphinx_type), prio, loc, dispname) =
        (name_domain_role, priority, uri, display_name)
            .parse(line)
            .map_err(|error| SphinxParseError::from_str_parse(&error, line_num))?;

    Ok(SphinxReference::new(
        &name,
        sphinx_type,
        prio,
        loc.unwrap_or_default(),
        dispname,
    ))
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
        assert_eq!(
            sphinx_ref.location,
            ReferenceString::Expanded("library/stdtypes.html".to_string())
        );
        assert_eq!(
            sphinx_ref.display_name,
            ReferenceString::Expanded("asdf".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_index_line() -> Result<(), SphinxParseError> {
        let input = "index std:doc -1  Furo".to_string();

        let sphinx_ref = parse_reference(&input, 0)?;
        assert_eq!(sphinx_ref.name, "index".to_string());
        assert_eq!(sphinx_ref.sphinx_type, SphinxType::Std(StdRole::Doc));
        assert_eq!(sphinx_ref.priority, SphinxPriority::Omit);
        assert_eq!(
            sphinx_ref.location,
            ReferenceString::Expanded(String::new())
        );
        assert_eq!(
            sphinx_ref.display_name,
            ReferenceString::Expanded("Furo".to_string())
        );

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
        assert_eq!(
            sphinx_ref.location,
            ReferenceString::Minified("library/stdtypes.html#$".to_string())
        );
        assert_eq!(
            sphinx_ref.display_name,
            ReferenceString::Minified("-".to_string())
        );

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
                "invalid missing domain:role\nexpected `std`, `py`, `c`, `rst`, `cpp`, `js`, `math`",
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
        let input = "str.join\n py:method 1 library/stdtypes.html#$ -";

        let result = parse_reference(input, 0);
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_example_record() -> Result<(), SphinxParseError> {
        let input = "str.join py:method 1 library/stdtypes.html#$ -".to_string();

        let sphinx_ref = parse_reference(&input, 0)?;
        assert_eq!(sphinx_ref.name, "str.join".to_string());
        assert_eq!(sphinx_ref.sphinx_type, SphinxType::Python(PyRole::Method));
        assert_eq!(sphinx_ref.priority, SphinxPriority::Standard);
        assert_eq!(
            sphinx_ref.location,
            ReferenceString::Minified("library/stdtypes.html#$".to_string())
        );
        assert_eq!(
            sphinx_ref.display_name,
            ReferenceString::Minified("-".to_string())
        );

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
            ReferenceString::Expanded(
                "accel/qaic/aic080.html#qualcomm-cloud-ai-80-aic080".to_string()
            )
        );
        assert_eq!(
            sphinx_ref.display_name,
            ReferenceString::Expanded("Qualcomm Cloud AI 80 (AIC080)".to_string())
        );

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
            ReferenceString::Expanded(
                "accel/qaic/aic080.html#qualcomm-cloud-ai-80-aic080".to_string()
            )
        );
        assert_eq!(
            sphinx_ref.display_name,
            ReferenceString::Expanded("Qualcomm Cloud AI 80 (AIC080)".to_string())
        );

        Ok(())
    }

    #[test]
    fn new_reference() {
        assert_eq!(
            SphinxReference {
                name: "foo".to_string(),
                sphinx_type: SphinxType::C(CRole::Macro),
                priority: SphinxPriority::Standard,
                location: ReferenceString::Expanded("foo/bar".to_string()),
                display_name: ReferenceString::Minified("-".to_string())
            },
            SphinxReference::new(
                "foo",
                SphinxType::C(CRole::Macro),
                SphinxPriority::Standard,
                "foo/bar",
                "-"
            )
        );
    }
}
