use std::path::Path;

use printpdf::{
    FontId, Line, LinePoint, Mm, Op, PaintMode, ParsedFont, PdfDocument, PdfFontHandle, PdfPage, PdfSaveOptions, Point, Polygon, PolygonRing, Pt, TextItem, WindingOrder, XObjectTransform,
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
    let g = PdfGenerator {
        current_x: X_MARGIN,
        current_y: LETTER_HEIGHT_MM - 20.0,
        font_id: FontId(String::new()),
    };
    let doc = g.generate()?;
    let bytes = doc
        .save(&PdfSaveOptions::default(), &mut Default::default());

    std::fs::write(out_path, bytes).unwrap();
    Ok(())
}

/// The pieces that make up the wrapped form of `text`: words borrowed from
/// `text`, interspersed with the separators that join them.
///
/// Nothing is allocated. Words are slices of `text` and every separator is a
/// `&'static str` literal, which coerces into the shorter `'a` so that both
/// travel on a single `Item` type. Concatenating the pieces in order produces
/// the wrapped text, where a separator is `" "` between two words on a line,
/// `"\n"` where a line wrapped, and `"\n\n"` for a blank line in the source.
fn wrap_text_pieces<'a>(
    w: &'a wrapper::WithCalc,
    text: &'a str,
    x: f32,
    max_x: f32,
    font_size: f32,
) -> impl Iterator<Item = &'a str> + 'a {
    let space_width = w.word_width(" ", font_size);
    let mut lines = text.lines();
    let mut words: Option<std::str::SplitWhitespace<'a>> = None;
    let mut current_x = x;
    // A word contributes at most three pieces: the wrap that precedes it, the
    // word itself, then the space that follows it.
    let mut buf: [&'a str; 3] = [""; 3];
    let (mut len, mut pos) = (0usize, 0usize);

    std::iter::from_fn(move || loop {
        if pos < len {
            let piece = buf[pos];
            pos += 1;
            return Some(piece);
        }
        let Some(words_iter) = words.as_mut() else {
            let line = lines.next()?;
            if line.trim().is_empty() {
                current_x = x;
                return Some("\n\n");
            }
            words = Some(line.split_whitespace());
            continue;
        };
        let Some(word) = words_iter.next() else {
            words = None;
            continue;
        };
        let word_len = w.word_width(word, font_size);
        len = 0;
        if current_x + word_len >= max_x {
            current_x = x;
            buf[len] = "\n";
            len += 1;
        }
        current_x += word_len;
        buf[len] = word;
        len += 1;
        if current_x + space_width <= max_x {
            buf[len] = " ";
            len += 1;
            current_x += space_width;
        }
        pos = 0;
    })
}

/// Groups [`wrap_text_pieces`] into the rendered lines of `text`.
///
/// The one `String` per line is what `Op::ShowText` needs anyway; what this
/// avoids is the second `String` holding the whole wrapped block, and the
/// re-scan that splitting it back apart required.
fn wrap_text<'a>(
    w: &'a wrapper::WithCalc,
    text: &'a str,
    x: f32,
    max_x: f32,
    font_size: f32,
) -> impl Iterator<Item = String> + 'a {
    let mut pieces = wrap_text_pieces(w, text, x, max_x, font_size);
    let mut blank_pending = false;
    let mut done = false;
    std::iter::from_fn(move || {
        if blank_pending {
            blank_pending = false;
            return Some(String::new());
        }
        if done {
            return None;
        }
        let mut line = String::new();
        loop {
            // Words come from `split_whitespace`, so no word can be mistaken
            // for one of the separators matched here.
            match pieces.next() {
                None => {
                    done = true;
                    return (!line.is_empty()).then_some(line);
                }
                Some("\n") => return Some(line),
                Some("\n\n") => {
                    blank_pending = true;
                    return Some(line);
                }
                Some(piece) => line.push_str(piece),
            }
        }
    })
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

struct PdfGenerator {
    current_y: f32,
    current_x: f32,
    font_id: FontId,
}

impl PdfGenerator {
    fn generate(mut self) -> Result<PdfDocument, String> {
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
        self.font_id = doc.add_font(&parsed_font);

        // The wrapper still owns a ttf-parser face for measuring text widths
        // (used by `wrap_text` below). printpdf's own text shaper handles
        // actual rendering; we only need widths here.
        let face = wrapper::TtfFace::from_vec(RESUME_FONT.to_vec())
            .map_err(|e| format!("Error parsing font face: {e}"))?;
        let w = WithCalc::new(face);

        let mut ops: Vec<Op> = self.generate_name().collect();

        // ---- Top horizontal rule ----
        // The old code did not explicitly set outline thickness here; printpdf
        // 0.7's default matched the PDF spec default of 1.0 pt, so we set the
        // same explicitly to preserve visual behavior across the upgrade.
        ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                    LinePoint {
                        p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });

        self.current_y -= 4.0;

        // ---- Phone ----
        if let Some(phone) = DATABASE.phone {
            let svg = printpdf::Svg::parse(PH_IMG, &mut warnings)
                .map_err(|e| format!("parse PH_IMG: {e}"))?;
            let svg_id = doc.add_xobject(&svg);
            ops.push(Op::UseXobject {
                id: svg_id.clone(),
                transform: XObjectTransform {
                    translate_x: Some(Mm(self.current_x).into_pt()),
                    translate_y: Some(Mm(self.current_y).into_pt()),
                    scale_x: Some(0.04),
                    scale_y: Some(0.04),
                    dpi: Some(300.0),
                    ..Default::default()
                },
            });
            let svg_width_pt = svg.width.map(|w| w.into_pt(300.0)).unwrap_or(Pt(0.0));
            let img_width = Mm::from(svg_width_pt).0;
            let img_width = (img_width * 0.04) + 0.5;
            self.current_x += img_width;

            let phone_width = w.word_width(phone, 10.0);
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(self.current_x), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
                size: Pt(10.0),
            });
            ops.push(Op::SetLineHeight { lh: Pt(10.0) });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(phone.to_string())],
            });
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);

            self.current_x += phone_width + 1.5;
        }

        // ---- Email ----
        if let Some(email) = DATABASE.email {
            let svg = printpdf::Svg::parse(MAIL_IMG, &mut warnings)
                .map_err(|e| format!("parse MAIL_IMG: {e}"))?;
            let svg_id = doc.add_xobject(&svg);
            ops.push(Op::UseXobject {
                id: svg_id.clone(),
                transform: XObjectTransform {
                    translate_x: Some(Mm(self.current_x).into_pt()),
                    translate_y: Some(Mm(self.current_y).into_pt()),
                    scale_x: Some(0.04),
                    scale_y: Some(0.04),
                    dpi: Some(300.0),
                    ..Default::default()
                },
            });
            let svg_width_pt = svg.width.map(|w| w.into_pt(300.0)).unwrap_or(Pt(0.0));
            let img_width = Mm::from(svg_width_pt).0;
            let img_width = (img_width * 0.04) + 0.5;
            self.current_x += img_width;

            let email_width = w.word_width(email, 10.0);
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(self.current_x), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
                size: Pt(10.0),
            });
            ops.push(Op::SetLineHeight { lh: Pt(10.0) });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(email.to_string())],
            });
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);

            self.current_x += email_width + 2.0;
        }

        // ---- GitHub ----
        if let Some(gh) = DATABASE.github {
            let svg = printpdf::Svg::parse(GITHUB_IMG, &mut warnings)
                .map_err(|e| format!("parse GITHUB_IMG: {e}"))?;
            let svg_id = doc.add_xobject(&svg);
            ops.push(Op::UseXobject {
                id: svg_id.clone(),
                transform: XObjectTransform {
                    translate_x: Some(Mm(self.current_x).into_pt()),
                    translate_y: Some(Mm(self.current_y).into_pt()),
                    scale_x: Some(0.03),
                    scale_y: Some(0.03),
                    dpi: Some(300.0),
                    ..Default::default()
                },
            });
            let svg_width_pt = svg.width.map(|w| w.into_pt(300.0)).unwrap_or(Pt(0.0));
            let gh_width = Mm::from(svg_width_pt).0;
            let gh_width = (gh_width * 0.03) + 0.5;
            self.current_x += gh_width;

            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(self.current_x), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
                size: Pt(10.0),
            });
            ops.push(Op::SetLineHeight { lh: Pt(10.0) });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(gh.to_string())],
            });
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);

            self.current_x += w.word_width(gh, 10.0) + 1.0;
        }

        // ---- LinkedIn ----
        if let Some(li) = DATABASE.linkedin {
            let svg = printpdf::Svg::parse(LI_IMG, &mut warnings)
                .map_err(|e| format!("parse LI_IMG: {e}"))?;
            let svg_id = doc.add_xobject(&svg);
            ops.push(Op::UseXobject {
                id: svg_id.clone(),
                transform: XObjectTransform {
                    translate_x: Some(Mm(self.current_x).into_pt()),
                    translate_y: Some(Mm(self.current_y).into_pt()),
                    scale_x: Some(0.065),
                    scale_y: Some(0.065),
                    dpi: Some(300.0),
                    ..Default::default()
                },
            });
            let svg_width_pt = svg.width.map(|w| w.into_pt(300.0)).unwrap_or(Pt(0.0));
            let li_width = Mm::from(svg_width_pt).0;
            let li_width = (li_width * 0.065) + 0.5;
            self.current_x += li_width;

            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(self.current_x), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
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
            self.current_y -= 2.0;
        }

        // ---- Bottom horizontal rule ----
        ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                    LinePoint {
                        p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
        self.current_y -= 6.0;

        // ---- "Work" header ----
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(self.font_id.clone()),
            size: Pt(18.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(18.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text("Work".to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);
        self.current_y -= 1.0;
        ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                    LinePoint {
                        p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
        self.current_y -= 3.0;

        // ---- Jobs ----
        for job in DATABASE.jobs {
            self.current_y -= 3.0;

            // Job name (14 pt)
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
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
            let date_width = w.word_width(&date, 11.0);
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(
                    Mm(LETTER_WIDTH_MM - X_MARGIN - date_width),
                    Mm(self.current_y),
                ),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
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
                            p: point(X_MARGIN, self.current_y - 0.4),
                            bezier: false,
                        },
                        LinePoint {
                            p: point(LETTER_WIDTH_MM - X_MARGIN, self.current_y - 0.4),
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                },
            });

            self.current_y -= 5.0;

            // Job title (12 pt)
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(X_MARGIN + 2.0), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
                size: Pt(12.0),
            });
            ops.push(Op::SetLineHeight { lh: Pt(12.0) });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(job.title.to_string())],
            });
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);

            self.current_y -= 4.0;

            for d in job.details {
                // Diamond bullet
                ops.push(Op::DrawPolygon {
                    polygon: Polygon {
                        rings: vec![PolygonRing {
                            points: diamond_points(X_MARGIN + 3.0, self.current_y + 1.0),
                        }],
                        mode: PaintMode::Fill,
                        winding_order: WindingOrder::EvenOdd,
                    },
                });

                for line in wrap_text(
                    &w,
                    d.snippet,
                    INNER_X_MARGIN,
                    LETTER_WIDTH_MM - X_MARGIN,
                    10.0,
                ) {
                    ops.push(Op::SaveGraphicsState);
                    ops.push(Op::StartTextSection);
                    ops.push(Op::SetTextCursor {
                        pos: Point::new(Mm(INNER_X_MARGIN), Mm(self.current_y)),
                    });
                    ops.push(Op::SetFont {
                        font: PdfFontHandle::External(self.font_id.clone()),
                        size: Pt(10.0),
                    });
                    ops.push(Op::SetLineHeight { lh: Pt(10.0) });
                    ops.push(Op::ShowText {
                        items: vec![TextItem::Text(line)],
                    });
                    ops.push(Op::EndTextSection);
                    ops.push(Op::RestoreGraphicsState);

                    self.current_y -= 4.0;
                }
            }
        }

        // ---- "Open Source" header ----
        self.current_y -= 3.0;
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(self.font_id.clone()),
            size: Pt(18.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(18.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text("Open Source".to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);
        self.current_y -= 1.0;
        ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                    LinePoint {
                        p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
        self.current_y -= 5.0;

        for proj in DATABASE.open_source {
            // Project name (14 pt)
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
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
                            p: point(X_MARGIN, self.current_y - 0.4),
                            bezier: false,
                        },
                        LinePoint {
                            p: point(LETTER_WIDTH_MM - X_MARGIN, self.current_y - 0.4),
                            bezier: false,
                        },
                    ],
                    is_closed: false,
                },
            });
            // let desc_width = doc.wi
            let desc_width = w.word_width(&proj.short_desc, 11.0);

            // Project short desc (12 pt)
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(
                    Mm(LETTER_WIDTH_MM - X_MARGIN - desc_width),
                    Mm(self.current_y),
                ),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
                size: Pt(11.0),
            });
            ops.push(Op::SetLineHeight { lh: Pt(11.0) });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(proj.short_desc.to_string())],
            });
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);

            self.current_y -= 4.0;

            for d in proj.sub_projects {
                ops.push(Op::DrawPolygon {
                    polygon: Polygon {
                        rings: vec![PolygonRing {
                            points: diamond_points(X_MARGIN + 3.0, self.current_y + 1.0),
                        }],
                        mode: PaintMode::Fill,
                        winding_order: WindingOrder::EvenOdd,
                    },
                });

                for line in wrap_text(
                    &w,
                    &format!("{}: {}", d.name, d.short_desc),
                    INNER_X_MARGIN,
                    LETTER_WIDTH_MM - (X_MARGIN * 2.2),
                    10.0,
                ) {
                    ops.push(Op::SaveGraphicsState);
                    ops.push(Op::StartTextSection);
                    ops.push(Op::SetTextCursor {
                        pos: Point::new(Mm(INNER_X_MARGIN), Mm(self.current_y)),
                    });
                    ops.push(Op::SetFont {
                        font: PdfFontHandle::External(self.font_id.clone()),
                        size: Pt(10.0),
                    });
                    ops.push(Op::SetLineHeight { lh: Pt(10.0) });
                    ops.push(Op::ShowText {
                        items: vec![TextItem::Text(line)],
                    });
                    ops.push(Op::EndTextSection);
                    ops.push(Op::RestoreGraphicsState);

                    self.current_y -= 4.0;
                }
            }
            if proj.sub_projects.is_empty() {
                ops.push(Op::DrawPolygon {
                    polygon: Polygon {
                        rings: vec![PolygonRing {
                            points: diamond_points(X_MARGIN + 3.0, self.current_y + 1.0),
                        }],
                        mode: PaintMode::Fill,
                        winding_order: WindingOrder::EvenOdd,
                    },
                });

                for line in wrap_text(
                    &w,
                    proj.long_desc,
                    INNER_X_MARGIN,
                    LETTER_WIDTH_MM - (X_MARGIN * 2.2),
                    10.0,
                ) {
                    ops.push(Op::SaveGraphicsState);
                    ops.push(Op::StartTextSection);
                    ops.push(Op::SetTextCursor {
                        pos: Point::new(Mm(INNER_X_MARGIN), Mm(self.current_y)),
                    });
                    ops.push(Op::SetFont {
                        font: PdfFontHandle::External(self.font_id.clone()),
                        size: Pt(10.0),
                    });
                    ops.push(Op::SetLineHeight { lh: Pt(10.0) });
                    ops.push(Op::ShowText {
                        items: vec![TextItem::Text(line)],
                    });
                    ops.push(Op::EndTextSection);
                    ops.push(Op::RestoreGraphicsState);

                    self.current_y -= 4.0;
                }
            }
        }

        // ---- "Education" header ----
        ops.push(Op::SaveGraphicsState);
        ops.push(Op::StartTextSection);
        ops.push(Op::SetTextCursor {
            pos: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
        });
        ops.push(Op::SetFont {
            font: PdfFontHandle::External(self.font_id.clone()),
            size: Pt(18.0),
        });
        ops.push(Op::SetLineHeight { lh: Pt(18.0) });
        ops.push(Op::ShowText {
            items: vec![TextItem::Text("Education".to_string())],
        });
        ops.push(Op::EndTextSection);
        ops.push(Op::RestoreGraphicsState);
        self.current_y -= 1.0;
        ops.push(Op::SetOutlineThickness { pt: Pt(1.0) });
        ops.push(Op::DrawLine {
            line: Line {
                points: vec![
                    LinePoint {
                        p: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                    LinePoint {
                        p: Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(self.current_y)),
                        bezier: false,
                    },
                ],
                is_closed: false,
            },
        });
        self.current_y -= 5.0;

        for school in DATABASE.education {
            // School name (14 pt)
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
                size: Pt(14.0),
            });
            ops.push(Op::SetLineHeight { lh: Pt(14.0) });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(school.name.to_string())],
            });
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);

            self.current_y -= 5.0;

            ops.push(Op::DrawPolygon {
                polygon: Polygon {
                    rings: vec![PolygonRing {
                        points: diamond_points(X_MARGIN + 1.0, self.current_y + 1.0),
                    }],
                    mode: PaintMode::Fill,
                    winding_order: WindingOrder::EvenOdd,
                },
            });

            // School desc (12 pt)
            ops.push(Op::SaveGraphicsState);
            ops.push(Op::StartTextSection);
            ops.push(Op::SetTextCursor {
                pos: Point::new(Mm(X_MARGIN + 3.5), Mm(self.current_y)),
            });
            ops.push(Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
                size: Pt(12.0),
            });
            ops.push(Op::SetLineHeight { lh: Pt(12.0) });
            ops.push(Op::ShowText {
                items: vec![TextItem::Text(school.desc.to_string())],
            });
            ops.push(Op::EndTextSection);
            ops.push(Op::RestoreGraphicsState);

            self.current_y -= 5.0;
        }

        // Build the page and serialize. `save` returns bytes directly (no
        // Result wrapper in printpdf 0.12).
        let page = PdfPage::new(Mm(LETTER_WIDTH_MM), Mm(LETTER_HEIGHT_MM), ops);
        doc.with_pages(vec![page]);
        Ok(doc)
    }

    fn generate_name(&mut self) -> impl Iterator<Item = Op> {
        let events = [
            Op::SaveGraphicsState,
            Op::StartTextSection,
            Op::SetTextCursor {
                pos: Point::new(Mm(X_MARGIN), Mm(self.current_y)),
            },
            Op::SetFont {
                font: PdfFontHandle::External(self.font_id.clone()),
                size: Pt(32.0),
            },
            Op::SetLineHeight { lh: Pt(32.0) },
            Op::ShowText {
                items: vec![TextItem::Text(DATABASE.name.to_string())],
            },
        Op::EndTextSection,
        Op::RestoreGraphicsState];

        self.current_y -= 2.0;
        events.into_iter()
    }
}
