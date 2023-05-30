use std::env;
use std::time::Instant;

use nom::bytes::complete::take_until;
use nom::bytes::streaming::tag;
use nom::character::complete::alphanumeric1;
use nom::combinator::value;
use nom::sequence::terminated;
use nom::IResult;

mod docx_parser;

#[derive(Clone, Debug, PartialEq)]
enum FileType {
    Docx,
    Pdf,
}

fn parse_filetype(file_path: &str) -> IResult<&str, FileType> {
    nom::branch::alt((
        value(FileType::Docx, take_until(".docx")),
        value(FileType::Pdf, take_until(".pdf")),
    ))(file_path)
}

#[test]
fn parse_filetype_test() {
    assert_eq!(
        parse_filetype("test-fil.docx"),
        Ok((".docx", FileType::Docx))
    );
    assert_eq!(parse_filetype("testfil.pdf"), Ok((".pdf", FileType::Pdf)));
    assert_eq!(parse_filetype("test fil.pdf"), Ok((".pdf", FileType::Pdf)));
    assert!(parse_filetype("testfilær.txt").is_err());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    println!("\nProgram args: {:?}\n", args);

    let contacts_path = &args[1];
    let file_path = &args[2];

    let start = Instant::now();
    let result = match parse_filetype(file_path) {
        Ok((_, FileType::Docx)) => docx_parser::parse(contacts_path, file_path),
        Ok((_, FileType::Pdf)) => Err("Pdf parsing not implemented"),
        Err(_) => Err("Unsupported file type"),
    };
    let duration = start.elapsed();
    println!(
        "\nParse & search execution took {} ms",
        duration.as_millis()
    );

    println!("{:?}", result);
}
