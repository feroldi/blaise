use std::ops::{Add, Sub};
use std::rc::Rc;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BytePos(pub usize);

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Span {
    pub start: BytePos,
    pub end: BytePos,
}

pub trait Pos {
    fn from_usize(value: usize) -> Self;
    fn to_usize(&self) -> usize;
}

pub const DUMMY_BPOS: BytePos = BytePos(0);
pub const DUMMY_SPAN: Span = Span {
    start: DUMMY_BPOS,
    end: DUMMY_BPOS,
};

impl Pos for BytePos {
    fn from_usize(value: usize) -> BytePos {
        BytePos(value)
    }

    fn to_usize(&self) -> usize {
        self.0
    }
}

impl Add for BytePos {
    type Output = BytePos;

    fn add(self, rhs: BytePos) -> BytePos {
        BytePos(self.0 + rhs.0)
    }
}

impl Sub for BytePos {
    type Output = BytePos;

    fn sub(self, rhs: BytePos) -> BytePos {
        BytePos(self.0 - rhs.0)
    }
}

pub struct Loc {
    source_file: Rc<SourceFile>,
    line: usize,
    col: BytePos,
}

/// Maps the content of a file into line and column positions.
pub struct SourceFile {
    /// File's content.
    pub src: Rc<String>,
    /// Name of the loaded file.
    name: String,
    /// Byte positions following every new line.
    lines: Vec<BytePos>,
}

impl SourceFile {
    pub fn new(name: String, src: String) -> SourceFile {
        let mut lines = vec![BytePos(0)];

        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                lines.push(BytePos(i + 1));
            }
        }

        SourceFile {
            src: Rc::new(src),
            name,
            lines,
        }
    }

    pub fn span_to_snippet(&self, s: Span) -> &str {
        &self.src[s.start.0..s.end.0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_line_positions() {
        let source_file = SourceFile::new(
            "test".into(),
            "first line.\nsecond line.\nthird line.\n".into(),
        );

        assert_eq!(BytePos(0), source_file.lines[0]);
        assert_eq!(BytePos(12), source_file.lines[1]);
        assert_eq!(BytePos(25), source_file.lines[2]);
        assert_eq!(BytePos(37), source_file.lines[3]);
    }

    #[test]
    fn get_snippets_from_span() {
        let source_file = SourceFile::new(
            "test".into(),
            "first line.\nsecond line.\nthird line.\n".into(),
        );

        let s = Span {
            start: BytePos(0),
            end: BytePos(5),
        };
        assert_eq!("first", source_file.span_to_snippet(s));

        let s = Span {
            start: BytePos(12),
            end: BytePos(18),
        };
        assert_eq!("second", source_file.span_to_snippet(s));
    }
}
