use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::fs::File;
use std::io::{Error, ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::read_needles_from_file;
use crate::util_parsers::parse_contact;

pub fn parse(needles_path: &str, haystack_path: &str) -> Result<(), Error> {
    let mut needle_buf = String::new();
    let needles = read_needles_from_file!(needles_path, needle_buf);
    println!("Searching accross {} contacts", needles.len());

    let bytes = std::fs::read(&haystack_path)?;

    println!("Extracting text from pdf: {}", haystack_path);
    let haystack = pdf_extract::extract_text_from_mem(&bytes).unwrap();

    println!("\nStarting search...");
    let matches = haystack.lines().fold(HashSet::new(), |mut acc, line| {
        let trimmed = line.trim();
        if trimmed.len() > 0 {
            println!("{}", trimmed);
            for needle in &needles {
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
