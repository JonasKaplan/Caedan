use std::{collections::HashSet, fs::File, io::{BufRead, BufReader}, iter::Peekable, num::NonZeroUsize, path::Path, str::{Chars, FromStr}};

#[derive(Debug)]
pub enum ParseError {
    BadData,
    DuplicateIdentifier,
    MalformedInstruction,
    MalformedLine,
    MalformedNumber,
    MalformedProcedureDeclaration,
    MalformedRegionDeclaration,
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
    Send(String),
    Receive(String),
    Call(String, Option<String>),
}

enum LineType {
    Region,
    Procedure,
    Comment,
    Whitespace,
    Err,
}

pub struct ParsedRegion {
    pub name: String,
    pub size: NonZeroUsize,
}

pub struct ParsedProcedure {
    pub name: String,
    pub instructions: Vec<ParsedInstruction>,
}


pub struct ParseResult {
    pub regions: Vec<ParsedRegion>,
    pub procedures: Vec<ParsedProcedure>,
}

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
                ParsedInstruction::Send(region) => references.push(ReferencedItem::Region(region)),
                ParsedInstruction::Call(procedure, None) => references.push(ReferencedItem::Procedure(procedure)),
                ParsedInstruction::Call(procedure, Some(region)) => {
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

fn skip_whitespace(iterator: &mut Peekable<Chars>) -> () {
    loop {
        match iterator.peek() {
            Some(c) if c.is_whitespace() => _ = iterator.next(),
            _ => break,
        }
    }
}

fn expect_keyword(iterator: &mut Peekable<Chars>, keyword: &str) -> Result<(), ParseError> {
    for keyword_c in keyword.chars() {
        match iterator.peek() {
            Some(c) if *c == keyword_c => _ = iterator.next(),
            _ => return Err(ParseError::MissingKeyword),
        }
    }
    return Ok(());
}

fn parse_identifier(iterator: &mut Peekable<Chars>) -> Result<String, ParseError> {
    let mut identifier = String::new();
    loop {
        match iterator.peek() {
            Some(c) if is_identifier_char(*c) => {
                identifier.push(*c);
                _ = iterator.next();
            },
            _ => break,
        }
    }
    if identifier.is_empty() {
        return Err(ParseError::MissingIdentifier);
    }
    return Ok(identifier);
}

fn parse_number<T: FromStr>(iterator: &mut Peekable<Chars>) -> Result<T, ParseError> {
    let mut text = String::new();
    loop {
        match iterator.peek() {
            Some(c) if c.is_numeric() => {
                text.push(*c);
                _ = iterator.next();
            },
            _ => break,
        }
    }
    return text.parse::<T>().map_err(|_| ParseError::MalformedNumber);
}

fn parse_instruction(iterator: &mut Peekable<Chars>) -> Result<ParsedInstruction, ParseError> {
    // I cannot express how much I hate this syntax
    let instruction: char = match iterator.peek() {
        Some(c) => *c,
        None => return Err(ParseError::MalformedInstruction),
    };
    if !is_identifier_char(instruction) {
        _ = iterator.next();
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
                match iterator.next() {
                    Some(c) => buf.push(c),
                    None => return Err(ParseError::MalformedInstruction),
                }
            }
            match u8::from_str_radix(&buf, 16) {
                Ok(value) => return Ok(ParsedInstruction::Quote(value)),
                Err(_) => return Err(ParseError::MalformedInstruction),
            }
        },
        '^' => return Ok(ParsedInstruction::Send(parse_identifier(iterator)?)),
        '&' => return Ok(ParsedInstruction::Receive(parse_identifier(iterator)?)),
        _ => {
            let procedure: String = parse_identifier(iterator)?;
            let mut region: Option<String> = None;
            skip_whitespace(iterator);
            if iterator.peek() == Option::Some(&'@') {
                _ = iterator.next();
                skip_whitespace(iterator);
                region = Some(parse_identifier(iterator)?);
            }
            return Ok(ParsedInstruction::Call(procedure, region));
        },
    }
}

fn make_anonymous_name(base_name: &str, anonymous_count: usize) -> String {
    let mut name: String = base_name.to_string();
    name.push_str("-anon-");
    name.push_str(&anonymous_count.to_string());
    return name;
}

fn parse_instruction_list(iterator: &mut Peekable<Chars>, name: &str) -> Result<Vec<(String, Vec<ParsedInstruction>)>, ParseError> {
    let mut anonymous_count: usize = 0;
    let mut anonymous_procedures: Vec<(String, Vec<ParsedInstruction>)> = Vec::new();
    let mut instructions: Vec<ParsedInstruction> = Vec::new();
    loop {
        skip_whitespace(iterator);
        match iterator.peek() {
            Some(c) if is_instruction_char(*c) => instructions.push(parse_instruction(iterator)?),
            Some(c) if *c == '(' => {
                _ = iterator.next();
                anonymous_count += 1;
                let anonymous_name = make_anonymous_name(name, anonymous_count);
                anonymous_procedures.append(&mut parse_instruction_list(iterator, &anonymous_name)?);
                _ = iterator.next();
                let mut region: Option<String> = None;
                skip_whitespace(iterator);
                if iterator.peek() == Option::Some(&'@') {
                    _ = iterator.next();
                    region = Some(parse_identifier(iterator)?);
                }
                instructions.push(ParsedInstruction::Call(anonymous_name, region));
            },
            Some(c) if *c != ')' => return Err(ParseError::MalformedProcedureDeclaration),
            _ => break,
        }
    }
    anonymous_procedures.push((name.to_string(), instructions));
    return Ok(anonymous_procedures);
}

fn parse_region(line: &str) -> Result<ParsedRegion, ParseError> {
    let mut iterator: Peekable<Chars> = line.chars().peekable();
    skip_whitespace(&mut iterator);
    expect_keyword(&mut iterator, "region")?;
    skip_whitespace(&mut iterator);
    let name: String = parse_identifier(&mut iterator)?;
    skip_whitespace(&mut iterator);
    expect_keyword(&mut iterator, "[")?;
    skip_whitespace(&mut iterator);
    // Again, I hate this. Sucks for me.
    let size: NonZeroUsize = match NonZeroUsize::new(parse_number::<usize>(&mut iterator)?) {
        Some(s) => s,
        None => return Err(ParseError::MalformedNumber),
    };
    expect_keyword(&mut iterator, "]")?;
    skip_whitespace(&mut iterator);
    if iterator.peek().is_some() {
        return Err(ParseError::MalformedRegionDeclaration);
    }
    return Ok(ParsedRegion { name, size });
}

fn parse_procedure(line: &str) -> Result<Vec<ParsedProcedure>, ParseError> {
    let mut procedures: Vec<ParsedProcedure> = Vec::new();
    let mut iterator: Peekable<Chars> = line.chars().peekable();
    skip_whitespace(&mut iterator);
    expect_keyword(&mut iterator, "proc")?;
    skip_whitespace(&mut iterator);
    let name: String = parse_identifier(&mut iterator)?;
    expect_keyword(&mut iterator, ":")?;
    let all_procedures: Vec<(String, Vec<ParsedInstruction>)> = parse_instruction_list(&mut iterator, &name)?;
    for (name, instructions) in all_procedures.into_iter() {
        procedures.push(ParsedProcedure { name, instructions });
    }
    if iterator.peek().is_some() {
        return Err(ParseError::MalformedProcedureDeclaration);
    }
    return Ok(procedures);
}

fn get_line_type(line: &str) -> LineType {
    if line.is_empty() {
        return LineType::Whitespace;
    }
    for c in line.chars() {
        if !c.is_whitespace() {
            match c {
                'r' => return LineType::Region,
                'p' => return LineType::Procedure,
                '#' => return LineType::Comment,
                _ => return LineType::Err,
            }
        }
    }
    return LineType::Whitespace;
}

pub fn parse(source_path: &Path) -> Result<ParseResult, ParseError> {
    let reader: BufReader<File> = BufReader::new(File::open(source_path).map_err(|_| ParseError::MissingFile)?);
    let mut result: ParseResult = ParseResult::new();
    for line in reader.lines() {
        match line.map(|text| (get_line_type(&text), text)) {
            Ok((LineType::Region, text)) => result.regions.push(parse_region(&text)?),
            Ok((LineType::Procedure, text)) => result.procedures.append(&mut parse_procedure(&text)?),
            Ok((LineType::Comment, _)) | Ok((LineType::Whitespace, _)) => continue,
            Ok((LineType::Err, _)) => return Err(ParseError::MalformedLine),
            Err(_) => return Err(ParseError::BadData),
        }
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
