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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Loc {
    pub line: usize,
    pub col: BytePos,
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

        lines.push(BytePos(src.len()));

        SourceFile {
            src: Rc::new(src),
            name,
            lines,
        }
    }

    pub fn span_to_snippet(&self, s: Span) -> &str {
        &self.src[s.start.0..s.end.0]
    }

    pub fn lookup_line_index(&self, pos: BytePos) -> Option<usize> {
        let pos_index = pos.to_usize();
        for (i, line_pos) in self.lines.iter().enumerate() {
            let line_pos_index = line_pos.to_usize();
            if pos_index < line_pos_index {
                return Some(i - 1);
            }
        }

        None
    }

    pub fn lookup_source_location(&self, pos: BytePos) -> Option<Loc> {
        self.lookup_line_index(pos).map(|line_index| {
            let line = line_index + 1;
            let col = pos - self.lines[line_index];

            Loc { line, col }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_source_file() -> SourceFile {
        SourceFile::new(
            "test".into(),
            "first line.\nsecond line.\nthird line.\n".into(),
        )
    }

    #[test]
    fn calc_line_positions_test() {
        let source_file = create_source_file();

        assert_eq!(BytePos(0), source_file.lines[0]);
        assert_eq!(BytePos(12), source_file.lines[1]);
        assert_eq!(BytePos(25), source_file.lines[2]);
        assert_eq!(BytePos(37), source_file.lines[3]);
    }

    #[test]
    fn get_snippets_from_span_test() {
        let source_file = create_source_file();

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

    #[test]
    fn lookup_line_indicies_test() {
        let source_file = create_source_file();

        assert_eq!(Some(0), source_file.lookup_line_index(BytePos(0)));
        assert_eq!(Some(0), source_file.lookup_line_index(BytePos(1)));
        assert_eq!(Some(1), source_file.lookup_line_index(BytePos(12)));
        assert_eq!(Some(2), source_file.lookup_line_index(BytePos(25)));
        assert_eq!(None, source_file.lookup_line_index(BytePos(37)));
    }

    #[test]
    fn lookup_source_locations_test() {
        let source_file = create_source_file();

        assert_eq!(
            Some(Loc {
                line: 1,
                col: BytePos(0),
            }),
            source_file.lookup_source_location(BytePos(0))
        );

        assert_eq!(
            Some(Loc {
                line: 1,
                col: BytePos(3),
            }),
            source_file.lookup_source_location(BytePos(3))
        );

        assert_eq!(
            Some(Loc {
                line: 2,
                col: BytePos(0),
            }),
            source_file.lookup_source_location(BytePos(12))
        );

        assert_eq!(
            Some(Loc {
                line: 2,
                col: BytePos(3),
            }),
            source_file.lookup_source_location(BytePos(15))
        );

        assert_eq!(
            None,
            source_file.lookup_source_location(BytePos(37))
        );
    }
}
