use std::{cmp::Ordering, collections::BTreeMap};

pub type Error = String;

pub type Integer = u64;
pub type RawCommand = u64;
pub type RawArgument = u64;

pub type RawProgram = Vec<(Integer, RawCommand, RawArgument, RawArgument)>;
pub type Heap = BTreeMap<Integer, Integer>;
pub type Program = BTreeMap<Integer, (Command, [Argument; 2])>;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Noop,
    Label,
    Branch,
    BranchGreaterEqual,
    Compare,
    Move,
    Add,
}

impl Command {}

#[derive(Debug, Clone, Copy)]
pub enum Argument {
    None,
    Raw(Integer),
    Register(u8),
    HeapRef(Integer),
    RawLabel(Integer),
}

#[derive(Debug, Clone)]
pub struct Registers {
    data: [Integer; 16],
}

impl Registers {
    pub fn new() -> Self {
        Registers { data: [0; 16] }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Flags {
    equal: bool,
    less: bool,
    greater: bool,
    overflow: bool,
}

impl Argument {
    pub fn resolve(&self, rt: &Runtime) -> Option<Integer> {
        match self {
            Argument::None => None,
            Argument::Raw(data) => Some(*data),
            Argument::Register(reg_id) => rt.registers.data.get(*reg_id as usize).copied(),
            Argument::HeapRef(ref_id) => Some(*rt.heap.get(&ref_id).unwrap()),
            Argument::RawLabel(_) => None,
        }
    }

    pub fn resolve_or_error(&self, rt: &Runtime) -> Result<Integer, Error> {
        self.resolve(rt)
            .ok_or_else(|| String::from("no valid argument"))
    }

    pub fn resolve_label(&self) -> Option<Integer> {
        match self {
            Argument::RawLabel(label_ref) => Some(*label_ref),
            _ => None,
        }
    }

    pub fn resolve_label_or_error(&self) -> Result<Integer, Error> {
        self.resolve_label()
            .ok_or_else(|| String::from("no valid argument"))
    }
}

#[derive(Debug, Clone)]
pub struct Runtime {
    flags: Flags,
    registers: Registers,
    program_counter: Integer,
    program: BTreeMap<Integer, (Command, [Argument; 2])>,
    heap: Heap,
    label_references: BTreeMap<Integer, Integer>,
}

impl Runtime {
    pub fn new(program: Program) -> Self {
        Runtime {
            flags: Flags::default(),
            registers: Registers::new(),
            heap: Heap::new(),
            program_counter: 0,
            label_references: Self::scan_labels(&program),
            program,
        }
    }

    fn scan_labels(program: &Program) -> BTreeMap<Integer, Integer> {
        let mut label_references = BTreeMap::new();
        for (index, (command, args)) in program.iter() {
            if let Command::Label = command {
                if let Some(label) = args[0].resolve_label() {
                    label_references.insert(label, *index);
                }
            }
        }
        label_references
    }

    pub fn run(&mut self) -> Result<(), Error> {
        loop {
            match self.tick() {
                Ok(true) => break,
                Ok(false) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn tick(&mut self) -> Result<bool, Error> {
        if let Some((command, args)) = self
            .program
            .get(&self.program_counter)
            .map(|(c, [a1, a2])| (*c, [*a1, *a2]))
        {
            self.apply_command(&command, &args)?;
            return Ok(false);
        };

        Ok(true)
    }

    pub fn apply_command(&mut self, command: &Command, args: &[Argument; 2]) -> Result<(), Error> {
        let mut increase_program_counter = true;
        match command {
            Command::Noop => (),
            Command::Move => {
                let new_value = args[1].resolve_or_error(self)?;

                match args[0] {
                    Argument::Register(reg) => self.registers.data[reg as usize] = new_value,
                    Argument::HeapRef(heap_id) => {
                        self.heap
                            .entry(heap_id)
                            .and_modify(|p| *p = new_value)
                            .or_insert(new_value);
                    }
                    _ => return Err("Invalid argument".to_string()),
                }
            }
            Command::Label => {
                // let label_ref = args[0].resolve_label_or_error(self)?;
                // self.label_references
                //     .insert(label_ref, self.program_counter);
            }
            Command::Branch => {
                self.brancher(args)?;
            }
            Command::BranchGreaterEqual => {
                if self.flags.equal || self.flags.greater {
                    self.brancher(args)?;
                }
            }
            Command::Compare => {
                let value_a = args[0].resolve_or_error(self)?;
                let value_b = args[1].resolve_or_error(self)?;

                match value_a.cmp(&value_b) {
                    Ordering::Equal => {
                        self.flags.equal = true;
                        self.flags.greater = false;
                        self.flags.less = false;
                    }
                    Ordering::Greater => {
                        self.flags.equal = false;
                        self.flags.greater = true;
                        self.flags.less = false;
                    }
                    Ordering::Less => {
                        self.flags.equal = false;
                        self.flags.greater = false;
                        self.flags.less = true;
                    }
                }
            }
            Command::Add => {
                self.calculate(command, args)?;
            }
        }

        self.program_counter += increase_program_counter as Integer;

        Ok(())
    }

    pub fn output(&self) -> Integer {
        self.registers.data[0]
    }

    fn brancher(&mut self, args: &[Argument; 2]) -> Result<(), Error> {
        let label_ref = args[0].resolve_label_or_error()?;
        self.label_references
            .get(&label_ref)
            .map(|p| self.program_counter = *p)
            .ok_or("Label not found".to_string())?;

        Ok(())
    }

    fn calculate(&mut self, command: &Command, args: &[Argument; 2]) -> Result<(), Error> {
        let function: fn(u64, u64) -> u64 = match command {
            Command::Add => Integer::wrapping_add,
            _ => return Err("Invalid calculate command".to_string()),
        };

        self.registers.data[0] = function(self.registers.data[0], args[0].resolve_or_error(self)?);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::btreemap;

    #[test]
    fn simple_add_test() {
        let mut rt = Runtime::new(btreemap! {
            0 => (Command::Move, [Argument::Register(0), Argument::Raw(123)]),
            1 => (Command::Move, [Argument::Register(1), Argument::Raw(321)]),
            2 => (Command::Add, [Argument::Register(1), Argument::None]),
        });

        rt.run().unwrap();

        assert_eq!(444, rt.output())
    }

    #[test]
    fn for_loop_test() {
        let start = 1254;
        let stop = 666;
        let mut rt = Runtime::new(btreemap! {
            0 => (Command::Label, [Argument::RawLabel(start), Argument::None]),
            1 => (Command::Add, [Argument::Raw(1), Argument::None]),
            2 => (Command::Compare, [Argument::Register(0), Argument::Raw(10)]),
            3 => (Command::BranchGreaterEqual, [Argument::RawLabel(stop), Argument::None]),
            4 => (Command::Branch, [Argument::RawLabel(start), Argument::None]),
            5 => (Command::Label, [Argument::RawLabel(stop), Argument::None]),
        });

        rt.run().unwrap();

        assert_eq!(10, rt.output())
    }
}
