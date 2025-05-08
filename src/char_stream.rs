use std::io::{Bytes, Read};

pub struct CharStream<R: Read> {
    source: Bytes<R>,
    buffer: Option<char>,
}

impl<R: Read> CharStream<R> {
    pub fn new(source: R) -> CharStream<R> {
        return CharStream {
            source: source.bytes(),
            buffer: None,
        };
    }

    pub fn next(&mut self) -> Option<char> {
        if let Some(c) = self.buffer {
            self.buffer = None;
            return Some(c);
        }
        let mut buf: [u8; 4] = [255, 0, 0, 0];
        let mut last: usize = 0;
        while std::str::from_utf8(&buf[0..=last]).is_err() {
            buf[last] = self.source.next()?.unwrap();
            last += 1;
            if last == (buf.len() + 1) {
                panic!("Source is not valid utf-8");
            }
        }
        return unsafe { std::str::from_utf8_unchecked(&buf[0..=last]).chars().nth(0) };
    }

    pub fn peek(&mut self) -> Option<char> {
        if self.buffer.is_none() {
            self.buffer = self.next();
        }
        return self.buffer;
    }

    pub fn advance(&mut self) -> () {
        _ = self.next();
    }
}
