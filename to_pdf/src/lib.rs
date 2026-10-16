use std::path::Path;

use printpdf::{
    Line, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle, PdfPage,
    PdfSaveOptions, Point, Polygon, PolygonRing, Pt, TextItem, WindingOrder, XObjectTransform,
};
use resume_tui_data::data::source::DATABASE;
use wrapper::WithCalc;

static RESUME_FONT: &[u8] = include_bytes!("../../assets/IosevkaEtoile-Thin.ttc");
static GITHUB_IMG: &str = include_str!("../../assets/github.svg");
static LI_IMG: &str = include_str!("../../assets/linkedin.svg");
static PH_IMG: &str = include_str!("../../assets/phone.svg");
static MAIL_IMG: &str = include_str!("../../assets/mail.svg");

const LETTER_HEIGHT_MM: f32 = 279.0;
pub(crate) const LETTER_WIDTH_MM: f32 = 216.0;
const X_MARGIN: f32 = 10.0;
const INNER_X_MARGIN: f32 = 15.0;

mod wrapper;

pub fn generate_pdf(out_path: impl AsRef<Path>) -> Result<(), String> {
    let mut font_warnings: Vec<printpdf::PdfFontParseWarning> = Vec::new();
    let mut warnings: Vec<printpdf::PdfWarnMsg> = Vec::new();

    // Set up the document and its metadata. printpdf 0.12 stores author/title
    // in `PdfDocumentInfo` rather than via separate `with_*` builders.
    let mut doc = PdfDocument::new("resume");
    doc.metadata.info.document_title = "resume".to_string();
    doc.metadata.info.author = DATABASE.name.to_string();

    // Register the external font with printpdf. It will be embedded by
    // the document when we save.
    let parsed_font = ParsedFont::from_bytes(RESUME_FONT, 0, &mut font_warnings)
        .ok_or_else(|| "failed to parse RESUME_FONT".to_string())?;
    let font_id = doc.add_font(&parsed_font);

    // The wrapper still owns a ttf-parser face for measuring text widths
    // (used by `wrap_text` below). printpdf's own text shaper handles
    // actual rendering; we only need widths here.
    let face = wrapper::TtfFace::from_vec(RESUME_FONT.to_vec())
        .map_err(|e| format!("Error parsing font face: {e}"))?;
    let w = WithCalc::new(face);

    let mut ops: Vec<Op> = Vec::new();
    let mut current_y = LETTER_HEIGHT_MM - 20.0;

    // ---- Name (48 pt) ----
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(X_MARGIN), Mm(current_y)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font_id.clone()),
        size: Pt(32.0),
    });
    ops.push(Op::SetLineHeight { lh: Pt(32.0) });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text(DATABASE.name.to_string())],
    });
    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);

    current_y -= 2.0;

    // ---- Top horizontal rule ----
    // The old code did not explicitly set outline thickness here; printpdf
    // 0.7's default matched the PDF spec default of 1.0 pt, so we set the
    // same explicitly to preserve visual behavior across the upgrade.
    ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point::new(Mm(X_MARGIN), Mm(current_y)),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(current_y)),
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });

    current_y -= 4.0;
    let mut current_x = X_MARGIN;

    // ---- Phone ----
    if let Some(phone) = DATABASE.phone {
        let svg = printpdf::Svg::parse(PH_IMG, &mut warnings)
            .map_err(|e| format!("parse PH_IMG: {e}"))?;
        let svg_id = doc.add_xobject(&svg);
        ops.push(Op::UseXobject {
            id: svg_id.clone(),
            transform: XObjectTransform {
                translate_x: Some(Mm(current_x).into_pt()),
                translate_y: Some(Mm(current_y).into_pt()),
                scale_x: Some(0.04),
                scale_y: Some(0.04),
                dpi: Some(300.0),
                ..Default::default()
            },
        });
        let svg_width_pt = svg.width.map(|w| w.into_pt(300.0)).unwrap_or(Pt(0.0));
        let img_width = Mm::from(svg_width_pt).0;
        let img_width = (img_width * 0.04) + 0.5;
        current_x += img_width;

        let phone_width = w.word_width(phone, 10.0);
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(current_x), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(10.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(10.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(phone.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        current_x += phone_width + 1.5;
    }

    // ---- Email ----
    if let Some(email) = DATABASE.email {
        let svg = printpdf::Svg::parse(MAIL_IMG, &mut warnings)
            .map_err(|e| format!("parse MAIL_IMG: {e}"))?;
        let svg_id = doc.add_xobject(&svg);
        ops.push(Op::UseXobject {
            id: svg_id.clone(),
            transform: XObjectTransform {
                translate_x: Some(Mm(current_x).into_pt()),
                translate_y: Some(Mm(current_y).into_pt()),
                scale_x: Some(0.04),
                scale_y: Some(0.04),
                dpi: Some(300.0),
                ..Default::default()
            },
        });
        let svg_width_pt = svg.width.map(|w| w.into_pt(300.0)).unwrap_or(Pt(0.0));
        let img_width = Mm::from(svg_width_pt).0;
        let img_width = (img_width * 0.04) + 0.5;
        current_x += img_width;

        let email_width = w.word_width(email, 10.0);
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(current_x), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(10.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(10.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(email.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        current_x += email_width + 2.0;
    }

    // ---- GitHub ----
    if let Some(gh) = DATABASE.github {
        let svg = printpdf::Svg::parse(GITHUB_IMG, &mut warnings)
            .map_err(|e| format!("parse GITHUB_IMG: {e}"))?;
        let svg_id = doc.add_xobject(&svg);
        ops.push(Op::UseXobject {
            id: svg_id.clone(),
            transform: XObjectTransform {
                translate_x: Some(Mm(current_x).into_pt()),
                translate_y: Some(Mm(current_y).into_pt()),
                scale_x: Some(0.03),
                scale_y: Some(0.03),
                dpi: Some(300.0),
                ..Default::default()
            },
        });
        let svg_width_pt = svg.width.map(|w| w.into_pt(300.0)).unwrap_or(Pt(0.0));
        let gh_width = Mm::from(svg_width_pt).0;
        let gh_width = (gh_width * 0.03) + 0.5;
        current_x += gh_width;

        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(current_x), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(10.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(10.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(gh.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        current_x += w.word_width(gh, 10.0) + 1.0;
    }

    // ---- LinkedIn ----
    if let Some(li) = DATABASE.linkedin {
        let svg = printpdf::Svg::parse(LI_IMG, &mut warnings)
            .map_err(|e| format!("parse LI_IMG: {e}"))?;
        let svg_id = doc.add_xobject(&svg);
        ops.push(Op::UseXobject {
            id: svg_id.clone(),
            transform: XObjectTransform {
                translate_x: Some(Mm(current_x).into_pt()),
                translate_y: Some(Mm(current_y).into_pt()),
                scale_x: Some(0.065),
                scale_y: Some(0.065),
                dpi: Some(300.0),
                ..Default::default()
            },
        });
        let svg_width_pt = svg.width.map(|w| w.into_pt(300.0)).unwrap_or(Pt(0.0));
        let li_width = Mm::from(svg_width_pt).0;
        let li_width = (li_width * 0.065) + 0.5;
        current_x += li_width;

        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(current_x), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(10.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(10.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(li.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);
    }

    if DATABASE.phone.is_some()
        || DATABASE.email.is_some()
        || DATABASE.github.is_some()
        || DATABASE.linkedin.is_some()
    {
        current_y -= 2.0;
    }

    // ---- Bottom horizontal rule ----
    ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point::new(Mm(X_MARGIN), Mm(current_y)),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(current_y)),
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
    current_y -= 6.0;

    // ---- "Work" header ----
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(X_MARGIN), Mm(current_y)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font_id.clone()),
        size: Pt(18.0),
    });
    ops.push(Op::SetLineHeight { lh: Pt(18.0) });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text("Work".to_string())],
    });
    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);
    current_y -= 1.0;
    ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point::new(Mm(X_MARGIN), Mm(current_y)),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(current_y)),
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
    current_y -= 3.0;

    // ---- Jobs ----
    for job in DATABASE.jobs {
        current_y -= 3.0;

        // Job name (14 pt)
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(14.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(14.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(job.name.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        // Date range (11 pt, right-aligned)
        let date = format!("{} - {}", job.start, job.end.unwrap_or("Current"));
        let date_width = w.word_width(&date, 14.0);
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN - date_width), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(11.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(11.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(date)],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        // Underline below job name
        ops.push(Op::SetOutlineThickness { pt: Pt(0.25) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: point(X_MARGIN, current_y - 0.4),
                        bezier: false,
                    },
                    LinePoint {
                        p: point(LETTER_WIDTH_MM - X_MARGIN, current_y - 0.4),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });

        current_y -= 5.0;

        // Job title (12 pt)
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN + 2.0), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(12.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(12.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(job.title.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        current_y -= 4.0;

        for d in job.details {
            // Diamond bullet
            ops.push(Op::DrawPolygon {
                polygon: Polygon {
                    rings: vec![PolygonRing {
                        points: diamond_points(X_MARGIN + 3.0, current_y + 1.0),
                    }],
                    mode: PaintMode::Fill,
                    winding_order: WindingOrder::EvenOdd,
                },
            });

            let text = wrap_text(
                &w,
                d.snippet,
                INNER_X_MARGIN,
                LETTER_WIDTH_MM - (X_MARGIN * 2.2),
                10.0,
            );
            for line in text.lines() {
                ops.push(Op::SaveGraphicsState);
                ops.push(Op::StartTextSection);
                ops.push(Op::SetTextCursor {
                    pos: Point::new(Mm(INNER_X_MARGIN), Mm(current_y)),
                });
                ops.push(Op::SetFont {
                    font: PdfFontHandle::External(font_id.clone()),
                    size: Pt(10.0),
                });
                ops.push(Op::SetLineHeight { lh: Pt(10.0) });
                ops.push(Op::ShowText {
                    items: vec![TextItem::Text(line.to_string())],
                });
                ops.push(Op::EndTextSection);
                ops.push(Op::RestoreGraphicsState);

                current_y -= 4.0;
            }
        }
    }

    // ---- "Open Source" header ----
    current_y -= 3.0;
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(X_MARGIN), Mm(current_y)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font_id.clone()),
        size: Pt(18.0),
    });
    ops.push(Op::SetLineHeight { lh: Pt(18.0) });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text("Open Source".to_string())],
    });
    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);
    current_y -= 1.0;
    ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
    ops.push(Op::DrawLine {
        line: Line {
            points: vec![
                LinePoint {
                    p: Point::new(Mm(X_MARGIN), Mm(current_y)),
                    bezier: false,
                },
                LinePoint {
                    p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(current_y)),
                    bezier: false,
                },
            ],
            is_closed: false,
        },
    });
    current_y -= 5.0;

    for proj in DATABASE.open_source {
        // Project name (14 pt)
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(14.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(14.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(proj.name.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        // Underline below project name
        ops.push(Op::SetOutlineThickness { pt: Pt(0.25) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: point(X_MARGIN, current_y - 0.4),
                        bezier: false,
                    },
                    LinePoint {
                        p: point(LETTER_WIDTH_MM - X_MARGIN, current_y - 0.4),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });

        current_y -= 5.0;

        // Project short desc (12 pt)
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN + 2.0), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(12.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(12.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(proj.short_desc.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        current_y -= 3.0;

        for d in proj.sub_projects {
            ops.push(Op::DrawPolygon {
                polygon: Polygon {
                    rings: vec![PolygonRing {
                        points: diamond_points(X_MARGIN + 3.0, current_y + 1.0),
                    }],
                    mode: PaintMode::Fill,
                    winding_order: WindingOrder::EvenOdd,
                },
            });

            let text = wrap_text(
                &w,
                &format!("{}: {}", d.name, d.short_desc),
                INNER_X_MARGIN,
                LETTER_WIDTH_MM - (X_MARGIN * 2.2),
                10.0,
            );

            for line in text.lines() {
                ops.push(Op::SaveGraphicsState);
                ops.push(Op::StartTextSection);
                ops.push(Op::SetTextCursor {
                    pos: Point::new(Mm(INNER_X_MARGIN), Mm(current_y)),
                });
                ops.push(Op::SetFont {
                    font: PdfFontHandle::External(font_id.clone()),
                    size: Pt(10.0),
                });
                ops.push(Op::SetLineHeight { lh: Pt(10.0) });
                ops.push(Op::ShowText {
                    items: vec![TextItem::Text(line.to_string())],
                });
                ops.push(Op::EndTextSection);
                ops.push(Op::RestoreGraphicsState);

                current_y -= 4.0;
            }
        }
    }

    // ---- "Education" header ----
    current_y -= 3.0;
    ops.push(Op::SaveGraphicsState);
    ops.push(Op::StartTextSection);
    ops.push(Op::SetTextCursor {
        pos: Point::new(Mm(X_MARGIN), Mm(current_y)),
    });
    ops.push(Op::SetFont {
        font: PdfFontHandle::External(font_id.clone()),
        size: Pt(18.0),
    });
    ops.push(Op::SetLineHeight { lh: Pt(18.0) });
    ops.push(Op::ShowText {
        items: vec![TextItem::Text("Education".to_string())],
    });
    ops.push(Op::EndTextSection);
    ops.push(Op::RestoreGraphicsState);
    current_y -= 5.0;

    for school in DATABASE.education {
        // School name (14 pt)
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(14.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(14.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(school.name.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        current_y -= 5.0;

        ops.push(Op::DrawPolygon {
            polygon: Polygon {
                rings: vec![PolygonRing {
                    points: diamond_points(X_MARGIN + 1.0, current_y + 1.0),
                }],
                mode: PaintMode::Fill,
                winding_order: WindingOrder::EvenOdd,
            },
        });

        // School desc (12 pt)
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN + 3.5), Mm(current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(font_id.clone()),
            size: Pt(12.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(12.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text(school.desc.to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);

        current_y -= 5.0;
    }

    println!("writen to {}", out_path.as_ref().display());

    // Build the page and serialize. `save` returns bytes directly (no
    // Result wrapper in printpdf 0.12).
    let page = PdfPage::new(Mm(LETTER_WIDTH_MM), Mm(LETTER_HEIGHT_MM), ops);
    let bytes = doc
        .with_pages(vec![page])
        .save(&PdfSaveOptions::default(), &mut warnings);

    std::fs::write(out_path, bytes).unwrap();
    Ok(())
}

fn wrap_text(w: &wrapper::WithCalc, text: &str, x: f32, max_x: f32, font_size: f32) -> String {
    let space_width = w.word_width(" ", font_size);
    let mut ret_text = String::new();
    let mut current_x = x;
    for line in text.lines() {
        if line.trim().is_empty() {
            ret_text.push('\n');
            ret_text.push('\n');
            current_x = x;
            continue;
        }
        for word in line.split_whitespace() {
            let word_len = w.word_width(word, font_size);
            if current_x + word_len >= max_x {
                ret_text.push('\n');
                current_x = x;
            }
            current_x += word_len;
            ret_text.push_str(word);
            if current_x + space_width <= max_x {
                ret_text.push(' ');
            }
        }
    }
    ret_text
}

fn diamond_points(center_x: f32, center_y: f32) -> Vec<LinePoint> {
    let half_width = 0.5;
    let left_point = point(center_x - half_width, center_y);
    let top_point = point(center_x, center_y - half_width);
    let right_point = point(center_x + half_width, center_y);
    let bottom_point = point(center_x, center_y + half_width);
    vec![
        LinePoint {
            p: left_point,
            bezier: false,
        },
        LinePoint {
            p: top_point,
            bezier: false,
        },
        LinePoint {
            p: right_point,
            bezier: false,
        },
        LinePoint {
            p: bottom_point,
            bezier: false,
        },
        LinePoint {
            p: left_point,
            bezier: false,
        },
    ]
}

fn point(x: f32, y: f32) -> Point {
    Point::new(Mm(x), Mm(y))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_pdf_works() {
        generate_pdf(std::env::current_dir().unwrap().join("resume.pdf")).unwrap();
    }
}
