use std::fs::File;
use std::io::Read;

use docx::document::Paragraph;
use docx::DocxFile;
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

pub fn parse(_search_term: &String, file_path: &String) -> Result<String, &'static str> {
    let file = File::open(file_path).unwrap();
    let mut archive = ZipArchive::new(file).unwrap();

    let doc_name = get_doc_name(&mut archive);

    if let Some(doc_name) = doc_name {
        println!("Found document name: {}", doc_name);
        let mut document = archive.by_name(&doc_name).unwrap();
        let mut buffer = String::new();
        document.read_to_string(&mut buffer).unwrap();

        let doc = roxmltree::Document::parse(&buffer).unwrap();
        println!("{:?}", doc.input_text());
        // for elem in doc.input_text() {
        //     println!("{:?}", elem);
        // }
    } else {
        return Err("Could not find document name");
    }

    Ok("".to_owned())
}
