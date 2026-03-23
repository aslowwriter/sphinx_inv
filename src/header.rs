use std::io::{BufRead, BufReader, Read};

use crate::error::InvalidHeaderError;

#[derive(Debug)]
pub struct SphinxInvHeader {
    pub project_name: String,
    pub project_version: String,
    pub sphinx_version: SphinxInvVersion,
}

#[derive(Debug, PartialEq)]
pub enum SphinxInvVersion {
    V1,
    V2,
}

fn parse_inv_version<R: Read>(
    reader: &mut BufReader<R>,
) -> Result<SphinxInvVersion, InvalidHeaderError> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    // this is how sphinx itself does it https://github.com/sphinx-doc/sphinx/blob/ac3f74a3e0fbb326f73989a16dfa369e072064ca/sphinx/util/inventory.py#L126
    // this part should only have ascii so we should be fine using just chars
    // even though they might be unicode
    let v: String = line.chars().skip(27).collect::<String>().trim().to_string();
    match v.parse() {
        Ok(1) => Ok(SphinxInvVersion::V1),
        Ok(2) => Ok(SphinxInvVersion::V2),
        _ => Err(InvalidHeaderError::InvalidSphinxVerison(v)),
    }
}

fn parse_inv_project_name<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    // this is how sphinx itself does it https://github.com/sphinx-doc/sphinx/blob/ac3f74a3e0fbb326f73989a16dfa369e072064ca/sphinx/util/inventory.py#L126
    let (_, proj_name) = line.split_at(11);

    Ok(proj_name.trim().to_string())
}

fn parse_inv_project_version<R: Read>(reader: &mut BufReader<R>) -> std::io::Result<String> {
    let mut line = String::new();
    reader.read_line(&mut line)?;
    // this is how sphinx itself does it https://github.com/sphinx-doc/sphinx/blob/ac3f74a3e0fbb326f73989a16dfa369e072064ca/sphinx/util/inventory.py#L126
    let (_, proj_version) = line.split_at(11);

    Ok(proj_version.trim().to_string())
}

pub(crate) fn parse_sphinx_inv_header<R: Read>(
    reader: &mut BufReader<R>,
) -> Result<SphinxInvHeader, InvalidHeaderError> {
    let inv_version = parse_inv_version(reader)?;
    if inv_version != SphinxInvVersion::V2 {
        return Err(InvalidHeaderError::UnsupportedSphinxVersion(inv_version));
    }
    let inv_project_name = parse_inv_project_name(reader)?;
    let inv_project_version = parse_inv_project_version(reader)?;
    let mut warning_header = String::new();
    reader.read_line(&mut warning_header)?;

    if !warning_header.contains("zlib") {
        return Err(InvalidHeaderError::InvalidCompressionMethod(warning_header));
    }

    let header = SphinxInvHeader {
        project_name: inv_project_name,
        project_version: inv_project_version,
        sphinx_version: inv_version,
    };

    Ok(header)
}
