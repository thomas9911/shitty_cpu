use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::{BufRead, BufReader, Cursor};

use winnow::ascii::{alpha1, dec_uint, space0};
use winnow::combinator::{alt, fail, preceded, terminated};
use winnow::error::{ContextError, StrContext};
use winnow::prelude::*;
use winnow::stream::AsChar;
use winnow::token::{take_till, take_while};

use shitty_types::{Argument, Command, Error, Integer, Program};

pub fn parse_from_str(input: &str) -> Result<Program, Error> {
    let cursor = Cursor::new(input);
    parse(BufReader::new(cursor))
}

pub fn parse(input: impl BufRead) -> Result<Program, Error> {
    let mut program = Program::default();

    for (index, line) in input.lines().enumerate() {
        let line = line.map_err(|e| e.to_string())?;
        let mut line_str = line.trim();
        if line_str.is_empty() {
            continue;
        }

        let command =
            if let Some((remainder, label)) = label_line_parser.parse_peek(&mut line_str).ok() {
                line_str = remainder;
                Command::LabelledData(hash_label(label))
            } else {
                line_str = line_str.trim();
                parse_command
                    .parse_next(&mut line_str)
                    .map_err(|e| e.to_string())?
            };
        line_str = line_str.trim();
        let mut args = [Argument::None, Argument::None];

        if line_str.is_empty() {
            match command {
                Command::LabelledData(label) => {
                    args[0] = Argument::RawLabel(label);
                    program.insert(index as Integer, (Command::Label, args));
                    continue;
                }
                Command::Return => {
                    program.insert(index as Integer, (Command::Return, args));
                    continue;
                }
                _ => return Err(format!("missing arguments for command: {command:?}")),
            };
        }

        let arg0 = parse_argument
            .parse_next(&mut line_str)
            .map_err(|e| e.to_string())?;
        args[0] = arg0;

        line_str = line_str.trim();
        if line_str.is_empty() {
            program.insert(index as Integer, (command, args));
        } else {
            let arg1 = parse_argument
                .parse_next(&mut line_str)
                .map_err(|e| e.to_string())?;
            args[1] = arg1;
            program.insert(index as Integer, (command, args));
        }
    }

    Ok(program)
}

fn generic_error(input: &mut &str, label: &'static str) -> PResult<()> {
    fail.context(StrContext::Label(label)).parse_next(input)
}

fn label_line_parser<'s>(input: &mut &'s str) -> PResult<&'s str> {
    terminated(take_till(1.., |c: char| [':', ' '].contains(&c)), ":").parse_next(input)
}

fn parse_command<'s>(input: &mut &'s str) -> PResult<Command> {
    let command = match alpha1
        .context(StrContext::Label("parse command"))
        .parse_next(input)?
    {
        "mov" => Command::Move,
        "add" => Command::Add,
        "sub" => Command::Subtract,
        "mul" => Command::Multiply,
        "div" => Command::Divide,
        "bgr" => Command::BranchGreater,
        "bge" => Command::BranchGreaterEqual,
        "bl" => Command::BranchLesser,
        "ble" => Command::BranchLesserEqual,
        "beq" => Command::BranchEqual,
        "bne" => Command::BranchNotEqual,
        "b" => Command::Branch,
        "cmp" => Command::Compare,
        "call" => Command::Call,
        "push" => Command::Push,
        "pop" => Command::Pop,
        "ret" => Command::Return,
        _ => return Err(generic_error(input, "invalid command").unwrap_err()),
    };
    Ok(command)
}

fn parse_argument<'s>(input: &mut &'s str) -> PResult<Argument> {
    let argument = match alt((
        ('[', take_while(1.., |c| c != ']') , ']').recognize(),
        take_while(1.., |c| !AsChar::is_space(c)),
    ))
    .context(StrContext::Label("parse argument"))
    .parse_next(input)?
    {
        "r0" => Argument::Register(0),
        "r1" => Argument::Register(1),
        "r2" => Argument::Register(2),
        "r3" => Argument::Register(3),
        "r4" => Argument::Register(4),
        "r5" => Argument::Register(5),
        "r6" => Argument::Register(6),
        "r7" => Argument::Register(7),
        "r8" => Argument::Register(8),
        "r9" => Argument::Register(9),
        "r10" => Argument::Register(10),
        "r11" => Argument::Register(11),
        "r12" => Argument::Register(12),
        "r13" => Argument::Register(13),
        "r14" => Argument::Register(14),
        "r15" => Argument::Register(15),
        "db" => {
            *input = input.trim();
            let arg = Argument::Literal(parse_db_literal(input)?);
            *input = "";
            arg
        }
        mut x if x.starts_with("#") => {
            preceded("#", dec_uint).map(|int| Argument::Raw(int)).parse_next(&mut x)?
        }
        mut x if x.contains(':') => {
            alt((
                winnow::seq!(_: (space0::<&str, ContextError>, '[', space0, ':'), take_while(1.., |c| !AsChar::is_space(c) && c != '+'), _: (space0, '+', space0), dec_uint, _: (space0, ']', space0)).map(|(label, offset): (&str, usize)| Argument::HeapDeref(hash_label(label), offset)),
                winnow::seq!(_: (space0::<&str, ContextError>, '[', space0, ':'), take_while(1.., |c| !AsChar::is_space(c) && c != ']'), _: (space0, ']', space0)).map(|(label, )| Argument::HeapDeref(hash_label(label), 0)),
                winnow::seq!(_: ':', take_while(1.., |c| !AsChar::is_space(c))).map(|(label, )| Argument::RawLabel(hash_label(label))),
                fail::<&str, _, ContextError>.context(StrContext::Label("invalid label argument")).map(|_: ()| Argument::None)
            )
            ).parse_next(&mut x)?
        }
        _ => {
            return Err(generic_error(input, "invalid argument").unwrap_err());
        }
    };
    Ok(argument)
}

fn hash_label(label: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    label.hash(&mut hasher);
    let hash = hasher.finish();
    hash
}

fn parse_db_literal(input: &mut &str) -> PResult<Vec<Integer>> {
    let mut output = Vec::new();
    for item in input.split(',') {
        let data: tinyjson::JsonValue = item
            .trim()
            .parse()
            .map_err(|e: tinyjson::JsonParseError| e.to_string())
            .map_err(|_| generic_error(input, "invalid db literal").unwrap_err())?;
        match data {
            tinyjson::JsonValue::String(x) => output.extend(x.chars().map(|x| x as Integer)),
            tinyjson::JsonValue::Number(x) => output.push(x as Integer),
            _ => return Err(generic_error(input, "invalid db literal").unwrap_err()),
        }
    }

    Ok(output)
}

#[test]
fn parse_simple_program() {
    let input = r#"
    mov r0 #7
    mov r1 #2
    add r0 r1
    "#;

    let program = parse_from_str(input).unwrap();

    assert_eq!(
        program,
        maplit::btreemap! {
            1 => (Command::Move, [Argument::Register(0), Argument::Raw(7)]),
            2 => (Command::Move, [Argument::Register(1), Argument::Raw(2)]),
            3 => (Command::Add, [Argument::Register(0), Argument::Register(1)]),
        }
    );
}

#[test]
fn parse_program_with_labels() {
    let input = r#"cmp r0 #10
    bgr :condition_a
    mul r0 #5
    b :stop
condition_a:
    sub r0 #10
stop:
    "#;

    let program = parse_from_str(input).unwrap();
    let condition_a = 8002582286646448037;
    let stop = 15597100844808768705;

    assert_eq!(
        program,
        maplit::btreemap! {
            0 => (Command::Compare, [Argument::Register(0), Argument::Raw(10)]),
            1 => (Command::BranchGreater, [Argument::RawLabel(condition_a), Argument::None]),
            2 => (Command::Multiply, [Argument::Register(0), Argument::Raw(5)]),
            3 => (Command::Branch, [Argument::RawLabel(stop), Argument::None]),
            4 => (Command::Label, [Argument::RawLabel(condition_a), Argument::None]),
            5 => (Command::Subtract, [Argument::Register(0), Argument::Raw(10)]),
            6 => (Command::Label, [Argument::RawLabel(stop), Argument::None]),
        }
    );
}

#[test]
fn parse_program_with_calls() {
    let input = r#"mov r0 #15
    call :add_one
    mul r0 #7
    b :end
add_one:
    add r0 #100
    ret
end:
    "#;

    let program = parse_from_str(input).unwrap();
    let add_one = 15338766068606827303;
    let end = 1666831356574994304;

    assert_eq!(
        program,
        maplit::btreemap! {
            0 => (Command::Move, [Argument::Register(0), Argument::Raw(15)]),
            1 => (Command::Call, [Argument::RawLabel(add_one), Argument::None]),
            2 => (Command::Multiply, [Argument::Register(0), Argument::Raw(7)]),
            3 => (Command::Branch, [Argument::RawLabel(end), Argument::None]),
            4 => (Command::Label, [Argument::RawLabel(add_one), Argument::None]),
            5 => (Command::Add, [Argument::Register(0), Argument::Raw(100)]),
            6 => (Command::Return, [Argument::None, Argument::None]),
            7 => (Command::Label, [Argument::RawLabel(end), Argument::None]),
        }
    );
}

#[test]
fn parse_program_with_string() {
    let input = r#"
data_str: db "Hallo",0,98
    mov r0 :data_str
    mov r1 [:data_str]
    mov r2 [:data_str+1]
    mov r3 [ :data_str + 2 ]
    "#;

    let program = parse_from_str(input).unwrap();
    let data_str = 12529907765057034586;

    assert_eq!(
        program,
        maplit::btreemap! {
            1 => (Command::LabelledData(data_str), [Argument::Literal("Hallo\0b".chars().map(|x| x as Integer).collect()), Argument::None]),
            2 => (Command::Move, [Argument::Register(0), Argument::RawLabel(data_str)]),
            3 => (Command::Move, [Argument::Register(1), Argument::HeapDeref(data_str, 0)]),
            4 => (Command::Move, [Argument::Register(2), Argument::HeapDeref(data_str, 1)]),
            5 => (Command::Move, [Argument::Register(3), Argument::HeapDeref(data_str, 2)]),
        }
    );
}
