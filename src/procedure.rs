use std::{cell::RefCell, collections::HashMap, io::{self, Read, Write}};

use crate::{parser::ParsedInstruction, program::Call, region::Region};

#[derive(Debug)]
pub enum Instruction {
    Right,
    Left,
    Reset,
    Plus,
    Minus,
    LoopStart(usize),
    LoopEnd(usize),
    Read,
    Write,
    Quote(u8),
    Send(String),
    Receive(String),
    Call(String, Option<String>),
}

#[derive(Debug)]
pub struct Procedure {
    pub name: String,
    instructions: Vec<Instruction>,
}

fn find_forwards(instructions: &[ParsedInstruction], starting_point: usize) -> usize {
    let mut total: i128 = 0;
    for (i, instruction) in instructions.iter().enumerate().skip(starting_point) {
        match instruction {
            ParsedInstruction::LoopStart => total += 1,
            ParsedInstruction::LoopEnd => total -= 1,
            _ => {},
        }
        if total == 0 {
            return i;
        }
    }
    panic!("No match found");
}

fn find_backwards(instructions: &[ParsedInstruction], starting_point: usize) -> usize {
    let mut total: i128 = 0;
    for i in (0..=starting_point).rev() {
        match instructions[i] {
            ParsedInstruction::LoopStart => total += 1,
            ParsedInstruction::LoopEnd => total -= 1,
            _ => {},
        }
        if total == 0 {
            return i;
        }
    }
    panic!("No match found");
}


impl Procedure {
    pub fn new(name: &str, parsed_instructions: Vec<ParsedInstruction>) -> Procedure {
        let mut instructions: Vec<Instruction> = Vec::new();
        for (i, instruction) in parsed_instructions.iter().enumerate() {
            match instruction {
                ParsedInstruction::Right => instructions.push(Instruction::Right),
                ParsedInstruction::Left => instructions.push(Instruction::Left),
                ParsedInstruction::Reset => instructions.push(Instruction::Reset),
                ParsedInstruction::Plus => instructions.push(Instruction::Plus),
                ParsedInstruction::Minus => instructions.push(Instruction::Minus),
                ParsedInstruction::LoopStart => instructions.push(Instruction::LoopStart(find_forwards(&parsed_instructions, i))),
                ParsedInstruction::LoopEnd => instructions.push(Instruction::LoopEnd(find_backwards(&parsed_instructions, i))),
                ParsedInstruction::Read => instructions.push(Instruction::Read),
                ParsedInstruction::Write => instructions.push(Instruction::Write),
                ParsedInstruction::Quote(value) => instructions.push(Instruction::Quote(*value)),
                ParsedInstruction::Send(region_name) => instructions.push(Instruction::Send(region_name.to_string())),
                ParsedInstruction::Receive(region_name) => instructions.push(Instruction::Receive(region_name.to_string())),
                ParsedInstruction::Call(procedure, region) => instructions.push(Instruction::Call(procedure.to_string(), region.clone())),
            }
        }
        return Procedure {
            name: name.to_string(),
            instructions,
        }
    }

    pub fn execute(&self, region: &mut Region, mut pointer: usize, regions: &HashMap<String, RefCell<Region>>) -> Option<Call> {
        //println!("{} @ {}", self.name, region.name);
        if (pointer == 0) && (self.instructions.is_empty()) {
            return None;
        }
        let mut return_pointer: Option<usize>;
        loop {
            if self.name.starts_with("lte") || self.name.starts_with("eq") {
                //println!("({}): {:?}", self.name, region);
            }
            match &self.instructions[pointer] {
                Instruction::LoopStart(location) if region.get() == 0 => pointer = *location,
                Instruction::LoopEnd(location) if region.get() != 0 => pointer = *location,
                _ => {},
            }
            let next: usize = usize::wrapping_add(pointer, 1);
            if (next == 0) || (next == self.instructions.len()) {
                return_pointer = None;
            } else {
                return_pointer = Some(next);
            }
            match &self.instructions[pointer] {
                Instruction::Right => region.right(),
                Instruction::Left => region.left(),
                Instruction::Reset => region.jump(0, 0),
                Instruction::Plus => region.set(u8::wrapping_add(region.get(), 1)),
                Instruction::Minus => region.set(u8::wrapping_sub(region.get(), 1)),
                Instruction::Read => {
                    let mut buf: [u8; 1] = [0; 1];
                    // No reason not to just panic if this fails, so the unwrap stays
                    io::stdin().read_exact(&mut buf).unwrap();
                    region.set(buf[0]);
                },
                // Same deal with the unwrap here
                Instruction::Write => io::stdout().write_all(&[region.get()]).unwrap(),
                Instruction::Quote(value) => region.set(*value),
                Instruction::Send(region_name) => {
                    if let Ok(mut reference) = regions.get(region_name).unwrap().try_borrow_mut() {
                        reference.set(region.get());
                    }
                },
                Instruction::Receive(region_name) => {
                    if let Ok(reference) = regions.get(region_name).unwrap().try_borrow() {
                        region.set(reference.get());
                    }
                },
                Instruction::Call(procedure_name, None) => {
                    return Some(Call {
                        procedure: procedure_name.to_string(),
                        region: region.name.to_string(),
                        return_pointer,
                    });
                },
                Instruction::Call(procedure_name, Some(region_name)) => {
                    return Some(Call {
                        procedure: procedure_name.to_string(),
                        region: region_name.to_string(),
                        return_pointer,
                    });
                },
                _ => {},
            }
            if let Some(next) = return_pointer {
                pointer = next;
            } else {
                return None;
            }
        }
    }
}
