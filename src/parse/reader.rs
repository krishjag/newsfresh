use std::io::BufRead;

use crate::error::NewsfreshError;

/// Streaming line-by-line reader for GKG tab-delimited files.
pub struct GkgReader<R: BufRead> {
    inner: R,
    line_buf: String,
    line_number: usize,
}

impl<R: BufRead> GkgReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            inner: reader,
            line_buf: String::with_capacity(8192),
            line_number: 0,
        }
    }
}

impl<R: BufRead> Iterator for GkgReader<R> {
    type Item = Result<(usize, String), NewsfreshError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.line_buf.clear();
        match self.inner.read_line(&mut self.line_buf) {
            Ok(0) => None,
            Ok(_) => {
                self.line_number += 1;
                let line = self.line_buf.trim_end_matches('\n').trim_end_matches('\r');
                if line.is_empty() {
                    self.next()
                } else {
                    Some(Ok((self.line_number, line.to_string())))
                }
            }
            Err(e) => Some(Err(NewsfreshError::Io(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn empty_input_yields_no_items() {
        let data = Cursor::new(b"");
        let reader = GkgReader::new(BufReader::new(data));
        let items: Vec<_> = reader.collect();
        assert!(items.is_empty());
    }

    #[test]
    fn single_line_yields_one_item() {
        let data = Cursor::new(b"hello world\n");
        let reader = GkgReader::new(BufReader::new(data));
        let items: Vec<_> = reader.collect();
        assert_eq!(items.len(), 1);
        let (line_num, content) = items[0].as_ref().unwrap();
        assert_eq!(*line_num, 1);
        assert_eq!(content, "hello world");
    }

    #[test]
    fn blank_lines_are_skipped() {
        let data = Cursor::new(b"first\n\n\nsecond\n\nthird\n");
        let reader = GkgReader::new(BufReader::new(data));
        let items: Vec<_> = reader.map(|r| r.unwrap()).collect();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0], (1, "first".to_string()));
        assert_eq!(items[1], (4, "second".to_string()));
        assert_eq!(items[2], (6, "third".to_string()));
    }
}
