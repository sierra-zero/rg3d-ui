use crate::{
    core::{
        color::Color,
        math::{
            vec2::Vec2,
            Rect,
        },
    },
    ttf::Font,
    HorizontalAlignment,
    VerticalAlignment,
    brush::Brush
};
use std::{
    ops::Range,
    sync::{
        Arc,
        Mutex,
    },
};

#[derive(Debug, Clone)]
pub struct TextGlyph {
    bounds: Rect<f32>,
    tex_coords: [Vec2; 4]
}

impl TextGlyph {
    pub fn get_bounds(&self) -> Rect<f32> {
        self.bounds
    }

    pub fn get_tex_coords(&self) -> &[Vec2; 4] {
        &self.tex_coords
    }
}

#[derive(Copy, Clone, Debug)]
pub struct TextLine {
    /// Index of starting symbol in text array.
    pub begin: usize,
    /// Index of ending symbol in text array.
    pub end: usize,
    /// Total width of line.
    pub width: f32,
    /// Total height of line. Usually just ascender of a font.
    pub height: f32,
    /// Local horizontal position of line.
    pub x_offset: f32,
    /// Local vertical position of line.
    pub y_offset: f32,
}

impl TextLine {
    fn new() -> TextLine {
        TextLine {
            begin: 0,
            end: 0,
            width: 0.0,
            height: 0.0,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }

    pub fn len(&self) -> usize {
        self.end - self.begin
    }

    pub fn is_empty(&self) -> bool {
        self.end == self.begin
    }
}

#[derive(Clone)]
pub struct FormattedText {
    font: Option<Arc<Mutex<Font>>>,
    /// Text in UTF32 format.
    text: Vec<u32>,
    /// Temporary buffer used to split text on lines. We need it to reduce memory allocations
    /// when we changing text too frequently, here we sacrifice some memory in order to get
    /// more performance.
    lines: Vec<TextLine>,
    /// Final glyphs for draw buffer.
    glyphs: Vec<TextGlyph>,
    vertical_alignment: VerticalAlignment,
    horizontal_alignment: HorizontalAlignment,
    brush: Brush,
    constraint: Vec2,
    wrap: bool,
}

impl FormattedText {
    pub fn get_glyphs(&self) -> &[TextGlyph] {
        &self.glyphs
    }

    pub fn get_font(&self) -> Option<Arc<Mutex<Font>>> {
        self.font.clone()
    }

    pub fn set_font(&mut self, font: Arc<Mutex<Font>>) -> &mut Self {
        self.font = Some(font);
        self
    }

    pub fn get_lines(&self) -> &[TextLine] {
        &self.lines
    }

    pub fn set_vertical_alignment(&mut self, vertical_alignment: VerticalAlignment) -> &mut Self {
        self.vertical_alignment = vertical_alignment;
        self
    }

    pub fn vertical_alignment(&self) -> VerticalAlignment {
        self.vertical_alignment
    }

    pub fn set_horizontal_alignment(&mut self, horizontal_alignment: HorizontalAlignment) -> &mut Self {
        self.horizontal_alignment = horizontal_alignment;
        self
    }

    pub fn horizontal_alignment(&self) -> HorizontalAlignment {
        self.horizontal_alignment
    }

    pub fn set_brush(&mut self, brush: Brush) -> &mut Self {
        self.brush = brush;
        self
    }

    pub fn brush(&self) -> Brush {
        self.brush.clone()
    }

    pub fn set_constraint(&mut self, constraint: Vec2) -> &mut Self {
        self.constraint = constraint;
        self
    }

    pub fn get_raw_text(&self) -> &[u32] {
        &self.text
    }

    pub fn text(&self) -> String {
        self.text
            .iter()
            .filter_map(|c| std::char::from_u32(*c))
            .collect()
    }

    pub fn get_range_width(&self, range: Range<usize>) -> f32 {
        let mut width = 0.0;
        if let Some(ref font) = self.font {
            let font = font.lock().unwrap();
            for index in range {
                width += font.get_glyph_advance(self.text[index]);
            }
        }
        width
    }

    pub fn set_text<P: AsRef<str>>(&mut self, text: P) -> &mut Self {
        // Convert text to UTF32.
        self.text.clear();
        for code in text.as_ref().chars().map(|c| c as u32) {
            self.text.push(code);
        }
        self
    }

    pub fn set_wrap(&mut self, wrap: bool) -> &mut Self {
        self.wrap = wrap;
        self
    }

    pub fn is_wrap(&self) -> bool {
        self.wrap
    }

    pub fn insert_char(&mut self, c: char, index: usize) -> &mut Self {
        let c = c as u32;
        if index == self.text.len() {
            self.text.push(c);
        } else {
            self.text.insert(index, c);
        }
        self
    }

    pub fn remove_at(&mut self, index: usize) -> &mut Self {
        self.text.remove(index);
        self
    }

    pub fn build(&mut self) -> Vec2 {
        let font = if let Some(font) = &self.font {
            font.lock().unwrap()
        } else {
            return Vec2::ZERO;
        };

        // Split on lines.
        let mut total_height = 0.0;
        let mut current_line = TextLine::new();
        self.lines.clear();
        for (i, code) in self.text.iter().enumerate() {
            let advance =
                match font.get_glyph(*code) {
                    Some(glyph) => glyph.get_advance(),
                    None => font.get_height()
                };
            let is_new_line = *code == u32::from(b'\n') || *code == u32::from(b'\r');
            let new_width = current_line.width + advance;
            if self.wrap && new_width > self.constraint.x || is_new_line {
                self.lines.push(current_line);
                current_line.begin = if is_new_line { i + 1 } else { i };
                current_line.end = current_line.begin + 1;
                current_line.width = advance;
                total_height += font.get_ascender();
            } else {
                current_line.width = new_width;
                current_line.end += 1;
            }
        }
        // Commit rest of text.
        if current_line.begin != current_line.end {
            current_line.end = self.text.len();
            self.lines.push(current_line);
            total_height += font.get_ascender();
        }

        // Align lines according to desired alignment.
        for line in self.lines.iter_mut() {
            match self.horizontal_alignment {
                HorizontalAlignment::Left => line.x_offset = 0.0,
                HorizontalAlignment::Center => {
                    if self.constraint.x.is_infinite() {
                        line.x_offset = 0.0;
                    } else {
                        line.x_offset = 0.5 * (self.constraint.x - line.width);
                    }
                }
                HorizontalAlignment::Right => {
                    if self.constraint.x.is_infinite() {
                        line.x_offset = 0.0;
                    } else {
                        line.x_offset = self.constraint.x - line.width
                    }
                },
                HorizontalAlignment::Stretch => line.x_offset = 0.0
            }
        }

        // Generate glyphs for each text line.
        self.glyphs.clear();

        let cursor_y_start = match self.vertical_alignment {
            VerticalAlignment::Top => 0.0,
            VerticalAlignment::Center => {
                if self.constraint.y.is_infinite() {
                    0.0
                } else {
                    (self.constraint.y - total_height) * 0.5
                }
            },
            VerticalAlignment::Bottom => {
                if self.constraint.y.is_infinite() {
                    0.0
                } else {
                    self.constraint.y - total_height
                }
            },
            VerticalAlignment::Stretch => 0.0
        };

        let cursor_x_start = if self.constraint.x.is_infinite() {
            0.0
        } else {
            self.constraint.x
        };

        let mut cursor = Vec2::new(cursor_x_start, cursor_y_start);
        for line in self.lines.iter_mut() {
            cursor.x = line.x_offset;

            for code_index in line.begin..line.end {
                let code = self.text[code_index];

                match font.get_glyph(code) {
                    Some(glyph) => {
                        // Insert glyph
                        if glyph.has_outline() {
                            let rect = Rect {
                                x: cursor.x + glyph.get_bitmap_left(),
                                y: cursor.y + font.get_ascender() - glyph.get_bitmap_top() - glyph.get_bitmap_height(),
                                w: glyph.get_bitmap_width(),
                                h: glyph.get_bitmap_height(),
                            };
                            let text_glyph = TextGlyph {
                                bounds: rect,
                                tex_coords: *glyph.get_tex_coords(),
                            };
                            self.glyphs.push(text_glyph);
                        }
                        cursor.x += glyph.get_advance();
                    }
                    None => {
                        // Insert invalid symbol
                        let rect = Rect {
                            x: cursor.x,
                            y: cursor.y + font.get_ascender(),
                            w: font.get_height(),
                            h: font.get_height(),
                        };
                        self.glyphs.push(TextGlyph {
                            bounds: rect,
                            tex_coords: [Vec2::ZERO; 4]
                        });
                        cursor.x += rect.w;
                    }
                }
            }
            line.height = font.get_ascender();
            line.y_offset = cursor.y;
            cursor.y += font.get_ascender();
        }

        let mut full_size = Vec2::new(0.0, total_height);
        for line in self.lines.iter() {
            full_size.x = line.width.max(full_size.x);
        }
        full_size
    }
}

pub struct FormattedTextBuilder {
    font: Option<Arc<Mutex<Font>>>,
    brush: Brush,
    constraint: Vec2,
    text: String,
    vertical_alignment: VerticalAlignment,
    horizontal_alignment: HorizontalAlignment,
    wrap: bool,
}

impl Default for FormattedTextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FormattedTextBuilder {
    /// Creates new formatted text builder with default parameters.
    pub fn new() -> FormattedTextBuilder {
        FormattedTextBuilder {
            font: None,
            text: "".to_owned(),
            horizontal_alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Top,
            brush: Brush::Solid(Color::WHITE),
            constraint: Vec2::new(128.0, 128.0),
            wrap: false,
        }
    }

    pub fn with_font(mut self, font: Arc<Mutex<Font>>) -> Self {
        self.font = Some(font);
        self
    }

    pub fn with_vertical_alignment(mut self, vertical_alignment: VerticalAlignment) -> Self {
        self.vertical_alignment = vertical_alignment;
        self
    }

    pub fn with_wrap(mut self, wrap: bool) -> Self {
        self.wrap = wrap;
        self
    }

    pub fn with_horizontal_alignment(mut self, horizontal_alignment: HorizontalAlignment) -> Self {
        self.horizontal_alignment = horizontal_alignment;
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = text;
        self
    }

    pub fn with_constraint(mut self, constraint: Vec2) -> Self {
        self.constraint = constraint;
        self
    }

    pub fn with_brush(mut self, brush: Brush) -> Self {
        self.brush = brush;
        self
    }

    pub fn build(self) -> FormattedText {
        FormattedText {
            font: self.font,
            text: self.text.chars().map(|c| c as u32).collect(),
            lines: Vec::new(),
            glyphs: Vec::new(),
            vertical_alignment: self.vertical_alignment,
            horizontal_alignment: self.horizontal_alignment,
            brush: self.brush,
            constraint: self.constraint,
            wrap: self.wrap,
        }
    }
}