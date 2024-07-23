use indexmap::IndexMap;

use shitty_types::{hash_label, Argument, Command};

use crate::ast::{
    Expression, FunctionCall, Ident, Integer, Line, PointerRef, ReturnStatement, Script, Statement,
    StringLiteral, Term,
};

pub fn script_to_program(script: &mut Script) -> Result<shitty_types::Program, anyhow::Error> {
    let mut commands: Vec<(Command, [Argument; 2])> = Vec::new();

    let mut defined_functions = IndexMap::new();
    let mut hoisted_static_values = IndexMap::new();
    let mut other_lines = Vec::new();

    commands.push((
        Command::Branch,
        [Argument::RawLabel(hash_label(".main")), Argument::None],
    ));

    for line in script.program.lines.iter_mut() {
        match line {
            Line::FunctionDef(x) => {
                {
                    let mut heap_values = Vec::new();
                    x.body.add_heap_values(&mut heap_values);
                    for (index, value) in heap_values.into_iter().enumerate() {
                        let mut key = x.ident.ident.clone();
                        key.push_str("_");
                        key.push_str(&index.to_string());

                        hoisted_static_values.insert(key.clone(), value.clone());
                        *value = Term::PointerRef(PointerRef { ident: key });
                    }
                }

                let key = x.ident.clone().ident;
                defined_functions.insert(key, x);
            }
            other => other_lines.push(other),
        }
    }

    for (key, static_values) in hoisted_static_values {
        let encoded_value = match static_values {
            Term::String(StringLiteral { value }) => {
                value.chars().map(|x| x as shitty_types::Integer).collect()
            }
            _ => continue,
        };

        commands.push((
            Command::LabelledData(hash_label(&key)),
            [Argument::Literal(encoded_value), Argument::None],
        ));
    }

    for (function_key, function) in defined_functions {
        commands.push((
            Command::Label,
            [
                Argument::RawLabel(hash_label(&function_key)),
                Argument::None,
            ],
        ));

        let mut args_registers = IndexMap::new();

        if let Some(ref args) = function.params {
            for (index, argument) in args.args.iter().enumerate() {
                let reg_index = index as u8 + 1;
                args_registers.insert(argument.ident.as_str(), reg_index);
                commands.push((
                    Command::Pop,
                    [Argument::Register(reg_index), Argument::None],
                ));
            }
        }

        for statement in function.body.statements.iter() {
            match statement {
                Statement::Expression(expr) => {
                    encode_expression(expr, &mut commands, &args_registers)?;
                }
                Statement::Return(ReturnStatement {
                    item: Term::Ident(Ident { ident }),
                }) => {
                    let reg_index = args_registers
                        .get(&ident.as_str())
                        .ok_or_else(|| anyhow::anyhow!("variable not found"))?;
                    commands.push((
                        Command::Push,
                        [Argument::Register(*reg_index), Argument::None],
                    ));
                }

                Statement::Return(ReturnStatement { item }) => match item {
                    Term::Number(Integer { value }) => {
                        commands.push((Command::Push, [Argument::Raw(*value), Argument::None]));
                    }
                    Term::PointerRef(PointerRef { ident }) => {
                        commands.push((
                            Command::Push,
                            [Argument::RawLabel(hash_label(ident)), Argument::None],
                        ));
                    }
                    _ => unreachable!(),
                },
            }

            commands.push((Command::Return, [Argument::None, Argument::None]))
        }
    }
    commands.push((
        Command::Label,
        [Argument::RawLabel(hash_label(".main")), Argument::None],
    ));

    for line in other_lines {
        match line {
            Line::FunctionDef(_) => unreachable!(),
            Line::FunctionCall(function_call) => {
                encode_function_call(function_call, &mut commands, &IndexMap::new())?;
            }
            Line::Term(_) => {}
        }
    }

    // dbg!(hoisted_static_values);
    // dbg!(&script.program);
    commands.push((Command::Pop, [Argument::Register(0), Argument::None]));

    Ok(commands
        .into_iter()
        .enumerate()
        .map(|(index, command)| (index as shitty_types::Integer, command))
        .collect())
}

fn encode_expression(
    expr: &Expression,
    commands: &mut Vec<(Command, [Argument; 2])>,
    args_registers: &IndexMap<&str, u8>,
) -> Result<(), anyhow::Error> {
    match expr {
        Expression::Term(_) => {}
        Expression::FunctionCall(function_call) => {
            // if let Some(ref args) = params {
            //     for argument in args.arguments.iter() {
            //         // let reg_index = index as u8 + 1;
            //         // args_registers.insert(argument.ident.as_str(), reg_index);
            //         // commands.push((Command::Push, [Argument::Register(reg_index), Argument::None]));
            //         match argument {
            //             Expression::Term(term) => {
            //                 match term {
            //                     Term::Number(Integer{value}) => {
            //                         commands.push((Command::Push, [Argument::Raw(*value), Argument::None]));
            //                     }
            //                     Term::PointerRef(PointerRef{ident}) => {
            //                         commands.push((Command::Push, [Argument::RawLabel(hash_label(ident)), Argument::None]));
            //                     }
            //                     Term::Ident(Ident{ident}) => {
            //                         let reg_index = args_registers.get(&ident.as_str()).ok_or_else(|| anyhow::anyhow!("variable not found"))?;
            //                         commands.push((Command::Push, [Argument::Register(*reg_index), Argument::None]));
            //                     }
            //                     Term::String(_) => unreachable!()
            //                 }
            //             }
            //             Expression::FunctionCall(_) => todo!()
            //         }
            //     }
            // }
            // commands.push((Command::Call, [Argument::RawLabel(hash_label(ident)), Argument::None]));
            encode_function_call(function_call, commands, args_registers)?;
        }
    }

    Ok(())
}

fn encode_function_call(
    FunctionCall {
        ident: Ident { ident },
        params,
    }: &FunctionCall,
    commands: &mut Vec<(Command, [Argument; 2])>,
    args_registers: &IndexMap<&str, u8>,
) -> Result<(), anyhow::Error> {
    if let Some(ref args) = params {
        for argument in args.arguments.iter() {
            // let reg_index = index as u8 + 1;
            // args_registers.insert(argument.ident.as_str(), reg_index);
            // commands.push((Command::Push, [Argument::Register(reg_index), Argument::None]));
            match argument {
                Expression::Term(term) => match term {
                    Term::Number(Integer { value }) => {
                        commands.push((Command::Push, [Argument::Raw(*value), Argument::None]));
                    }
                    Term::PointerRef(PointerRef { ident }) => {
                        commands.push((
                            Command::Push,
                            [Argument::RawLabel(hash_label(ident)), Argument::None],
                        ));
                    }
                    Term::Ident(Ident { ident }) => {
                        let reg_index = args_registers
                            .get(&ident.as_str())
                            .ok_or_else(|| anyhow::anyhow!("variable not found"))?;
                        commands.push((
                            Command::Push,
                            [Argument::Register(*reg_index), Argument::None],
                        ));
                    }
                    Term::String(_) => unreachable!(),
                },
                Expression::FunctionCall(_) => todo!(),
            }
        }
    }
    commands.push((
        Command::Call,
        [Argument::RawLabel(hash_label(ident)), Argument::None],
    ));
    Ok(())
}
