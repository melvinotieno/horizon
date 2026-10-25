use std::{borrow::Cow, collections::HashMap};

use horizon_theme::Tokens;
use parley::{
    Alignment, AlignmentOptions, FontContext, FontFamily, FontFeatures, FontWeight, Layout,
    LayoutContext, LineHeight, PositionedLayoutItem, StyleProperty,
};
use vello::{
    Glyph, Scene,
    kurbo::{Affine, Point},
    peniko::{Color, Fill},
};

/// The maximum number of text layouts to cache before clearing the cache.
const CACHE_LIMIT: usize = 1024;

/// A key for caching text layouts, based on the text and its style parameters.
#[derive(Clone, PartialEq, Eq, Hash)]
struct Key {
    text: String,
    style: String,
    weight: u32,
    width: u32,
    scale: u32,
}

/// A text style request used to look up typography's font style.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    /// The name of the font style in the typography tokens.
    pub style: &'static str,
    /// An optional weight to override the font style's weight.
    pub weight: Option<f32>,
}

/// A system for managing text layout and rendering, with caching for performance.
pub struct TextSystem {
    /// The font context used to manage fonts.
    pub(crate) fonts: FontContext,
    /// Scratch space parley reuses between layouts.
    pub(crate) layouts: LayoutContext<()>,
    /// A cache for storing previously computed text layouts.
    cache: HashMap<Key, Layout<()>>,
    /// The font family to use for text rendering, as specified in the tokens.
    family: String,
    /// The font features to use for text rendering, as specified in the tokens.
    features: String,
}

impl TextSystem {
    pub fn new(tokens: &Tokens) -> Self {
        Self {
            fonts: FontContext::new(),
            layouts: LayoutContext::new(),
            cache: HashMap::new(),
            family: format!("\"{}\", sans-serif", tokens.font_family),
            features: tokens.font_features.clone(),
        }
    }

    /// Draws the specified text onto the given scene, applying the provided style, color, and transform.
    pub fn draw(
        &mut self,
        tokens: &Tokens,
        scene: &mut Scene,
        text: &str,
        text_style: TextStyle,
        max_width: Option<f64>,
        color: Color,
        origin: Point,
        affine: Affine,
        scale: f64,
        hint: bool,
    ) {
        let layout = self.layout(tokens, text, text_style, max_width, scale);
        let device = affine * Affine::translate(origin.to_vec2()) * Affine::scale(1.0 / scale);

        let mut transform = device;

        // If hinting is enabled, adjust the transform to snap the text origin to device pixels.
        if hint {
            let o = transform * Point::ORIGIN;
            transform = Affine::translate((o.x.round() - o.x, o.y.round() - o.y)) * transform;
        }

        // Draw each line of the layout, applying the transform and color.
        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(glyph_run) = item else {
                    continue;
                };
                let run = glyph_run.run();
                let coords: Vec<i16> = run
                    .normalized_coords()
                    .iter()
                    .map(|c| c.to_bits())
                    .collect();
                scene
                    .draw_glyphs(run.font())
                    .font_size(run.font_size())
                    .normalized_coords(&coords)
                    .transform(transform)
                    .hint(hint)
                    .brush(color)
                    .draw(
                        Fill::NonZero,
                        glyph_run.positioned_glyphs().map(|g| Glyph {
                            id: g.id,
                            x: g.x,
                            y: g.y,
                        }),
                    );
            }
        }
    }

    /// Builds or retrieves a cached layout for the requested text parameters.
    fn layout(
        &mut self,
        tokens: &Tokens,
        text: &str,
        text_style: TextStyle,
        max_width: Option<f64>,
        scale: f64,
    ) -> &Layout<()> {
        let key = Key {
            text: text.to_string(),
            style: text_style.style.to_string(),
            weight: text_style.weight.unwrap_or(0.0) as u32,
            width: max_width.map_or(u32::MAX, |w| (w * 64.0) as u32),
            scale: (scale * 1000.0) as u32,
        };

        if !self.cache.contains_key(&key) {
            // If the cache exceeds the limit, clear it to free up space for new layouts.
            if self.cache.len() >= CACHE_LIMIT {
                self.cache.clear();
            }

            let font_style = tokens.font_style(text_style.style);

            // Create a layout builder with the specified style properties.
            let mut builder =
                self.layouts
                    .ranged_builder(&mut self.fonts, text, scale as f32, true);
            builder.push_default(StyleProperty::FontFamily(FontFamily::Source(Cow::Owned(
                self.family.clone(),
            ))));
            builder.push_default(StyleProperty::FontSize(font_style.size));
            builder.push_default(StyleProperty::FontWeight(FontWeight::new(
                text_style.weight.unwrap_or(font_style.weight),
            )));
            builder.push_default(StyleProperty::LineHeight(LineHeight::Absolute(
                font_style.line,
            )));
            builder.push_default(StyleProperty::LetterSpacing(
                font_style.tracking * font_style.size,
            ));
            if !self.features.is_empty() {
                builder.push_default(StyleProperty::FontFeatures(FontFeatures::Source(
                    Cow::Owned(self.features.clone()),
                )));
            }

            // Build the layout and apply line breaking and alignment.
            let mut layout = builder.build(text);
            layout.break_all_lines(max_width.map(|w| (w * scale) as f32));
            layout.align(Alignment::Start, AlignmentOptions::default());

            self.cache.insert(key.clone(), layout);
        }

        &self.cache[&key]
    }
}
