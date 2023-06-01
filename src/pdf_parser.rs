use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::util_parsers::parse_contact;

pub fn parse(needles_path: &str, haystack_path: &str) -> Result<(), Error> {
    let mut needles_file = File::open(needles_path)?;
    let mut buf = String::new();
    let _ = needles_file.read_to_string(&mut buf)?;
    let contacts: Vec<(&str, &str)> = buf.lines().fold(Vec::new(), |mut acc, line| {
        if let Ok((_, contact)) = parse_contact(line) {
            acc.push(contact);
        }
        acc
    });
    println!("Searching accross {} contacts", contacts.len());

    let bytes = std::fs::read(&haystack_path)?;

    println!("Extracting text from pdf: {}", haystack_path);
    let haystack = pdf_extract::extract_text_from_mem(&bytes).unwrap();

    println!("\nStarting search...");
    let matches = haystack.lines().fold(HashSet::new(), |mut acc, line| {
        let trimmed = line.trim();
        if trimmed.len() > 0 {
            println!("{}", trimmed);
            for needle in &contacts {
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
