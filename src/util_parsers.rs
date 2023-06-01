use std::char::ParseCharError;

use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::ParseError;
use nom::sequence::*;
use nom::Err;
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
