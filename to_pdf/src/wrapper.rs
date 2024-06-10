use std::collections::HashMap;

use owned_ttf_parser::{AsFaceRef, Face, OwnedFace};
use printpdf::{FontData, FontMetrics, GlyphMetrics};

const MM_IN_PT: f32 = 0.3527777778;

pub struct WithCalc {
    font: TtfFace,
}

impl WithCalc {
    pub fn new(font: TtfFace) -> Self {
        Self { font }
    }

    pub fn word_width(&self, word: &str, pts_height: f32) -> f32 {
        word.chars().map(|c| self.char_width(c, pts_height)).sum()
    }

    fn char_width(&self, c: char, pts_height: f32) -> f32 {
        let id = self
            .font
            .glyph_id(c)
            .unwrap_or_else(|| panic!("Unknown glyph: {c}"));
        let w = self.font.glyph_metrics(id).unwrap_or_else(|| {
            panic!("Unexpected glyph! {c} {id}");
        });
        let units_per_em = self.font.face().as_face_ref().units_per_em() as f32;
        let ratio = pts_height / units_per_em;
        (ratio * w.width as f32) * MM_IN_PT
    }
}

/// Wrapper struct for `owned_ttf_parser::OwnedFace` that implements `Clone` and that makes sure
/// that the font is scalable.
#[derive(Clone, Debug)]
pub struct TtfFace {
    inner: std::sync::Arc<OwnedFace>,
    units_per_em: u16,
}

impl TtfFace {
    pub fn from_vec(v: Vec<u8>) -> Result<Self, String> {
        let face =
            OwnedFace::from_vec(v, 0).map_err(|e| format!("Error parsing font face: {e}"))?;
        let units_per_em = face.as_face_ref().units_per_em();
        Ok(Self {
            inner: std::sync::Arc::new(face),
            units_per_em,
        })
    }

    fn face(&self) -> &Face<'_> {
        self.inner.as_face_ref()
    }
}

impl FontData for TtfFace {
    fn font_metrics(&self) -> FontMetrics {
        FontMetrics {
            ascent: self.face().ascender(),
            descent: self.face().descender(),
            units_per_em: self.units_per_em,
        }
    }

    fn glyph_id(&self, c: char) -> Option<u16> {
        self.face().glyph_index(c).map(|id| id.0)
    }

    fn glyph_ids(&self) -> HashMap<u16, char> {
        let subtables = self
            .face()
            .tables()
            .cmap
            .map(|cmap| cmap.subtables.into_iter().filter(|v| v.is_unicode()));
        let Some(subtables) = subtables else {
            return HashMap::new();
        };
        let mut map = HashMap::with_capacity(self.face().number_of_glyphs().into());
        for subtable in subtables {
            subtable.codepoints(|c| {
                use std::convert::TryFrom as _;

                if let Ok(ch) = char::try_from(c) {
                    if let Some(idx) = subtable.glyph_index(c).filter(|idx| idx.0 > 0) {
                        map.entry(idx.0).or_insert(ch);
                    }
                }
            })
        }
        map
    }

    fn glyph_count(&self) -> u16 {
        self.face().number_of_glyphs()
    }

    fn glyph_metrics(&self, glyph_id: u16) -> Option<GlyphMetrics> {
        let glyph_id = owned_ttf_parser::GlyphId(glyph_id);
        if let Some(width) = self.face().glyph_hor_advance(glyph_id) {
            let width = width as u32;
            let height = self
                .face()
                .glyph_bounding_box(glyph_id)
                .map(|bbox| bbox.y_max - bbox.y_min - self.face().descender())
                .unwrap_or(1000) as u32;
            Some(GlyphMetrics { width, height })
        } else {
            None
        }
    }
}
