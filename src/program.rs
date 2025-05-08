use std::{cell::RefCell, collections::{HashMap, VecDeque}, path::Path};

use crate::{parser::{parse, ParseResult, ParseError}, procedure::Procedure, region::Region};

#[derive(Debug)]
pub struct Program {
    regions: HashMap<String, RefCell<Region>>,
    procedures: HashMap<String, Procedure>,
}

pub struct StackFrame {
    pub procedure: String,
    pub region: String,
    pub pointer: usize,
}

impl StackFrame {
    pub fn new(procedure: &str, region: &str, pointer: usize) -> StackFrame {
        return StackFrame {
            procedure: procedure.to_string(),
            region: region.to_string(),
            pointer,
        };
    }
}

pub struct Call {
    pub procedure: String,
    pub region: String,
    pub return_pointer: Option<usize>,
}

impl Program {
    pub fn from_source(source_path: &Path) -> Result<Program, ParseError> {
        let result: ParseResult = parse(source_path)?;
        let mut regions: HashMap<String, RefCell<Region>> = HashMap::new();
        let mut procedures: HashMap<String, Procedure> = HashMap::new();
        for region in result.regions.into_iter() {
            regions.insert(region.name.clone(), RefCell::new(Region::new(&region.name, region.size)));
        }
        for procedure in result.procedures.into_iter() {
            procedures.insert(procedure.name.clone(), Procedure::new(&procedure.name, procedure.instructions, procedure.is_anonymous));
        }
        return Ok(Program { regions, procedures });
    }

    // References are checked at compile time, so these will never fail
    pub fn get_region(&self, name: &str) -> &RefCell<Region> {
        return self.regions.get(name).unwrap();
    }

    pub fn get_procedure(&self, name: &str) -> &Procedure {
        return self.procedures.get(name).unwrap();
    }

    pub fn run(self) -> () {
        let mut call_stack: VecDeque<StackFrame> = VecDeque::new();
        call_stack.push_back(StackFrame::new("main", "main", 0));
        let mut back_reference: String = "main".to_string();
        while !call_stack.is_empty() {
            let frame: StackFrame = call_stack.pop_back().unwrap();
            let procedure: &Procedure = self.get_procedure(&frame.procedure);
            if !procedure.is_anonymous {
                back_reference = frame.region.clone();
            }
            let region: &mut Region = &mut self.get_region(&frame.region).borrow_mut();
            if let Some(call) = procedure.execute(region, frame.pointer, &self.regions, &back_reference) {
                if let Some(pointer) = call.return_pointer {
                    call_stack.push_back(StackFrame::new(&procedure.name, &region.name, pointer));
                }
                call_stack.push_back(StackFrame::new(&call.procedure, &call.region, 0));
            }
        }
    }
}
