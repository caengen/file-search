#![allow(dead_code)]
use std::env;
use std::io::{Error, ErrorKind};
use std::time::Instant;

use crate::util::{parse_filetype, FileType};

mod docx_parser;
mod pdf_parser;
#[macro_use]
mod util;

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

    // Expect the format of the needles file to be:
    // <search term>,<other unique identifier>
    // e.g.: "Ola Nordmann,ola.nordmann@epost.no"
    let needles_path = &args[1];
    let haystack_path = &args[2];

    let start = Instant::now();
    let result = match parse_filetype(haystack_path) {
        Ok((_, FileType::Docx)) => docx_parser::parse_from_path(needles_path, haystack_path),
        Ok((_, FileType::Pdf)) => pdf_parser::parse_from_path(needles_path, haystack_path),
        Err(_) => Err(Error::new(ErrorKind::Unsupported, "Unsupported file type").into()),
    };
    let duration = start.elapsed();
    println!(
        "\nParse & search execution took {} ms",
        duration.as_millis()
    );

    println!("{:?}", result);
}
