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
    let image_id = document.add_object(Stream::new(
        dictionary! {
            "Type" => "XObject", "Subtype" => "Image", "Width" => 2, "Height" => 2,
            "ColorSpace" => "DeviceRGB", "BitsPerComponent" => 8,
        },
        vec![214, 162, 74, 91, 86, 194, 91, 86, 194, 214, 162, 74],
    ));
    let resources_id = document.add_object(dictionary! {
        "Font" => dictionary! { "F1" => font_id },
        "XObject" => dictionary! { "Im1" => image_id },
    });
    let mut page_ids = Vec::new();
    for index in 0..10 {
        let crossing = if index == 4 {
            "Cross-page source begins here and"
        } else if index == 5 {
            "continues here without losing its source address."
        } else {
            "Deterministic bookshelf fixture."
        };
        let mut operations = vec![
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 20.into()]),
            Operation::new("Td", vec![72.into(), 760.into()]),
            Operation::new(
                "Tj",
                vec![Object::string_literal(format!(
                    "Chapter {}: Reliable replication",
                    index + 1
                ))],
            ),
            Operation::new("ET", vec![]),
            Operation::new("BT", vec![]),
            Operation::new("Tf", vec!["F1".into(), 12.into()]),
            Operation::new("TL", vec![22.into()]),
            Operation::new("Td", vec![72.into(), 710.into()]),
        ];
        for line in [
            "Repeated search phrase: leader replication and explicit ownership.",
            "A deterministic text layer keeps selection and search reproducible.",
            crossing,
            "The reader preserves the page and the authored source context.",
            "Failure handling belongs in the model rather than hidden infrastructure.",
            "A negative result is still a completed practical application.",
        ] {
            operations.push(Operation::new("Tj", vec![Object::string_literal(line)]));
            operations.push(Operation::new("T*", vec![]));
        }
        operations.extend([
            Operation::new("ET", vec![]),
            Operation::new("q", vec![]),
            Operation::new("re", vec![72.into(), 430.into(), 220.into(), 90.into()]),
            Operation::new("S", vec![]),
            Operation::new("Q", vec![]),
            Operation::new("q", vec![]),
            Operation::new(
                "cm",
                vec![
                    100.into(),
                    0.into(),
                    0.into(),
                    60.into(),
                    330.into(),
                    445.into(),
                ],
            ),
            Operation::new("Do", vec!["Im1".into()]),
            Operation::new("Q", vec![]),
        ]);
        let content = Content { operations };
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
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/fixtures/pdf/compressed-outline-fonts.pdf".into());
    document.save(output).unwrap();
}
