use std::path::Path;

use printpdf::{Font, IndirectFontRef, Line, Mm, PdfLayerReference, Point, Polygon};
use resume_tui::DATABASE;
use wrapper::WithCalc;

static RESUME_FONT: &[u8] = include_bytes!("../../assets/BhuTukaExpandedOne-Regular.ttf");

const LETTER_HEIGHT_MM: f32 = 279.0;
pub(crate) const LETTER_WIDTH_MM: f32 = 216.0;
const X_MARGIN: f32 = 10.0;
const INNER_X_MARGIN: f32 = 15.0;

mod wrapper;

pub fn generate_pdf(out_path: impl AsRef<Path>) -> Result<(), String> {
    let (mut doc, page1, layer1) = printpdf::PdfDocument::new("resume", Mm(LETTER_WIDTH_MM), Mm(LETTER_HEIGHT_MM), "text");
    doc = doc.with_author(DATABASE.name)
        .with_title("resume");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_external_font(RESUME_FONT).unwrap();
    let face = wrapper::TtfFace::from_vec(RESUME_FONT.to_vec()).unwrap();
    // let font_bold = doc.add_builtin_font(printpdf::BuiltinFont::CourierBold).unwrap();
    let mut current_y = LETTER_HEIGHT_MM-20.0;
    current_layer.use_text(DATABASE.name, 48.0, Mm(X_MARGIN), Mm(current_y), &font);
    current_y -= 10.0;
    if let Some(gh) = DATABASE.github {
        current_layer.use_text(format!("https://www.github.com/{gh}"), 12.0, Mm(X_MARGIN), Mm(current_y), &font);
        current_y -= 5.0;
    }
    if let Some(li) = DATABASE.linkedin {
        current_layer.use_text(format!("https://www.linkedin.com/in/{li}"), 12.0, Mm(X_MARGIN), Mm(current_y), &font);
        current_y -= 5.0;
    }
    let line = printpdf::Line {
        points: vec![
            (Point::new(Mm(X_MARGIN), Mm(current_y)), false),
            (Point::new(Mm(LETTER_WIDTH_MM-X_MARGIN), Mm(current_y)), false),
        ],
        is_closed: false,
    };
    current_layer.add_line(line);
    current_y -= 8.0;
    
    current_layer.use_text("Work", 24.0, Mm(X_MARGIN), Mm(current_y), &font);
    current_y -= 7.0;
    let w = WithCalc::new(face);
    for job in DATABASE.jobs {
        current_layer.use_text(job.name, 18.0, Mm(X_MARGIN), Mm(current_y), &font);
        let date = format!("{} - {}", job.start, job.end.unwrap_or("Current"));
        let date_width = w.word_width(&date, 12.0);
        current_layer.use_text(&date, 11.0, Mm(LETTER_WIDTH_MM - X_MARGIN - date_width), Mm(current_y), &font);
        current_layer.set_outline_thickness(0.25);
        current_layer.add_line(Line {
                points: vec![
                    (point(X_MARGIN, current_y-0.4), false),
                    (point(LETTER_WIDTH_MM - X_MARGIN, current_y-0.4), false),
                ],
                is_closed: false
            });
        current_y -= 7.0;
        current_layer.use_text(job.title, 14.0, Mm(X_MARGIN + 2.0), Mm(current_y), &font);
        current_y -= 6.0;
        for d in job.details {
            current_layer.add_polygon(Polygon {
                rings: vec![diamond_points(X_MARGIN + 3.0, current_y + 1.0)],
                mode: printpdf::path::PaintMode::Fill,
                winding_order: printpdf::path::WindingOrder::EvenOdd,
            });
            current_layer.use_text(d.headline, 12.0, Mm(INNER_X_MARGIN), Mm(current_y), &font);
            current_y -= 5.0;
            let text = wrap_text(&w, d.detail, INNER_X_MARGIN, LETTER_WIDTH_MM - (X_MARGIN * 2.2), 10.0);
            
            for line in text.lines() {
                current_layer.use_text(line, 10.0, Mm(INNER_X_MARGIN), Mm(current_y), &font);
                current_y -= 4.0;
            }
            current_y -= 4.0;
        }
    }
    
    std::fs::write(out_path, doc.save_to_bytes().unwrap()).unwrap();
    Ok(())
}

fn wrap_text(
    w: &wrapper::WithCalc,
    text: &str,
    x: f32,
    max_x: f32,
    font_size: f32,
) -> String {
    let space_width = w.word_width(" ", font_size);
    let mut ret_text = String::new();
    let mut current_x = x;
    for line in text.lines() {
        if line.trim().is_empty() {
            ret_text.push('\n');
            ret_text.push('\n');
            let mut current_x = x;
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
            // layer.use_text(word, font_size, Mm(current_x), Mm(current_y), font);
            if current_x + space_width <= max_x {
                ret_text.push(' ');
            }
        }
    }
    ret_text
}

fn diamond_points(center_x: f32, center_y: f32) -> Vec<(Point, bool)> {
    let half_width = 0.5;
    let left_point = point(center_x - half_width, center_y);
    let top_point = point(center_x, center_y - half_width);
    let right_point = point(center_x + half_width, center_y);
    let bottom_point = point(center_x, center_y + half_width);
    let mut ret = Vec::new();
    ret.push((left_point, false));
    ret.push((top_point, false));
    ret.push((right_point, false));
    ret.push((bottom_point, false));
    ret.push((left_point, false));
    ret
}

fn point(x: f32, y: f32) -> Point {
    Point::new(Mm(x), Mm(y))
}


fn gen_md() -> String {
    let mut ret = String::new();
    ret.push_str(r#"---

# Front Matter (YAML)

language: "en-US"

---
# "#);
    ret.push_str(resume_tui::DATABASE.name);
    ret.push('\n');
    ret.push('\n');
    ret.push_str(r#"<div style="display: flex; flex-flow: column; align-content: space-between;">"#);
    if let Some(gh) = DATABASE.github {
        ret.push_str("<span>https://www.github.com/");
        ret.push_str(gh);
        ret.push_str("</span>");
    }
    if let Some(li) = DATABASE.linkedin {
        ret.push_str("<span>https://www.linkedin.com/in/");
        ret.push_str(li);
        ret.push_str("</span>");
    }
    ret.push_str("</div>\n\n");
    ret.push_str("---\n");
    ret.push_str("## Work\n");
    for job in DATABASE.jobs {
        ret.push_str("### ");
        ret.push_str(job.name);
        ret.push('\n');
        ret.push_str("__");
        ret.push_str(&job.start);
        ret.push_str(" - ");
        ret.push_str(job.end.unwrap_or("Current"));
        ret.push_str("__\n");
        for d in job.details {
            ret.push_str("- __");
            ret.push_str(d.headline);
            ret.push_str("__\n\n");
            for line in d.detail.lines() {
                ret.push_str("    ");
                ret.push_str(line);
                ret.push('\n');
            }
        }
    }
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_pdf_works() {
        generate_pdf(std::env::current_dir().unwrap().join("resume.pdf")).unwrap();
    }
}
