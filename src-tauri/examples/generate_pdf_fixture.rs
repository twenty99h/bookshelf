#[macro_use]
extern crate lopdf;

use lopdf::content::{Content, Operation};
use lopdf::{Bookmark, Document, Object, Stream};

fn main() {
    let mut document = Document::with_version("1.7");
    let pages_id = document.new_object_id();
    let embedded_font_file = document.add_object(Stream::new(
        dictionary! { "Length1" => 8 },
        b"BOOKFONT".to_vec(),
    ));
    let font_descriptor = document.add_object(dictionary! {
        "Type" => "FontDescriptor", "FontName" => "BookshelfCorpusFont", "Flags" => 32,
        "FontBBox" => vec![0.into(), (-200).into(), 1000.into(), 900.into()],
        "ItalicAngle" => 0, "Ascent" => 800, "Descent" => -200, "CapHeight" => 700,
        "StemV" => 80, "FontFile2" => embedded_font_file,
    });
    let font_id = document.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "TrueType", "BaseFont" => "BookshelfCorpusFont",
        "Encoding" => "WinAnsiEncoding", "FirstChar" => 32, "LastChar" => 255,
        "Widths" => vec![600.into(); 224], "FontDescriptor" => font_descriptor,
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
        if index == 0 {
            operations.extend([
                Operation::new("BT", vec![]),
                Operation::new("Tf", vec!["F1".into(), 10.into()]),
                Operation::new(
                    "Tm",
                    vec![
                        1.into(),
                        0.into(),
                        0.into(),
                        1.into(),
                        320.into(),
                        380.into(),
                    ],
                ),
                Operation::new("Tj", vec![Object::string_literal("COLUMN_RIGHT")]),
                Operation::new(
                    "Tm",
                    vec![
                        1.into(),
                        0.into(),
                        0.into(),
                        1.into(),
                        72.into(),
                        380.into(),
                    ],
                ),
                Operation::new("Tj", vec![Object::string_literal("COLUMN_LEFT")]),
                Operation::new(
                    "Tm",
                    vec![
                        1.into(),
                        0.into(),
                        0.into(),
                        1.into(),
                        72.into(),
                        350.into(),
                    ],
                ),
                Operation::new(
                    "Tj",
                    vec![Object::string_literal("FORMULA_SUM: sigma(i=1..n) x_i")],
                ),
                Operation::new(
                    "Tm",
                    vec![
                        1.into(),
                        0.into(),
                        0.into(),
                        1.into(),
                        72.into(),
                        310.into(),
                    ],
                ),
                Operation::new(
                    "Tj",
                    vec![Object::string_literal("VISUAL_SECOND_STORED_FIRST")],
                ),
                Operation::new(
                    "Tm",
                    vec![
                        1.into(),
                        0.into(),
                        0.into(),
                        1.into(),
                        72.into(),
                        330.into(),
                    ],
                ),
                Operation::new(
                    "Tj",
                    vec![Object::string_literal("VISUAL_FIRST_STORED_SECOND")],
                ),
                Operation::new("ET", vec![]),
            ]);
        }
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
        if let Some(destination) = document.objects.values_mut().find_map(|object| {
            object
                .as_dict_mut()
                .ok()
                .and_then(|dictionary| dictionary.get_mut(b"D").ok())
                .and_then(|value| value.as_array_mut().ok())
        }) {
            destination[0] = Object::Reference((999, 0));
        }
    }
    document.compress();
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "tests/fixtures/pdf/compressed-outline-fonts.pdf".into());
    document.save(output).unwrap();
}
