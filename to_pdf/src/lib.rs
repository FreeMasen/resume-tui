use std::{io::Cursor, path::Path};

use printpdf::{
    Font, IndirectFontRef, Line, Mm, PdfLayerReference, Point, Polygon, Pt, Px, SvgTransform,
};
use resume_tui::DATABASE;
use wrapper::WithCalc;

static RESUME_FONT: &[u8] = include_bytes!("../../assets/BhuTukaExpandedOne-Regular.ttf");
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
    let (mut doc, page1, layer1) =
        printpdf::PdfDocument::new("resume", Mm(LETTER_WIDTH_MM), Mm(LETTER_HEIGHT_MM), "text");
    doc = doc.with_author(DATABASE.name).with_title("resume");
    let current_layer = doc.get_page(page1).get_layer(layer1);
    let font = doc.add_external_font(RESUME_FONT).unwrap();
    let face = wrapper::TtfFace::from_vec(RESUME_FONT.to_vec()).unwrap();
    // let font_bold = doc.add_builtin_font(printpdf::BuiltinFont::CourierBold).unwrap();
    let mut current_y = LETTER_HEIGHT_MM - 20.0;
    current_layer.use_text(DATABASE.name, 48.0, Mm(X_MARGIN), Mm(current_y), &font);
    current_y -= 2.0;
    let w = WithCalc::new(face);
    let line = printpdf::Line {
        points: vec![
            (Point::new(Mm(X_MARGIN), Mm(current_y)), false),
            (
                Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(current_y)),
                false,
            ),
        ],
        is_closed: false,
    };
    current_layer.add_line(line);
    current_y -= 3.0;
    let mut current_x = X_MARGIN;
    if let Some(phone) = DATABASE.phone {
        let svg = printpdf::Svg::parse(PH_IMG).unwrap();
        let svg_ref = svg.into_xobject(&current_layer);
        svg_ref.clone().add_to_layer(
            &current_layer,
            SvgTransform {
                translate_x: Some(Mm(current_x).into_pt()),
                translate_y: Some(Mm(current_y - 0.5).into_pt()),
                scale_x: Some(0.04),
                scale_y: Some(0.04),
                ..Default::default()
            },
        );
        let img_width = Mm::from(svg_ref.width.into_pt(300.0)).0;
        let img_width = (img_width * 0.04) + 0.5;
        current_x += img_width;
        let phone_width = w.word_width(&phone, 10.0);
        current_layer.use_text(phone, 10.0, Mm(current_x), Mm(current_y), &font);
        current_x += phone_width + 1.5;
    }
    if let Some(email) = DATABASE.email {
        let svg = printpdf::Svg::parse(MAIL_IMG).unwrap();
        let svg_ref = svg.into_xobject(&current_layer);
        svg_ref.clone().add_to_layer(
            &current_layer,
            SvgTransform {
                translate_x: Some(Mm(current_x).into_pt()),
                translate_y: Some(Mm(current_y - 0.5).into_pt()),
                scale_x: Some(0.04),
                scale_y: Some(0.04),
                ..Default::default()
            },
        );
        let img_width = Mm::from(svg_ref.width.into_pt(300.0)).0;
        let img_width = (img_width * 0.04) + 0.5;
        current_x += img_width;
        let email_width = w.word_width(&email, 10.0);
        current_layer.use_text(email, 10.0, Mm(current_x), Mm(current_y), &font);
        current_x += email_width + 2.0;
    }
    if let Some(gh) = DATABASE.github {
        let svg = printpdf::Svg::parse(GITHUB_IMG).unwrap();
        let svg_ref = svg.into_xobject(&current_layer);
        svg_ref.clone().add_to_layer(
            &current_layer,
            SvgTransform {
                translate_x: Some(Mm(current_x).into_pt()),
                translate_y: Some(Mm(current_y - 0.5).into_pt()),
                scale_x: Some(0.03),
                scale_y: Some(0.03),
                ..Default::default()
            },
        );
        let gh_width = Mm::from(svg_ref.width.into_pt(300.0)).0;
        let gh_width = (gh_width * 0.03) + 0.5;
        current_x += gh_width;
        current_layer.use_text(gh, 10.0, Mm(current_x), Mm(current_y), &font);
        current_x += w.word_width(gh, 10.0) + 1.0;
    }
    if let Some(li) = DATABASE.linkedin {
        let svg = printpdf::Svg::parse(LI_IMG).unwrap();
        let svg_ref = svg.into_xobject(&current_layer);
        svg_ref.clone().add_to_layer(
            &current_layer,
            SvgTransform {
                translate_x: Some(Mm(current_x).into_pt()),
                translate_y: Some(Mm(current_y - 0.4).into_pt()),
                scale_x: Some(0.065),
                scale_y: Some(0.065),
                ..Default::default()
            },
        );
        let li_width = Mm::from(svg_ref.width.into_pt(300.0)).0;
        let li_width = (li_width * 0.065) + 0.5;
        current_x += li_width;
        current_layer.use_text(
            li,
            10.0,
            Mm(current_x),
            Mm(current_y),
            &font,
        );
    }
    if DATABASE.phone.is_some()
        || DATABASE.email.is_some()
        || DATABASE.github.is_some()
        || DATABASE.linkedin.is_some()
    {
        current_y -= 2.0;
    }
    let line = printpdf::Line {
        points: vec![
            (Point::new(Mm(X_MARGIN), Mm(current_y)), false),
            (
                Point::new(Mm(LETTER_WIDTH_MM - X_MARGIN), Mm(current_y)),
                false,
            ),
        ],
        is_closed: false,
    };
    current_layer.add_line(line);
    current_y -= 6.0;
    
    current_layer.use_text("Work", 18.0, Mm(X_MARGIN), Mm(current_y), &font);
    current_y -= 2.0;
    
    for job in DATABASE.jobs {
        current_y -= 3.0;
        current_layer.use_text(job.name, 14.0, Mm(X_MARGIN), Mm(current_y), &font);
        let date = format!("{} - {}", job.start, job.end.unwrap_or("Current"));
        let date_width = w.word_width(&date, 12.0);
        current_layer.use_text(
            &date,
            11.0,
            Mm(LETTER_WIDTH_MM - X_MARGIN - date_width),
            Mm(current_y),
            &font,
        );
        current_layer.set_outline_thickness(0.25);
        current_layer.add_line(Line {
            points: vec![
                (point(X_MARGIN, current_y - 0.4), false),
                (point(LETTER_WIDTH_MM - X_MARGIN, current_y - 0.4), false),
            ],
            is_closed: false,
        });
        current_y -= 5.0;
        current_layer.use_text(job.title, 12.0, Mm(X_MARGIN + 2.0), Mm(current_y), &font);
        current_y -= 3.0;
        for d in job.details {
            current_layer.add_polygon(Polygon {
                rings: vec![diamond_points(X_MARGIN + 3.0, current_y + 1.0)],
                mode: printpdf::path::PaintMode::Fill,
                winding_order: printpdf::path::WindingOrder::EvenOdd,
            });
            current_layer.use_text(d.headline, 10.0, Mm(INNER_X_MARGIN), Mm(current_y), &font);
            current_y -= 4.0;
            let text = wrap_text(
                &w,
                d.snippet,
                INNER_X_MARGIN,
                LETTER_WIDTH_MM - (X_MARGIN * 2.2),
                10.0,
            );

            for line in text.lines() {
                current_layer.use_text(line, 10.0, Mm(INNER_X_MARGIN), Mm(current_y), &font);
                current_y -= 4.0;
            }
        }
    }

    current_y -= 3.0;
    current_layer.use_text("Open Source", 18.0, Mm(X_MARGIN), Mm(current_y), &font);
    current_y -= 5.0;

    for proj in DATABASE.open_source {
        current_layer.use_text(proj.name, 14.0, Mm(X_MARGIN), Mm(current_y), &font);
        current_layer.set_outline_thickness(0.25);
        current_layer.add_line(Line {
            points: vec![
                (point(X_MARGIN, current_y - 0.4), false),
                (point(LETTER_WIDTH_MM - X_MARGIN, current_y - 0.4), false),
            ],
            is_closed: false,
        });
        current_y -= 5.0;
        current_layer.use_text(proj.short_desc, 12.0, Mm(X_MARGIN + 2.0), Mm(current_y), &font);
        current_y -= 3.0;
        for d in proj.sub_projects {
            current_layer.add_polygon(Polygon {
                rings: vec![diamond_points(X_MARGIN + 3.0, current_y + 1.0)],
                mode: printpdf::path::PaintMode::Fill,
                winding_order: printpdf::path::WindingOrder::EvenOdd,
            });
            current_layer.use_text(d.name, 10.0, Mm(INNER_X_MARGIN), Mm(current_y), &font);
            current_y -= 4.0;
            let text = wrap_text(
                &w,
                d.short_desc,
                INNER_X_MARGIN,
                LETTER_WIDTH_MM - (X_MARGIN * 2.2),
                10.0,
            );

            for line in text.lines() {
                current_layer.use_text(line, 10.0, Mm(INNER_X_MARGIN), Mm(current_y), &font);
                current_y -= 4.0;
            }
            // current_y -= 4.0;
        }
    }

    current_y -= 3.0;
    current_layer.use_text("Education", 18.0, Mm(X_MARGIN), Mm(current_y), &font);
    current_y -= 5.0;

    for school in DATABASE.education {
        current_layer.use_text(school.name, 14.0, Mm(X_MARGIN), Mm(current_y), &font);
        current_y -= 5.0;
        current_layer.add_polygon(Polygon {
            rings: vec![diamond_points(X_MARGIN + 1.0, current_y + 1.0)],
            mode: printpdf::path::PaintMode::Fill,
            winding_order: printpdf::path::WindingOrder::EvenOdd,
        });
        current_layer.use_text(school.desc, 12.0, Mm(X_MARGIN + 3.5), Mm(current_y), &font);
        current_y -= 5.0;
    }
    println!("writen to {}", out_path.as_ref().display());
    std::fs::write(out_path, doc.save_to_bytes().unwrap()).unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gen_pdf_works() {
        generate_pdf(std::env::current_dir().unwrap().join("resume.pdf")).unwrap();
    }
}
