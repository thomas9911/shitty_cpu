use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io::{BufRead, BufReader, Cursor},
};

use shitty_types::{Argument, Command, Error, Program};

pub fn parse_from_str(input: &str) -> Result<Program, Error> {
    let cursor = Cursor::new(input);
    parse(BufReader::new(cursor))
}

pub fn parse(input: impl BufRead) -> Result<Program, Error> {
    let mut program = Program::default();

    let mut index = 0;
    for line in input.lines() {
        let line = line.map_err(|e| e.to_string())?;
        if let Some((command, raw_args)) = line.trim().split_once(' ') {
            let command = match command {
                "mov" => Command::Move,
                "add" => Command::Add,
                "sub" => Command::Subtract,
                "mul" => Command::Multiply,
                "div" => Command::Divide,
                "bgr" => Command::BranchGreater,
                "b" => Command::Branch,
                "cmp" => Command::Compare,
                _ => return Err(format!("unknown command {}", command)),
            };
            let mut args = [Argument::None, Argument::None];
            let mut arg_index = 0;
            for raw_arg in raw_args.split(' ').filter(|part| !part.is_empty()) {
                if arg_index >= 2 {
                    return Err(format!("too many arguments for command {:?}", command));
                }

                args[arg_index] = match raw_arg {
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
                    x if x.starts_with("#") => {
                        let Some((_, rest)) = x.split_once('#') else {
                            return Err(format!(
                                "Invalid amount of # for literal input {:?}",
                                command
                            ));
                        };

                        Argument::Raw(rest.parse().unwrap())
                    }
                    x if x.split_once(':').is_some() => {
                        let (_, label) = x.split_once(':').unwrap();
                        Argument::RawLabel(hash_label(label))
                    }
                    _ => return Err(format!("unknown argument {}", raw_arg)),
                };
                arg_index += 1;
            }
            program.insert(index, (command, args));
        } else {
            let line = line.trim().trim_end_matches(':');
            if !line.is_empty() {
                program.insert(
                    index,
                    (
                        Command::Label,
                        [Argument::RawLabel(hash_label(line)), Argument::None],
                    ),
                );
            }
        }
        index += 1;
    }

    Ok(program)
}

fn hash_label(label: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    label.hash(&mut hasher);
    let hash = hasher.finish();
    hash
}

#[test]
fn parse_simple_program() {
    let input = r#"
    mov r0 #7
    mov r1 #2
    add r1
    "#;

    let program = parse_from_str(input).unwrap();

    assert_eq!(
        program,
        maplit::btreemap! {
            1 => (Command::Move, [Argument::Register(0), Argument::Raw(7)]),
            2 => (Command::Move, [Argument::Register(1), Argument::Raw(2)]),
            3 => (Command::Add, [Argument::Register(1), Argument::None]),
        }
    );
}

#[test]
fn parse_program_with_labels() {
    let input = r#"cmp r0 #10
    bgr :condition_a
    mul #5
    b :stop
condition_a:
    sub #10
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
            2 => (Command::Multiply, [Argument::Raw(5), Argument::None]),
            3 => (Command::Branch, [Argument::RawLabel(stop), Argument::None]),
            4 => (Command::Label, [Argument::RawLabel(condition_a), Argument::None]),
            5 => (Command::Subtract, [Argument::Raw(10), Argument::None]),
            6 => (Command::Label, [Argument::RawLabel(stop), Argument::None]),
        }
    );
}
