use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::hash::{DefaultHasher, Hash, Hasher};

pub type Error = String;

pub type Integer = u64;
pub type RawCommand = u64;
pub type RawArgument = u64;

pub type RawProgram = Vec<(Integer, RawCommand, RawArgument, RawArgument)>;
pub type Heap = Vec<Literal>;
pub type Stack = Vec<Integer>;
pub type Program = BTreeMap<Integer, (Command, [Argument; 2])>;
pub type Literal = Vec<Integer>;

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    #[serde(rename = "_")]
    Noop,
    #[serde(rename = "lbl")]
    Label,
    #[serde(rename = "ld")]
    LabelledData(Integer),
    #[serde(rename = "b")]
    Branch,
    #[serde(rename = "be")]
    BranchEqual,
    #[serde(rename = "bne")]
    BranchNotEqual,
    #[serde(rename = "bge")]
    BranchGreaterEqual,
    #[serde(rename = "bg")]
    BranchGreater,
    #[serde(rename = "bl")]
    BranchLesser,
    #[serde(rename = "ble")]
    BranchLesserEqual,
    #[serde(rename = "cmp")]
    Compare,
    #[serde(rename = "mov")]
    Move,
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "sub")]
    Subtract,
    #[serde(rename = "mul")]
    Multiply,
    #[serde(rename = "div")]
    Divide,
    #[serde(rename = "mod")]
    Modulo,
    Push,
    Pop,
    Call,
    #[serde(rename = "func")]
    Function,
    #[serde(rename = "ret")]
    Return,
}

impl Command {}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Argument {
    #[serde(rename = "_")]
    None,
    Raw(Integer),
    #[serde(rename = "reg")]
    Register(u8),
    HeapRef(Integer),
    #[serde(rename = "rlbl")]
    RawLabel(Integer),
    #[serde(rename = "lit")]
    Literal(Literal),
    HeapDeref(Integer, usize),
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

pub fn hash_label(label: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    label.hash(&mut hasher);
    let hash = hasher.finish();
    hash
}
