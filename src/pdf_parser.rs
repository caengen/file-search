use std::{
    collections::HashSet,
    io::{Error, ErrorKind},
};

use crate::util::{read_needles_from_file, read_needles_from_mem};

pub fn parse_from_mem(
    needle_bytes: &[u8],
    haystack_bytes: &[u8],
) -> Result<HashSet<(String, String)>, Box<dyn std::error::Error>> {
    let needles: Vec<(&str, &str)> = read_needles_from_mem(needle_bytes)?;
    println!("Searching accross {} contacts", needles.len());

    parse(&needles, haystack_bytes)
}

pub fn parse_from_path(
    needles_path: &str,
    haystack_path: &str,
) -> Result<HashSet<(String, String)>, Box<dyn std::error::Error>> {
    let mut needle_buf = String::new();
    let needles = read_needles_from_file(needles_path, &mut needle_buf);
    println!("Searching accross {} contacts", needles.len());

    println!("Extracting text from pdf: {}", haystack_path);
    let bytes: Vec<u8> = std::fs::read(&haystack_path)?;

    parse(&needles, &bytes)
}

fn parse(
    needles: &Vec<(&str, &str)>,
    haystack_bytes: &[u8],
) -> Result<HashSet<(String, String)>, Box<dyn std::error::Error>> {
    let haystack = pdf_extract::extract_text_from_mem(&haystack_bytes)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed to extract text from pdf"))?;

    println!("\nStarting search...");
    let matches = haystack.lines().filter(|line| line.trim().len() > 0).fold(
        HashSet::new(),
        |mut acc, line| {
            needles.iter().filter(|n| line.contains(n.0)).for_each(|n| {
                acc.insert((n.0.to_owned(), n.1.to_owned()));
            });

            acc
        },
    );

    println!("Found {} matches", matches.len());
    Ok(matches)
}
