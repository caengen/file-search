use std::char::ParseCharError;
use std::fs::File;
use std::io::Read;
use std::str::from_utf8;

use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::sequence::*;
use nom::IResult;

pub type Contact<'a> = (&'a str, &'a str);

// maybe want to use a struct later
fn contact_from_str<'a>(input: (&'a str, &'a str)) -> Result<Contact<'a>, ParseCharError> {
    Ok(input)
}

fn parse_contact_line(input: &str) -> IResult<&str, Contact> {
    separated_pair(is_not(","), char(','), is_not("\n"))(input)
}

pub fn parse_contact(input: &str) -> IResult<&str, Contact> {
    map_res(parse_contact_line, contact_from_str)(input)
}

#[derive(Clone, Debug, PartialEq)]
pub enum FileType {
    Docx,
    Pdf,
}

pub fn parse_filetype(file_path: &str) -> IResult<&str, FileType> {
    nom::branch::alt((
        value(FileType::Docx, take_until(".docx")),
        value(FileType::Pdf, take_until(".pdf")),
    ))(file_path)
}

pub fn read_needles_from_file<'a>(path: &str, buf: &'a mut String) -> Vec<Contact<'a>> {
    let mut needles_file = File::open(path).unwrap();
    let _ = needles_file.read_to_string(buf).unwrap();

    let needles = buf.lines().fold(Vec::new(), |mut acc, line| {
        if let Ok((_, contact)) = parse_contact(line) {
            acc.push(contact);
        }
        acc
    });

    needles
}
pub fn read_needles_from_mem(bytes: &[u8]) -> Result<Vec<Contact>, Box<dyn std::error::Error>> {
    let buf = from_utf8(bytes)?;

    let needles = buf.lines().fold(Vec::new(), |mut acc, line| {
        if let Ok((_, contact)) = parse_contact(line) {
            acc.push(contact);
        }
        acc
    });

    Ok(needles)
}
