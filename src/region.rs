use std::num::NonZeroUsize;

#[derive(Debug)]
pub struct Region {
    pub name: String,
    bytes: Box<[u8]>,
    pointer: usize,
}

impl Region {
    pub fn new(name: &str, size: NonZeroUsize) -> Region {
        return Region {
            name: String::from(name),
            bytes: vec![0; size.get()].into_boxed_slice(),
            pointer: 0,
        };
    }

    pub fn right(&mut self) -> () {
        if self.pointer == (self.bytes.len() - 1) {
            self.pointer = 0;
        } else {
            self.pointer += 1;
        }
    }

    pub fn left(&mut self) -> () {
        if self.pointer == 0 {
            self.pointer = self.bytes.len() - 1;
        } else {
            self.pointer -= 1;
        }
    }

    pub fn goto(&mut self, location: usize) -> () {
        self.pointer = location;
    }

    pub fn get(&self) -> u8 {
        return self.bytes[self.pointer];
    }

    pub fn set(&mut self, value: u8) -> () {
        self.bytes[self.pointer] = value;
    }

    pub fn increment(&mut self) -> () {
        self.bytes[self.pointer] = u8::wrapping_add(self.bytes[self.pointer], 1);
    }

    pub fn decrement(&mut self) -> () {
        self.bytes[self.pointer] = u8::wrapping_sub(self.bytes[self.pointer], 1);
    }
}
