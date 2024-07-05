use std::collections::BTreeMap;

pub type Error = String;

pub type Integer = u64;
pub type RawCommand = u64;
pub type RawArgument = u64;

pub type RawProgram = Vec<(Integer, RawCommand, RawArgument, RawArgument)>;
pub type Heap = Vec<Vec<Integer>>;
pub type Program = BTreeMap<Integer, (Command, [Argument; 2])>;

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Noop,
    Label,
    LabelledData(Integer),
    Branch,
    BranchEqual,
    BranchNotEqual,
    BranchGreaterEqual,
    BranchGreater,
    BranchLesser,
    BranchLesserEqual,
    Compare,
    Move,
    Add,
    Subtract,
    Multiply,
    Divide,
    Push,
    Pop,
    Call,
    Return,
}

impl Command {}

#[derive(Debug, Clone, PartialEq)]
pub enum Argument {
    None,
    Raw(Integer),
    Register(u8),
    HeapRef(Integer),
    RawLabel(Integer),
    Literal(Vec<Integer>),
    HeapDeref(Integer, Integer),
}

impl Argument {
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
