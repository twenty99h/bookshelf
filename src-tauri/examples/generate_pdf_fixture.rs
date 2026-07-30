#[macro_use]
extern crate lopdf;

use lopdf::content::{Content, Operation};
use lopdf::{Bookmark, Document, Object, Stream};

fn main() {
    let mut document = Document::with_version("1.7");
    let pages_id = document.new_object_id();
    let font_id = document.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica", "Encoding" => "WinAnsiEncoding",
    });
    let resources_id =
        document.add_object(dictionary! { "Font" => dictionary! { "F1" => font_id } });
    let mut page_ids = Vec::new();
    for index in 0..2 {
        let text = format!(
            "Chapter {}: {}",
            index + 1,
            "compressed text layer ".repeat(40)
        );
        let content = Content {
            operations: vec![
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 12.into()]),
                Operation::new("Td", vec![72.into(), (720 - index * 40).into()]),
                Operation::new(
                    "TJ",
                    vec![Object::Array(vec![
                        Object::string_literal(text),
                        (-25).into(),
                    ])],
                ),
                Operation::new("ET", vec![]),
            ],
        };
        let mut stream = Stream::new(dictionary! {}, content.encode().unwrap());
        stream.compress().unwrap();
        let content_id = document.add_object(stream);
        let page_id = document.add_object(
            dictionary! { "Type" => "Page", "Parent" => pages_id, "Contents" => content_id },
        );
        page_ids.push(page_id);
        document.add_bookmark(
            Bookmark::new(
                format!("Chapter {}", index + 1),
                [0.2, 0.3, 0.4],
                0,
                page_id,
            ),
            None,
        );
    }
    document.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages",
            "Kids" => page_ids.iter().copied().map(Object::Reference).collect::<Vec<_>>(),
            "Count" => page_ids.len() as i64,
            "Resources" => resources_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
        }),
    );
    let catalog_id = document.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    document.trailer.set("Root", catalog_id);
    if let Some(outline_id) = document.build_outline() {
        document
            .get_object_mut(catalog_id)
            .unwrap()
            .as_dict_mut()
            .unwrap()
            .set("Outlines", outline_id);
    }
    document.compress();
    document
        .save("tests/fixtures/pdf/compressed-outline-fonts.pdf")
        .unwrap();
}
