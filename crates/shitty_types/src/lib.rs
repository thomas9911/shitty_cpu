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

impl Command {
    pub fn to_name(&self) -> String {
        match serde_value::to_value(self) {
            Ok(serde_value::Value::String(s)) => s,
            Ok(serde_value::Value::Map(map)) => {
                let mut s = String::new();
                for (_k, v) in map.iter() {
                    match v {
                        serde_value::Value::U64(n) => {
                            s.push_str(&n.to_string());
                            s.push_str(": ");
                            s.push_str("db")
                        }
                        _ => unreachable!(),
                    }
                }
                s
            }
            _ => {
                unreachable!()
            }
        }
    }
}

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
    pub fn format(&self) -> String {
        match self {
            Argument::None => "".to_string(),
            Argument::Raw(n) => format!("#{n}"),
            Argument::Register(r) => format!("r{r}"),
            Argument::HeapRef(h) => h.to_string(),
            Argument::Literal(l) if l.is_empty() => String::from(r#"db """#),
            Argument::Literal(l) => {
                let out: Option<String> = l
                    .iter()
                    .map(|b| (*b).try_into().ok())
                    .map(|c: Option<u32>| c.map(char::from_u32).flatten())
                    .collect();
                let value = if let Some(valid_string) = out {
                    format!("{:?}", valid_string)
                } else {
                    let data: Vec<_> = l.iter().map(|b| b.to_string()).collect();
                    data.join(",")
                };

                let mut return_value = String::from("db ");
                return_value.push_str(&value);
                return_value
            }
            Argument::HeapDeref(h, 0) => format!("[:{h}]"),
            Argument::HeapDeref(h, i) => format!("[:{h} + {i}]"),
            Argument::RawLabel(l) => format!(":{l}"),
        }
    }
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

pub fn format_program(program: &Program) -> String {
    let mut s = String::new();

    for (_i, (command, [arg0, arg1])) in program.iter() {
        match command {
            Command::Label => match arg0 {
                Argument::RawLabel(label) => s.push_str(format!("{label}:\n").as_str()),
                _ => unreachable!(),
            },
            Command::LabelledData(label) => {
                let mut formatted_line = String::new();

                formatted_line.push_str(format!("{label}: ").as_str());
                formatted_line.push_str(&arg0.format());
                formatted_line.push(' ');
                formatted_line.push_str(&arg1.format());

                s.push_str(&formatted_line.trim_end());
                s.push('\n');
            }
            _ => {
                let mut formatted_line = String::new();
                formatted_line.push_str("    ");
                formatted_line.push_str(&command.to_name());
                formatted_line.push(' ');
                formatted_line.push_str(&arg0.format());
                formatted_line.push(' ');
                formatted_line.push_str(&arg1.format());

                s.push_str(&formatted_line.trim_end());
                s.push('\n');
            }
        }
    }

    s
}

#[test]
fn test_command_to_name() {
    assert_eq!(Command::Add.to_name(), "add");
    assert_eq!(Command::Subtract.to_name(), "sub");
    assert_eq!(Command::LabelledData(8421).to_name(), "8421: db")
}

#[test]
fn test_argument_format() {
    assert_eq!(Argument::None.format(), "");
    assert_eq!(Argument::Raw(1234).format(), "#1234");
    assert_eq!(Argument::Register(5).format(), "r5");
    assert_eq!(Argument::RawLabel(123456).format(), ":123456");
    assert_eq!(Argument::HeapDeref(123456, 0).format(), "[:123456]");
    assert_eq!(Argument::HeapDeref(123456, 12).format(), "[:123456 + 12]");
    assert_eq!(
        Argument::Literal(vec![116, 101, 115, 116, 105, 110, 103]).format(),
        r#"db "testing""#
    );
    assert_eq!(
        Argument::Literal(vec![116, 101, 115, 116, 105, 110, 103, 9410051]).format(),
        "db 116,101,115,116,105,110,103,9410051"
    );
    assert_eq!(
        Argument::Literal(vec![1, 0, 1, 0, 1, 0]).format(),
        r#"db "\u{1}\0\u{1}\0\u{1}\0""#
    );
    assert_eq!(Argument::Literal(vec![]).format(), "db \"\"");
}

#[test]
fn test_program_format() {
    let data_str = 12529907765057034586;
    let program = maplit::btreemap! {
        1 => (Command::LabelledData(data_str), [Argument::Literal(vec![116, 101, 115, 116, 105, 110, 103]), Argument::None]),
        2 => (Command::Move, [Argument::Register(1), Argument::Raw(1)]),
        3 => (Command::Move, [Argument::Register(2), Argument::Raw(2)]),
        4 => (Command::Move, [Argument::Register(3), Argument::Raw(3)]),
        5 => (Command::Move, [Argument::HeapDeref(data_str, 0), Argument::Register(1)]),
        6 => (Command::Move, [Argument::HeapDeref(data_str, 1), Argument::Register(2)]),
        7 => (Command::Move, [Argument::HeapDeref(data_str, 2), Argument::Register(3)]),
        8 => (Command::Label, [Argument::RawLabel(2184574), Argument::None]),
    };

    let expected = r#"12529907765057034586: db "testing"
    mov r1 #1
    mov r2 #2
    mov r3 #3
    mov [:12529907765057034586] r1
    mov [:12529907765057034586 + 1] r2
    mov [:12529907765057034586 + 2] r3
2184574:
"#;

    assert_eq!(expected, format_program(&program));
}
