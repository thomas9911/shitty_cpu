use std::collections::BTreeMap;

pub type Error = String;

pub type Integer = u64;
pub type RawCommand = u64;
pub type RawArgument = u64;

pub type RawProgram = Vec<(Integer, RawCommand, RawArgument, RawArgument)>;
pub type Heap = BTreeMap<Integer, Integer>;
pub type Program = BTreeMap<Integer, (Command, Argument, Argument)>;

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Noop,
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

impl Argument {
    pub fn resolve(&self, rt: &Runtime) -> Option<Integer> {
        match self {
            Argument::None => None,
            Argument::Raw(data) => Some(*data),
            Argument::Register(reg_id) => rt.registers.data.get(*reg_id as usize).copied(),
            Argument::HeapRef(ref_id) => Some(*rt.heap.get(&ref_id).unwrap()),
        }
    }

    pub fn resolve_or_error(&self, rt: &Runtime) -> Result<Integer, Error> {
        self.resolve(rt)
            .ok_or_else(|| String::from("no valid argument"))
    }
}

#[derive(Debug, Clone)]
pub struct Runtime {
    registers: Registers,
    program_counter: Integer,
    program: BTreeMap<Integer, (Command, Argument, Argument)>,
    heap: Heap,
}

impl Runtime {
    pub fn new(program: Program) -> Self {
        Runtime {
            registers: Registers::new(),
            heap: Heap::new(),
            program_counter: 0,
            program,
        }
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
        if let Some((command, arg1, arg2)) = self
            .program
            .get(&self.program_counter)
            .map(|(c, a1, a2)| (*c, *a1, *a2))
        {
            self.apply_command(&command, &arg1, &arg2)?;
            return Ok(false);
        };

        Ok(true)
    }

    pub fn apply_command(
        &mut self,
        command: &Command,
        arg1: &Argument,
        arg2: &Argument,
    ) -> Result<(), Error> {
        let mut increase_program_counter = true;
        match command {
            Command::Noop => (),
            Command::Move => {
                let new_value = arg2.resolve_or_error(self)?;

                match arg1 {
                    Argument::Register(reg) => self.registers.data[*reg as usize] = new_value,
                    Argument::HeapRef(heap_id) => {
                        self.heap
                            .entry(*heap_id)
                            .and_modify(|p| *p = new_value)
                            .or_insert(new_value);
                    }
                    _ => return Err("Invalid argument".to_string()),
                }
            }
            Command::Add => {
                self.registers.data[0] =
                    self.registers.data[0].wrapping_add(arg1.resolve_or_error(self)?);
            }
        }

        self.program_counter += increase_program_counter as Integer;

        Ok(())
    }

    pub fn output(&self) -> Integer {
        self.registers.data[0]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maplit::btreemap;

    #[test]
    fn simple_add_test() {
        let mut rt = Runtime::new(btreemap! {
            0 => (Command::Move, Argument::Register(0), Argument::Raw(123)),
            1 => (Command::Move, Argument::Register(1), Argument::Raw(321)),
            2 => (Command::Add, Argument::Register(1), Argument::None),
        });

        rt.run().unwrap();

        assert_eq!(444, rt.output())
    }
}
