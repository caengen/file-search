use std::io::Error;
use std::{collections::HashSet, io::Cursor};

use zip::ZipArchive;

use crate::util::{read_needles_from_file, read_needles_from_mem};

pub fn parse_from_mem(
    needle_bytes: &[u8],
    haystack_bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let needles: Vec<(&str, &str)> = read_needles_from_mem(needle_bytes)?;
    println!("Searching accross {} contacts", needles.len());

    parse(&needles, haystack_bytes)
}

pub fn parse_from_path(
    needles_path: &str,
    haystack_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut needle_buf = String::new();
    let needles = read_needles_from_file(needles_path, &mut needle_buf);
    println!("Searching accross {} contacts", needles.len());

    println!("Extracting text from pdf: {}", haystack_path);
    let bytes: Vec<u8> = std::fs::read(&haystack_path)?;

    parse(&needles, &bytes)
}

pub fn parse(
    needles: &Vec<(&str, &str)>,
    haystack_bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let haystack = pdf_extract::extract_text_from_mem(&haystack_bytes).unwrap();

    println!("\nStarting search...");
    let matches = haystack.lines().fold(HashSet::new(), |mut acc, line| {
        let trimmed = line.trim();
        if trimmed.len() > 0 {
            println!("{}", trimmed);
            for needle in needles {
                if trimmed.contains(needle.0) {
                    acc.insert(needle);
                }
            }
        }

        acc
    });

    println!("Found {} matches", matches.len());
    for (i, match_) in matches.iter().enumerate() {
        println!("{}: {:?}", i + 1, match_);
    }

    Ok(())
}
