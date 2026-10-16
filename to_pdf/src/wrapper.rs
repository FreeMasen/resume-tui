use owned_ttf_parser::{AsFaceRef, Face, OwnedFace};

const MM_IN_PT: f32 = 0.352_777_78;

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
///
/// This used to also `impl printpdf::FontData` to register the face with
/// printpdf 0.7. The 0.12 upgrade moved font registration to a different
/// surface (`printpdf::ParsedFont`), so the trait impl is gone; this
/// wrapper is now used purely to measure text widths for layout.
#[derive(Clone, Debug)]
pub struct TtfFace {
    inner: std::sync::Arc<OwnedFace>,
}

impl TtfFace {
    pub fn from_vec(v: Vec<u8>) -> Result<Self, String> {
        let face =
            OwnedFace::from_vec(v, 0).map_err(|e| format!("Error parsing font face: {e}"))?;
        Ok(Self {
            inner: std::sync::Arc::new(face),
        })
    }

    fn face(&self) -> &Face<'_> {
        self.inner.as_face_ref()
    }

    fn glyph_id(&self, c: char) -> Option<u16> {
        self.face().glyph_index(c).map(|id| id.0)
    }

    fn glyph_metrics(&self, glyph_id: u16) -> Option<GlyphMetrics> {
        let gid = owned_ttf_parser::GlyphId(glyph_id);
        self.face()
            .glyph_hor_advance(gid)
            .map(|width| GlyphMetrics {
                width: width as u32,
            })
    }
}

struct GlyphMetrics {
    width: u32,
}
