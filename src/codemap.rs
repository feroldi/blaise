use std::ops::Add;
use std::ops::Sub;

#[derive(Debug, PartialEq, Eq)]
pub struct BytePos(pub usize);

impl BytePos {
    pub fn invalid() -> BytePos {
        BytePos(<usize>::max_value())
    }
}

impl Add for BytePos {
    type Output = BytePos;

    fn add(self, BytePos(n): BytePos) -> BytePos {
        BytePos(self.0 + n)
    }
}

impl Sub for BytePos {
    type Output = BytePos;

    fn sub(self, BytePos(n): BytePos) -> BytePos {
        BytePos(self.0 - n)
    }
}

pub struct CodeMap {
    file_name: String,
    src: String,
    lines: Vec<BytePos>,
}

impl CodeMap {
    pub fn from_string(file_name: impl Into<String>, contents: String) -> CodeMap {
        let lines = {
            let mut lines = vec![BytePos(0)];
            for (i, &c) in contents.as_bytes().iter().enumerate() {
                if c == b'\n' {
                    lines.push(BytePos(i + 1));
                }
            }
            lines
        };

        CodeMap {
            file_name: file_name.into(),
            src: contents,
            lines: lines,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calc_line_offsets() {
        let codemap =
            CodeMap::from_string("test", "first line.\nsecond line.\nthird line.\n".into());
        assert_eq!(4, codemap.lines.len());
        assert_eq!(BytePos(0), codemap.lines[0]);
        assert_eq!(BytePos(12), codemap.lines[1]);
        assert_eq!(BytePos(25), codemap.lines[2]);
        assert_eq!(BytePos(37), codemap.lines[3]);
    }
}
