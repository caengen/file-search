use std::env;
use std::time::Instant;

use nom::bytes::streaming::tag;
use nom::character::complete::alphanumeric1;
use nom::combinator::value;
use nom::sequence::terminated;
use nom::IResult;

mod docx;

#[derive(Clone, Debug, PartialEq)]
enum FileType {
    Docx,
    Pdf,
}

fn parse_filetype(file_path: &str) -> IResult<&str, FileType> {
    nom::branch::alt((
        value(FileType::Docx, terminated(alphanumeric1, tag(".docx"))),
        value(FileType::Pdf, terminated(alphanumeric1, tag(".pdf"))),
    ))(file_path)
}

#[test]
fn parse_filetype_test() {
    assert_eq!(parse_filetype("testfil.docx"), Ok(("", FileType::Docx)));
    assert_eq!(parse_filetype("testfil.pdf"), Ok(("", FileType::Pdf)));
    assert!(parse_filetype("testfilær.txt").is_err());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("Running with args: {:?}", args);

    let term = &args[1];
    let file_path = &args[2];

    let start = Instant::now();
    let result = match parse_filetype(file_path) {
        Ok((_, FileType::Docx)) => docx::parse(term, file_path),
        Ok((_, FileType::Pdf)) => Err("Pdf parsing not implemented"),
        Err(_) => Err("Unsupported file type"),
    };
    let duration = start.elapsed();
    println!("Operation took {} ms", duration.as_millis());

    println!("Result: {:?}", result);
}
