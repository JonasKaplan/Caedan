use std::{collections::HashSet, fs::File, num::NonZeroUsize, path::Path, str::FromStr};

use crate::{char_stream::CharStream, procedure::RegionReference};

#[derive(Debug)]
pub enum ParseError {
    DuplicateIdentifier,
    InvalidIdentifier,
    MalformedInstruction,
    MalformedLine,
    MalformedNumber,
    MalformedProcedureDeclaration,
    MissingFile,
    MissingIdentifier,
    MissingKeyword,
    UndefinedReference,
}

#[derive(Debug)]
pub enum ParsedInstruction {
    Right,
    Left,
    Reset,
    Plus,
    Minus,
    LoopStart,
    LoopEnd,
    Read,
    Write,
    Quote(u8),
    Send(RegionReference),
    Receive(RegionReference),
    Call(String, Option<RegionReference>),
}

#[derive(Debug)]
pub struct ParsedRegion {
    pub name: String,
    pub size: NonZeroUsize,
}

#[derive(Debug)]
pub struct ParsedProcedure {
    pub name: String,
    pub is_anonymous: bool,
    pub instructions: Vec<ParsedInstruction>,
}

#[derive(Debug)]
pub struct ParseResult {
    pub regions: Vec<ParsedRegion>,
    pub procedures: Vec<ParsedProcedure>,
}

#[derive(Debug)]
pub enum ReferencedItem<'a> {
    Procedure(&'a str),
    Region(&'a str),
}

impl ParseResult {
    pub fn new() -> ParseResult {
        return ParseResult {
            regions: Vec::new(),
            procedures: Vec::new(),
        }
    }
}

impl ParsedProcedure {
    pub fn get_all_references(&self) -> Vec<ReferencedItem> {
        let mut references: Vec<ReferencedItem> = Vec::new();
        for instruction in &self.instructions {
            match instruction {
                ParsedInstruction::Send(RegionReference::Named(region)) => references.push(ReferencedItem::Region(region)),
                ParsedInstruction::Receive(RegionReference::Named(region)) => references.push(ReferencedItem::Region(region)),
                ParsedInstruction::Call(procedure, None) => references.push(ReferencedItem::Procedure(procedure)),
                ParsedInstruction::Call(procedure, Some(RegionReference::Named(region))) => {
                    references.push(ReferencedItem::Procedure(procedure));
                    references.push(ReferencedItem::Region(region));
                },
                _ => {},
            }
        }
        return references;
    }
}

fn is_identifier_char(c: char) -> bool {
    return c.is_ascii() && (c.is_alphanumeric() || (c == '_'));
}

fn is_instruction_char(c: char) -> bool {
    return
        is_identifier_char(c) ||
        (c == '>') ||
        (c == '<') ||
        (c == '~') ||
        (c == '+') ||
        (c == '-') ||
        (c == '[') ||
        (c == ']') ||
        (c == ',') ||
        (c == '.') ||
        (c == '"') ||
        (c == '^') ||
        (c == '&');
}

fn skip_whitespace(stream: &mut CharStream<File>) -> () {
    loop {
        match stream.peek() {
            Some(c) if c.is_whitespace() => stream.advance(),
            _ => break,
        }
    }
}

fn skip_comment(stream: &mut CharStream<File>) -> () {
    loop {
        match stream.peek() {
            Some('\n') | None => break,
            _ => stream.advance(),
        }
    }
    stream.advance();
}

fn expect_keyword(stream: &mut CharStream<File>, keyword: &str) -> Result<(), ParseError> {
    for keyword_c in keyword.chars() {
        if stream.next().ok_or(ParseError::MissingKeyword)? != keyword_c {
            return Err(ParseError::MissingKeyword);
        }
    }
    return Ok(());
}

fn parse_identifier(stream: &mut CharStream<File>) -> Result<String, ParseError> {
    let mut identifier = String::new();
    loop {
        match stream.peek() {
            Some(c) if is_identifier_char(c) => {
                identifier.push(c);
                stream.advance();
            },
            _ => break,
        }
    }
    if identifier.is_empty() {
        return Err(ParseError::MissingIdentifier);
    }
    if (identifier == "proc") || (identifier == "region") {
        return Err(ParseError::InvalidIdentifier);
    }
    return Ok(identifier);
}

fn parse_number<T: FromStr>(stream: &mut CharStream<File>) -> Result<T, ParseError> {
    let mut text = String::new();
    loop {
        match stream.peek() {
            Some(c) if c.is_numeric() => {
                text.push(c);
                stream.advance();
            },
            _ => break,
        }
    }
    return text.parse::<T>().map_err(|_| ParseError::MalformedNumber);
}

fn parse_region_reference(stream: &mut CharStream<File>) -> Result<RegionReference, ParseError> {
    match stream.peek() {
        Some('$') => {
            stream.advance();
            return Ok(RegionReference::BackReference);
        },
        Some(_) => {
            return Ok(RegionReference::Named(parse_identifier(stream)?));
        }
        _ => return Err(ParseError::MissingIdentifier),
    }
}

fn parse_instruction(stream: &mut CharStream<File>) -> Result<ParsedInstruction, ParseError> {
    let instruction: char = stream.peek().ok_or(ParseError::MalformedInstruction)?;
    if !is_identifier_char(instruction) {
        stream.advance();
    }
    match instruction {
        '>' => return Ok(ParsedInstruction::Right),
        '<' => return Ok(ParsedInstruction::Left),
        '~' => return Ok(ParsedInstruction::Reset),
        '+' => return Ok(ParsedInstruction::Plus),
        '-' => return Ok(ParsedInstruction::Minus),
        '[' => return Ok(ParsedInstruction::LoopStart),
        ']' => return Ok(ParsedInstruction::LoopEnd),
        ',' => return Ok(ParsedInstruction::Read),
        '.' => return Ok(ParsedInstruction::Write),
        '"' => {
            let mut buf = String::new();
            for _ in 0..2 {
                match stream.next() {
                    Some(c) => buf.push(c),
                    None => return Err(ParseError::MalformedInstruction),
                }
            }
            if let Ok(value) = u8::from_str_radix(&buf, 16) {
                return Ok(ParsedInstruction::Quote(value));
            } else {
                return Err(ParseError::MalformedInstruction);
            }
        },
        '^' => {
            skip_whitespace(stream);
            return Ok(ParsedInstruction::Send(parse_region_reference(stream)?));
        },
        '&' => {
            skip_whitespace(stream);
            return Ok(ParsedInstruction::Receive(parse_region_reference(stream)?));
        },
        _ => {
            let procedure: String = parse_identifier(stream)?;
            skip_whitespace(stream);
            match stream.peek() {
                Some('@') => {
                    stream.advance();
                    return Ok(ParsedInstruction::Call(procedure, Some(parse_region_reference(stream)?)));
                }
                _ => return Ok(ParsedInstruction::Call(procedure, None)),
            }
        },
    }
}

fn make_anonymous_name(base_name: &str, anonymous_count: usize) -> String {
    let mut name: String = base_name.to_string();
    name.push_str("-anon-");
    name.push_str(&anonymous_count.to_string());
    return name;
}

fn parse_instruction_list(stream: &mut CharStream<File>, name: &str) -> Result<Vec<(String, Vec<ParsedInstruction>)>, ParseError> {
    let mut anonymous_count: usize = 0;
    let mut anonymous_procedures: Vec<(String, Vec<ParsedInstruction>)> = Vec::new();
    let mut instructions: Vec<ParsedInstruction> = Vec::new();
    loop {
        skip_whitespace(stream);
        match stream.peek() {
            Some(c) if is_instruction_char(c) => instructions.push(parse_instruction(stream)?),
            Some('(') => {
                stream.advance();
                let anonymous_name = make_anonymous_name(name, anonymous_count);
                anonymous_procedures.append(&mut parse_instruction_list(stream, &anonymous_name)?);
                anonymous_count += 1;
                stream.advance();
                skip_whitespace(stream);
                match stream.peek() {
                    Some('@') => {
                        stream.advance();
                        instructions.push(ParsedInstruction::Call(anonymous_name, Some(parse_region_reference(stream)?)));
                    }
                    _ => instructions.push(ParsedInstruction::Call(anonymous_name, None)),
                }
            },
            Some(';') => break,
            Some(c) if c != ')' => return Err(ParseError::MalformedProcedureDeclaration),
            _ => break,
        }
    }
    anonymous_procedures.push((name.to_string(), instructions));
    return Ok(anonymous_procedures);
}

fn parse_region(stream: &mut CharStream<File>) -> Result<ParsedRegion, ParseError> {
    expect_keyword(stream, "region")?;
    skip_whitespace(stream);
    let name: String = parse_identifier(stream)?;
    skip_whitespace(stream);
    expect_keyword(stream, "[")?;
    skip_whitespace(stream);
    // Again, I hate this. Sucks for me.
    let size: NonZeroUsize = match NonZeroUsize::new(parse_number::<usize>(stream)?) {
        Some(s) => s,
        None => return Err(ParseError::MalformedNumber),
    };
    expect_keyword(stream, "]")?;
    skip_whitespace(stream);
    expect_keyword(stream, ";")?;
    return Ok(ParsedRegion { name, size });
}

fn parse_procedure(stream: &mut CharStream<File>) -> Result<Vec<ParsedProcedure>, ParseError> {
    let mut procedures: Vec<ParsedProcedure> = Vec::new();
    expect_keyword(stream, "proc")?;
    skip_whitespace(stream);
    let name: String = parse_identifier(stream)?;
    expect_keyword(stream, ":")?;
    let all_procedures: Vec<(String, Vec<ParsedInstruction>)> = parse_instruction_list(stream, &name)?;
    expect_keyword(stream, ";")?;
    for (name, instructions) in all_procedures.into_iter() {
        procedures.push(ParsedProcedure { name, instructions, is_anonymous: true });
    }
    // There is always at least one element
    procedures.last_mut().unwrap().is_anonymous = false;
    return Ok(procedures);
}

pub fn parse(source_path: &Path) -> Result<ParseResult, ParseError> {
    let stream: &mut CharStream<File> = &mut CharStream::new(File::open(source_path).map_err(|_| ParseError::MissingFile)?);
    let mut result: ParseResult = ParseResult::new();

    skip_whitespace(stream);
    while let Some(c) = stream.peek() {
        match c {
            'r' => result.regions.push(parse_region(stream)?),
            'p' => result.procedures.append(&mut parse_procedure(stream)?),
            '#' => skip_comment(stream),
            _ => return Err(ParseError::MalformedLine),
        }
        skip_whitespace(stream);
    }

    // Verify that all references are resolved before execution, to avoid runtime issues
    let mut procedure_names: HashSet<&str> = HashSet::new();
    let mut region_names: HashSet<&str> = HashSet::new();
    for procedure in &result.procedures {
        if !procedure_names.insert(&procedure.name) {
            return Err(ParseError::DuplicateIdentifier);
        }
    }
    for region in &result.regions {
        if !region_names.insert(&region.name) {
            return Err(ParseError::DuplicateIdentifier);
        }
    }
    for procedure in &result.procedures {
        for reference in procedure.get_all_references() {
            match reference {
                ReferencedItem::Region(region) if region_names.contains(region) => {},
                ReferencedItem::Procedure(procedure) if procedure_names.contains(procedure) => {},
                _ => return Err(ParseError::UndefinedReference),
            }
        }
    }
    return Ok(result);
}
