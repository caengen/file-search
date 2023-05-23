use std::fs::File;
use std::io::Read;

use docx::document::Paragraph;
use docx::DocxFile;
use nom::bytes::complete::{tag_no_case, take_while};
use nom::character::complete::{alphanumeric1, char};
use nom::multi::many0;
use nom::sequence::delimited;
use nom::{branch, AsChar, IResult};
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

fn get_doc_name(archive: &mut ZipArchive<File>) -> Option<String> {
    let mut doc_name = None;
    let mut rels = archive.by_name("_rels/.rels").unwrap();
    let mut rels_buffer = String::new();
    rels.read_to_string(&mut rels_buffer).unwrap();

    let rel_xml = roxmltree::Document::parse(&rels_buffer).unwrap();

    for elem in rel_xml.descendants() {
        let mut found = false;
        for attr in elem.attributes() {
            if attr.name() == "Type" && attr.value() == AttributeType::OfficeDocument.as_str() {
                if let Some(target) = elem.attribute("Target") {
                    doc_name = Some(target.to_owned());
                }
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }

    doc_name
}

fn parse_tag_content(input: &str) -> IResult<&str, &str> {
    take_while(AsChar::is_alphanum)(input)
}

fn parse_xml_tag_begin(input: &str) -> IResult<&str, &str> {
    delimited(char('<'), alphanumeric1, char('>'))(input)
}

#[test]
fn parse_xml_tag_begin_test() {
    assert_eq!(Ok(("", "1")), parse_xml_tag_begin("<1>"));
    assert!(parse_xml_tag_begin("<>").is_err());
    assert_eq!(Ok(("", "span")), parse_xml_tag_begin("<span>"));
}

fn parse_xml_tag_end(input: &str) -> IResult<&str, &str> {
    delimited(tag_no_case("</"), alphanumeric1, char('>'))(input)
}
#[test]
fn parse_xml_tag_end_test() {
    assert!(parse_xml_tag_end("<1>").is_err());
    assert!(parse_xml_tag_end("<>").is_err());
    assert_eq!(Ok(("", "span")), parse_xml_tag_end("</span>"));
}

fn parse_xml_string_content(input: &str) -> IResult<&str, Vec<&str>> {
    many0(branch::alt((
        delimited(parse_xml_tag_begin, parse_tag_content, parse_xml_tag_end),
        delimited(parse_xml_tag_begin, parse_tag_content, parse_xml_tag_begin),
    )))(input)
}

#[test]
fn parse_xml_string_content_test() {
    assert_eq!(
        Ok(("", vec!["test"])),
        parse_xml_string_content("<1>test</1>")
    );
    assert_eq!(
        Ok(("", vec!["test, dette, her"])),
        parse_xml_string_content("<a>test<sp>dette</sp><str>her</str></a>")
    );
}

pub fn parse(_search_term: &String, file_path: &String) -> Result<String, &'static str> {
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
                let parsed = parse_xml_string_content(doc.input_text());
                println!("{:?}", parsed);
            }
            Err(_) => {
                return Err("Could not parse XML tree");
            }
        }
        // for elem in doc.input_text() {
        //     println!("{:?}", elem);
        // }
    } else {
        return Err("Could not find document name");
    }

    Ok("".to_owned())
}
