use std::collections::HashSet;
use std::fs::File;
use std::io::{Error, ErrorKind, Read};

use zip::ZipArchive;

use crate::read_needles_from_file;
use crate::util::parse_contact;
enum AttributeType {
    OfficeDocument,
}
impl AttributeType {
    fn as_str(&self) -> &'static str {
        match self {
            AttributeType::OfficeDocument => {
                "http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument"
            }
        }
    }
}

fn get_doc_name(archive: &mut ZipArchive<File>) -> Option<String> {
    let mut doc_name = None;
    let mut rels = archive.by_name("_rels/.rels").unwrap();
    let mut rels_buffer = String::new();
    rels.read_to_string(&mut rels_buffer).unwrap();

    let rel_xml = roxmltree::Document::parse(&rels_buffer).unwrap();

    for elem in rel_xml.descendants() {
        'outer: for attr in elem.attributes() {
            if attr.name() == "Type" && attr.value() == AttributeType::OfficeDocument.as_str() {
                if let Some(target) = elem.attribute("Target") {
                    doc_name = Some(target.to_owned());
                }
                break 'outer;
            }
        }
    }

    doc_name
}

// The largest chunk of time is spent on parsing the XML tree of the document.
// Maybe this can be simplified with nom?
// Time complexity of search is O(n^2) ?
// Contact list of n elements * Two-Way matching of input.contains (n)
// todo: clean up
pub fn parse(needle_path: &String, file_path: &String) -> Result<(), Error> {
    let mut needle_buf = String::new();
    let needles = read_needles_from_file!(needle_path, needle_buf);
    println!("Searching accross {} contacts", needles.len());

    let file: File = File::open(file_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    let doc_name = get_doc_name(&mut archive);

    if let Some(doc_name) = doc_name {
        println!("Found document name: {}", doc_name);

        let mut document = archive.by_name(&doc_name).unwrap();
        let mut buffer = String::new();
        document.read_to_string(&mut buffer).unwrap();
        let doc = roxmltree::Document::parse(&buffer);

        match doc {
            Ok(doc) => {
                let root = doc.root().first_child().unwrap();
                let body = root.first_element_child().unwrap();
                let haystack = body.descendants().fold(Vec::new(), |mut acc, elem| {
                    // check if it has a paragraph tag
                    if !elem.has_tag_name("p") {
                        return acc;
                    }

                    elem.descendants().for_each(|elem| {
                        // check if it has a run tag
                        if !elem.has_tag_name("r") {
                            return;
                        }
                        elem.descendants().for_each(|elem| {
                            // check if it has a text tag
                            if !elem.has_tag_name("t") {
                                return;
                            }

                            // check if it has text
                            if let Some(text) = elem.text() {
                                acc.push(text);
                            }
                        });
                    });

                    acc
                });

                println!("\nStarting search...");
                let matches = haystack.iter().fold(HashSet::new(), |mut acc, substack| {
                    for needle in &needles {
                        if substack.contains(needle.0) {
                            acc.insert(needle);
                        }
                    }
                    acc
                });

                println!("Found {} matches", matches.len());
                for (i, match_) in matches.iter().enumerate() {
                    println!("{}: {:?}", i + 1, match_);
                }
            }
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    "Could not parse XML tree",
                ));
            }
        }
    } else {
        return Err(Error::new(
            ErrorKind::NotFound,
            "Could not find document name",
        ));
    }

    Ok(())
}
