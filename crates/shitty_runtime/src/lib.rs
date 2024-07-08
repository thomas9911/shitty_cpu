use shitty_types::{Argument, Command, Error, Heap, Integer, Program};
use std::cmp::Ordering;
use std::collections::BTreeMap;

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

#[derive(Debug, Clone)]
pub struct Runtime {
    flags: Flags,
    registers: Registers,
    program_counter: Integer,
    program: Program,
    heap: Heap,
    stack: Vec<Integer>,
    label_references: BTreeMap<Integer, Integer>,
    debug: bool,
}

impl Runtime {
    pub fn new(program: Program) -> Self {
        Runtime {
            flags: Flags::default(),
            registers: Registers::new(),
            heap: Heap::new(),
            stack: Vec::new(),
            program_counter: 0,
            label_references: Self::scan_labels(&program),
            program,
            debug: false,
        }
    }

    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
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
        let Some((last_line, _)) = self.program.last_key_value() else {
            return Ok(());
        };
        let end = *last_line + 1;

        loop {
            match self.tick(end) {
                Ok(true) => break,
                Ok(false) => (),
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    pub fn tick(&mut self, end: Integer) -> Result<bool, Error> {
        if self.program_counter >= end {
            return Ok(true);
        }

        if let Some((command, args)) = self
            .program
            .get(&self.program_counter)
            .map(|(c, [a1, a2])| (c.clone(), [a1.clone(), a2.clone()]))
        {
            self.apply_command(&command, &args)?;
            if self.debug {
                self.print_registers();
            }
        } else {
            self.program_counter += 1;
        };

        Ok(false)
    }

    pub fn apply_command(&mut self, command: &Command, args: &[Argument; 2]) -> Result<(), Error> {
        let mut increase_program_counter = true;
        match command {
            Command::Noop => (),
            Command::Move => {
                let new_value = self.resolve_argument_or_error(&args[1])?;

                match args[0] {
                    Argument::Register(reg) => self.registers.data[reg as usize] = new_value,
                    Argument::HeapRef(heap_id) => {
                        // self.heap
                        //     .entry(heap_id)
                        //     .and_modify(|p| *p = new_value)
                        //     .or_insert(new_value);
                        todo!("move heap");
                    }
                    _ => return Err("Invalid argument".to_string()),
                }
            }
            Command::Label => {}
            Command::Branch => {
                self.brancher(args)?;
            }
            Command::BranchEqual => {
                if self.flags.equal {
                    self.brancher(args)?;
                }
            }
            Command::BranchNotEqual => {
                if !self.flags.equal {
                    self.brancher(args)?;
                }
            }
            Command::BranchGreater => {
                if self.flags.greater {
                    self.brancher(args)?;
                }
            }
            Command::BranchGreaterEqual => {
                if self.flags.equal || self.flags.greater {
                    self.brancher(args)?;
                }
            }
            Command::BranchLesser => {
                if self.flags.less {
                    self.brancher(args)?;
                }
            }
            Command::BranchLesserEqual => {
                if self.flags.equal || self.flags.less {
                    self.brancher(args)?;
                }
            }
            Command::Compare => {
                let value_a = self.resolve_argument_or_error(&args[0])?;
                let value_b = self.resolve_argument_or_error(&args[1])?;

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
            Command::Add | Command::Subtract | Command::Multiply | Command::Divide => {
                self.calculate(command, args)?;
            }
            Command::Push => {
                let value = self.resolve_argument_or_error(&args[0])?;
                self.stack.push(value);
            }
            Command::Pop => {
                if let Some(value) = self.stack.pop() {
                    if let Some(pointer) = self.resolve_argument_mut(&args[0]) {
                        *pointer = value;
                    }
                }
            }
            Command::Call => {
                self.stack.push(self.program_counter);
                self.brancher(args)?;
            }
            Command::Return => {
                if let Some(value) = self.stack.pop() {
                    self.program_counter = value;
                } else {
                    // error or ignore?
                    // return Err("Stack underflow".to_string());
                }
            }
            Command::LabelledData(label) => {
                let Argument::Literal(value) = &args[0] else {
                    return Err(String::from("argument can only be literal"));
                };

                self.label_references
                    .insert(*label, self.heap.len() as Integer);
                self.heap.push(value.clone());
            }
        }

        self.program_counter += increase_program_counter as Integer;

        Ok(())
    }

    pub fn output(&self) -> Integer {
        self.registers.data[0]
    }

    fn resolve_argument(&self, argument: &Argument) -> Option<Integer> {
        match argument {
            Argument::None => None,
            Argument::Raw(data) => Some(*data),
            Argument::Register(reg_id) => self.registers.data.get(*reg_id as usize).copied(),
            Argument::HeapRef(ref_id) => Some(*self.label_references.get(&ref_id).unwrap()),
            Argument::RawLabel(label) => self.label_references.get(label).copied(),
            Argument::HeapDeref(label, offset) => {
                let ref_id = self.label_references.get(label).copied();
                ref_id
                    .map(|x| {
                        self.heap
                            .get(x as usize)
                            .map(|y| y.get(*offset as usize).copied())
                    })
                    .flatten()
                    .flatten()
            }
            Argument::Literal(_) => todo!(),
        }
    }

    fn resolve_argument_mut(&mut self, argument: &Argument) -> Option<&mut Integer> {
        match argument {
            Argument::None => None,
            Argument::Raw(_data) => None,
            Argument::Register(reg_id) => self.registers.data.get_mut(*reg_id as usize),
            Argument::HeapRef(ref_id) => self.label_references.get_mut(&ref_id),
            Argument::RawLabel(label) => self.label_references.get_mut(label),
            Argument::HeapDeref(label, offset) => {
                let ref_id = self.label_references.get(label).copied();
                ref_id
                    .map(|x| {
                        self.heap
                            .get_mut(x as usize)
                            .map(|y| y.get_mut(*offset as usize))
                    })
                    .flatten()
                    .flatten()
            }
            Argument::Literal(_) => todo!(),
        }
    }

    pub fn resolve_argument_or_error(&self, argument: &Argument) -> Result<Integer, Error> {
        self.resolve_argument(argument)
            .ok_or_else(|| String::from("no valid argument"))
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
        let function: fn(u64, u64) -> (u64, bool) = match command {
            Command::Add => Integer::overflowing_add,
            Command::Subtract => Integer::overflowing_sub,
            Command::Multiply => Integer::overflowing_mul,
            Command::Divide => Integer::overflowing_div,
            // Command::Shiftleft => Integer::overflowing_shl,
            // Command::Shiftright => Integer::overflowing_shr,
            _ => return Err("Invalid calculate command".to_string()),
        };

        let (out, overflow) = function(
            self.resolve_argument_or_error(&args[0])?,
            self.resolve_argument_or_error(&args[1])?,
        );
        // self.registers.data[0] = out;
        if let Some(pointer) = self.resolve_argument_mut(&args[0]) {
            *pointer = out;
        }
        self.flags.overflow = overflow;

        Ok(())
    }

    fn print_registers(&self) {
        print!("{} => ", self.program_counter);
        for (index, register) in self.registers.data.iter().enumerate() {
            if register != &0 {
                print!("r{}: {}|", index, register);
            }
        }
        println!();
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
            2 => (Command::Add, [Argument::Register(0), Argument::Register(1)]),
        });

        rt.run().unwrap();

        assert_eq!(444, rt.output())
    }

    #[test]
    fn empty_lines_test() {
        let mut rt = Runtime::new(btreemap! {
            1 => (Command::Move, [Argument::Register(0), Argument::Raw(123)]),
            3 => (Command::Move, [Argument::Register(1), Argument::Raw(321)]),
            7 => (Command::Add, [Argument::Register(0), Argument::Register(1)]),
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
            1 => (Command::Add, [Argument::Register(0), Argument::Raw(1)]),
            2 => (Command::Compare, [Argument::Register(0), Argument::Raw(10)]),
            3 => (Command::BranchGreaterEqual, [Argument::RawLabel(stop), Argument::None]),
            4 => (Command::Branch, [Argument::RawLabel(start), Argument::None]),
            5 => (Command::Label, [Argument::RawLabel(stop), Argument::None]),
        });

        rt.run().unwrap();

        assert_eq!(10, rt.output())
    }

    #[test]
    fn if_statement_test() {
        let condition_a = 8411;
        let stop = 8419;

        let prepared_rt = Runtime::new(btreemap! {
            0 => (Command::Compare, [Argument::Register(0), Argument::Raw(10)]),
            1 => (Command::BranchGreater, [Argument::RawLabel(condition_a), Argument::None]),
            2 => (Command::Multiply, [Argument::Register(0), Argument::Raw(5)]),
            3 => (Command::Branch, [Argument::RawLabel(stop), Argument::None]),
            4 => (Command::Label, [Argument::RawLabel(condition_a), Argument::None]),
            5 => (Command::Subtract, [Argument::Register(0), Argument::Raw(10)]),
            6 => (Command::Label, [Argument::RawLabel(stop), Argument::None]),
        });

        let mut rt = prepared_rt.clone();
        rt.registers.data[0] = 12;
        rt.run().unwrap();
        assert_eq!(2, rt.output());

        let mut rt = prepared_rt.clone();
        rt.registers.data[0] = 8;
        rt.run().unwrap();
        assert_eq!(40, rt.output())
    }

    #[test]
    fn push_pop_test() {
        let mut rt = Runtime::new(btreemap! {
            0 => (Command::Move, [Argument::Register(0), Argument::Raw(15)]),
            1 => (Command::Push, [Argument::Register(0), Argument::None]),
            2 => (Command::Move, [Argument::Register(0), Argument::Raw(11)]),
            3 => (Command::Push, [Argument::Register(0), Argument::None]),
            4 => (Command::Move, [Argument::Register(0), Argument::Raw(9)]),
            5 => (Command::Push, [Argument::Register(0), Argument::None]),
            6 => (Command::Pop, [Argument::Register(2), Argument::None]),
            7 => (Command::Pop, [Argument::Register(1), Argument::None]),
        });

        rt.run().unwrap();

        assert_eq!(rt.registers.data[2], 9);
        assert_eq!(rt.registers.data[1], 11);
        assert_eq!(rt.stack, vec![15])
    }

    #[test]
    fn call_return_test() {
        let add_one = 8411;
        let end = 18427;

        let mut rt = Runtime::new(btreemap! {
            //   mov r0 15
            0 => (Command::Move, [Argument::Register(0), Argument::Raw(15)]),
            //   call :add_one
            1 => (Command::Call, [Argument::RawLabel(add_one), Argument::None]),
            //   mul r0 7
            2 => (Command::Multiply, [Argument::Register(0), Argument::Raw(7)]),
            //   b :end
            3 => (Command::Branch, [Argument::RawLabel(end), Argument::None]),
            // add_one:
            4 => (Command::Label, [Argument::RawLabel(add_one), Argument::None]),
            //   add r0 100
            5 => (Command::Add, [Argument::Register(0), Argument::Raw(100)]),
            //   ret
            6 => (Command::Return, [Argument::None, Argument::None]),
            // end:
            7 => (Command::Label, [Argument::RawLabel(end), Argument::None]),
        });

        rt.run().unwrap();

        assert_eq!(805, rt.output());
    }

    #[test]
    fn string_literal() {
        let data_str = 12529907765057034586;
        let data_str2 = 12529904465057034586;

        let mut rt = Runtime::new(maplit::btreemap! {
            1 => (Command::LabelledData(data_str), [Argument::Literal("Hallo\0".chars().map(|x| x as Integer).collect()), Argument::None]),
            2 => (Command::LabelledData(data_str2), [Argument::Literal("Test\0".chars().map(|x| x as Integer).collect()), Argument::None]),
            3 => (Command::Move, [Argument::Register(1), Argument::RawLabel(data_str)]),
            4 => (Command::Move, [Argument::Register(0), Argument::RawLabel(data_str2)]),
            5 => (Command::Move, [Argument::Register(2), Argument::HeapDeref(data_str, 0)]),
            6 => (Command::Move, [Argument::Register(3), Argument::HeapDeref(data_str, 1)]),
        });

        rt.run().unwrap();

        assert_eq!(1, rt.registers.data[0]);
        assert_eq!(0, rt.registers.data[1]);
        assert_eq!(b'H' as Integer, rt.registers.data[2]);
        assert_eq!(b'a' as Integer, rt.registers.data[3]);
    }
}
