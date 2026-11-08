//! Conversion between byte offsets and LSP positions.
//!
//! LSP positions count lines from 0 and columns in UTF-16 code units by
//! default. Clients may agree on UTF-8 instead, which is cheaper.

use lsp_types::{Position, Range};

/// How columns are counted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Encoding {
    /// Columns are UTF-8 bytes.
    Utf8,
    /// Columns are UTF-16 code units (the LSP default).
    #[default]
    Utf16,
}

/// Line start offsets of a text.
#[derive(Debug, Clone)]
pub struct LineIndex {
    line_starts: Vec<usize>,
    encoding: Encoding,
}

impl LineIndex {
    /// Builds the index for `text`.
    pub fn new(text: &str, encoding: Encoding) -> Self {
        let mut line_starts = vec![0];
        line_starts.extend(text.match_indices('\n').map(|(i, _)| i + 1));
        LineIndex {
            line_starts,
            encoding,
        }
    }

    /// LSP position of a byte offset.
    pub fn position(&self, text: &str, offset: usize) -> Position {
        let offset = floor_char_boundary(text, offset.min(text.len()));
        let line = self.line_starts.partition_point(|&s| s <= offset) - 1;
        let start = self.line_starts[line];
        let prefix = &text[start..offset];
        let character = match self.encoding {
            Encoding::Utf8 => prefix.len(),
            Encoding::Utf16 => prefix.encode_utf16().count(),
        };
        Position::new(line as u32, character as u32)
    }

    /// LSP range of a byte range.
    pub fn range(&self, text: &str, start: usize, end: usize) -> Range {
        Range::new(self.position(text, start), self.position(text, end))
    }

    /// Byte offset of an LSP position. Positions past the end of a line
    /// are moved to the end of that line, positions past the end of the
    /// text to the end of the text.
    pub fn offset(&self, text: &str, pos: Position) -> usize {
        let Some(&start) = self.line_starts.get(pos.line as usize) else {
            return text.len();
        };
        let end = self
            .line_starts
            .get(pos.line as usize + 1)
            .map(|&next| next - 1)
            .unwrap_or(text.len());
        let line = &text[start..end];
        let wanted = pos.character as usize;
        let mut units = 0;
        for (i, c) in line.char_indices() {
            if units >= wanted {
                return start + i;
            }
            units += match self.encoding {
                Encoding::Utf8 => c.len_utf8(),
                Encoding::Utf16 => c.len_utf16(),
            };
        }
        end
    }
}

fn floor_char_boundary(text: &str, mut offset: usize) -> usize {
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utf16_columns() {
        // `°` is one UTF-16 unit but two UTF-8 bytes; `🤖` is two units, four bytes.
        let text = "a 90°;\n🤖x\n";
        let idx = LineIndex::new(text, Encoding::Utf16);
        let semicolon = text.find(';').unwrap();
        assert_eq!(idx.position(text, semicolon), Position::new(0, 5));
        let x = text.find('x').unwrap();
        assert_eq!(idx.position(text, x), Position::new(1, 2));
        assert_eq!(idx.offset(text, Position::new(1, 2)), x);
        assert_eq!(idx.offset(text, Position::new(0, 5)), semicolon);
    }

    #[test]
    fn utf8_columns() {
        let text = "a 90°;\n";
        let idx = LineIndex::new(text, Encoding::Utf8);
        let semicolon = text.find(';').unwrap();
        assert_eq!(idx.position(text, semicolon), Position::new(0, 6));
        assert_eq!(idx.offset(text, Position::new(0, 6)), semicolon);
    }

    #[test]
    fn out_of_range_positions_are_clamped() {
        let text = "ab\ncd";
        let idx = LineIndex::new(text, Encoding::Utf16);
        assert_eq!(idx.offset(text, Position::new(0, 99)), 2);
        assert_eq!(idx.offset(text, Position::new(9, 0)), text.len());
        assert_eq!(idx.position(text, 999), Position::new(1, 2));
    }
}
