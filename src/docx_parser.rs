use std::collections::HashSet;
use std::fs::File;
use std::io::{Cursor, Error, ErrorKind, Read};
use zip::ZipArchive;

use crate::util::{read_needles_from_file, read_needles_from_mem};
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

fn get_doc_name<R>(archive: &mut ZipArchive<R>) -> Option<String>
where
    R: std::io::Seek,
    R: std::io::Read,
{
    let mut doc_name = None;
    let names: Vec<_> = archive.file_names().collect();
    println!("Found {} files in archive, {:?}", names.len(), names);
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

pub fn parse_from_mem(
    needle_bytes: &[u8],
    haystack_bytes: &[u8],
) -> Result<HashSet<(String, String)>, Box<dyn std::error::Error>> {
    let needles = read_needles_from_mem(needle_bytes)?;
    println!("Searching accross {} contacts", needles.len());

    let haystack_reader = Cursor::new(haystack_bytes);
    let mut archive = ZipArchive::new(haystack_reader)?;

    parse(&needles, &mut archive)
}

// The largest chunk of time is spent on parsing the XML tree of the document.
// Maybe this can be simplified with nom?
// Time complexity of search is O(n^2) ?
// Contact list of n elements * Two-Way matching of input.contains (n)
// todo: clean up
pub fn parse_from_path(
    needle_path: &String,
    file_path: &String,
) -> Result<HashSet<(String, String)>, Box<dyn std::error::Error>> {
    let mut needle_buf = String::new();
    let needles = read_needles_from_file(needle_path, &mut needle_buf);
    println!("Searching accross {} contacts", needles.len());

    let file: File = File::open(file_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();
    parse(&needles, &mut archive)
}

fn parse<R>(
    needles: &Vec<(&str, &str)>,
    archive: &mut ZipArchive<R>,
) -> Result<HashSet<(String, String)>, Box<dyn std::error::Error>>
where
    R: std::io::Seek,
    R: std::io::Read,
{
    let doc_name = get_doc_name(archive).ok_or(Error::new(
        ErrorKind::NotFound,
        "Could not find document name",
    ))?;

    println!("Found document name: {}", doc_name);

    let mut document = archive
        .by_name(&doc_name)
        .map_err(|_| Error::new(ErrorKind::NotFound, "Could not find document in archive"))?;

    let mut buffer = String::new();
    document
        .read_to_string(&mut buffer)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Failed to write document to buffer"))?;

    let doc = roxmltree::Document::parse(&buffer)
        .map_err(|_| Error::new(ErrorKind::InvalidData, "Could not parse XML tree"))?;

    let root = doc.root().first_child().ok_or(Error::new(
        ErrorKind::InvalidData,
        "Could not find root node",
    ))?;

    let body = root
        .first_element_child()
        .ok_or(Error::new(ErrorKind::InvalidData, "Root node is empty"))?;

    let haystack = body
        .descendants()
        .filter(|elem| elem.has_tag_name("p"))
        .fold(Vec::new(), |mut acc, elem| {
            elem.descendants()
                .filter(|elem| elem.has_tag_name("r"))
                .for_each(|elem| {
                    // check if it has a run tag
                    elem.descendants()
                        .filter(|elem| elem.has_tag_name("t"))
                        .for_each(|elem| {
                            elem.text().and_then(|text| {
                                return Some(acc.push(text));
                            });
                        });
                });

            acc
        });

    println!("\nStarting search...");
    let matches = haystack.iter().fold(HashSet::new(), |mut acc, substack| {
        needles
            .iter()
            .filter(|needle| substack.contains(needle.0))
            .for_each(|needle| {
                acc.insert((needle.0.to_owned(), needle.1.to_owned()));
            });

        acc
    });

    println!("Found {} matches", matches.len());
    matches
        .iter()
        .enumerate()
        .for_each(|(i, match_)| println!("{}: {:?}", i + 1, match_));

    Ok(matches)
}
