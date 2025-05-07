use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct Region {
    pub name: String,
    bytes: Vec<u8>,
    pointer: usize,
}

impl Region {
    pub fn new(name: &str, size: NonZeroUsize) -> Region {
        return Region {
            name: String::from(name),
            bytes: vec![0; size.get()],
            pointer: 0,
        };
    }

    pub fn jump(&mut self, location: usize, fallback: usize) -> () {
        if location >= self.bytes.len() {
            self.pointer = fallback;
        } else {
            self.pointer = location;
        }
    }

    pub fn right(&mut self) -> () {
        self.jump(usize::wrapping_add(self.pointer, 1), 0);
    }

    pub fn left(&mut self) -> () {
        self.jump(usize::wrapping_sub(self.pointer, 1), usize::wrapping_sub(self.bytes.len(), 1));
    }

    pub fn get(&self) -> u8 {
        return self.bytes[self.pointer];
    }

    pub fn set(&mut self, value: u8) -> () {
        self.bytes[self.pointer] = value;
    }
}
