use std::char::ParseCharError;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;

use nom::bytes::complete::*;
use nom::character::complete::*;
use nom::combinator::*;
use nom::error::ParseError;
use nom::sequence::*;
use nom::Err;
use nom::IResult;
use zip::ZipArchive;
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

#[derive(Clone, Debug)]
struct Paragraph {
    text: String, // runs: Vec<Run>,
}

#[derive(Clone, Debug)]
struct Run {
    text: String,
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

fn sp(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c.is_whitespace())(input)
}

fn parse_str(input: &str) -> IResult<&str, &str> {
    escaped(alphanumeric0, '\\', one_of("\"\\"))(input)
}

#[test]
fn parse_str_test() {
    assert_eq!(Ok(("", "test")), parse_str("\"test\""));
}
type Contact<'a> = (&'a str, &'a str);

fn contact_from_str<'a>(input: (&'a str, &'a str)) -> Result<Contact<'a>, ParseCharError> {
    Ok(input)
}

fn parse_contact_line(input: &str) -> IResult<&str, (&str, &str)> {
    separated_pair(is_not(","), char(','), is_not("\n"))(input)
}

fn parse_contact(input: &str) -> IResult<&str, Contact> {
    map_res(parse_contact_line, contact_from_str)(input)
}

// The largest chunk of time is spent on parsing the XML tree of the document.
// Maybe this can be simplified with nom?
// Time complexity of search is O(n^2) ?
// Contact list of n elements * Two-Way matching of input.contains (n)
// todo: clean up
pub fn parse(contacts_path: &String, file_path: &String) -> Result<String, &'static str> {
    let mut needles_file = File::open(contacts_path).unwrap();
    let mut buf = String::new();
    let _ = needles_file.read_to_string(&mut buf).unwrap();
    let contacts = buf.lines().fold(Vec::new(), |mut acc, line| {
        if let Ok((_, contact)) = parse_contact(line) {
            acc.push(contact);
        }
        acc
    });
    println!("Found {} contacts", contacts.len());

    let file = File::open(file_path).unwrap();
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
                    if elem.has_tag_name("p") {
                        elem.descendants().for_each(|elem| {
                            if elem.has_tag_name("r") {
                                elem.descendants().for_each(|elem| {
                                    if elem.has_tag_name("t") {
                                        if let Some(text) = elem.text() {
                                            acc.push(text);
                                        }
                                    }
                                });
                            }
                        });
                    }

                    acc
                });

                println!("\nStarting search...");
                let matches = haystack.iter().fold(HashSet::new(), |mut acc, substack| {
                    for needle in &contacts {
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
                return Err("Could not parse XML tree");
            }
        }
    } else {
        return Err("Could not find document name");
    }

    Ok("Success".to_owned())
}
