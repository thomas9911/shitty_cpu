use indexmap::IndexMap;

use from_pest::FromPest;
use pest::Parser;
use pest_derive::Parser;
use shitty_types::{hash_label, Argument, Command};

use crate::ast::PointerRef;
pub use crate::ast::{
    Arguments, Block, Expression, FunctionArguments, FunctionCall, FunctionDef, Ident, Integer,
    Line, Program, ReturnStatement, Script, Statement, StringLiteral, Term, EOI,
};

#[derive(Parser)]
#[grammar = "script.pest"]
struct ScriptParser;

mod ast {
    use super::Rule;
    use pest::Span;
    use pest_ast::FromPest;

    fn span_into_str(span: Span) -> &str {
        span.as_str()
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::script))]
    pub struct Script {
        pub program: Program,
        pub _eoi: EOI,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::program))]
    pub struct Program {
        pub lines: Vec<Line>,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::line))]
    pub enum Line {
        FunctionCall(FunctionCall),
        FunctionDef(FunctionDef),
        Term(Term),
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::function_call))]
    pub struct FunctionCall {
        pub ident: Ident,
        pub params: Option<FunctionArguments>,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::inner_call_function_arguments))]
    pub struct FunctionArguments {
        pub arguments: Vec<Expression>,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::function_def))]
    pub struct FunctionDef {
        pub ident: Ident,
        pub params: Option<Arguments>,
        pub body: Block,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::ident))]
    pub struct Ident {
        #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
        pub ident: String,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::inner_function_arguments))]
    pub struct Arguments {
        pub args: Vec<Ident>,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::block))]
    pub struct Block {
        pub statements: Vec<Statement>,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::inner_statement))]
    pub enum Statement {
        Return(ReturnStatement),
        Expression(Expression),
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::expression))]
    pub enum Expression {
        Term(Term),
        FunctionCall(FunctionCall),
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::return_statement))]
    pub struct ReturnStatement {
        pub item: Term,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::term))]
    pub enum Term {
        Ident(Ident),
        Number(Integer),
        String(StringLiteral),
        // pointer ref is never parsed from the file
        PointerRef(PointerRef),
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::term))]
    pub struct PointerRef {
        #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
        pub ident: String,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::integer))]
    pub struct Integer {
        #[pest_ast(outer(with(span_into_str), with(str::parse), with(Result::unwrap)))]
        pub value: u64,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::inner_string))]
    pub struct StringLiteral {
        #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
        pub value: String,
    }

    #[derive(Debug, FromPest, PartialEq, Clone)]
    #[pest_ast(rule(Rule::EOI))]
    pub struct EOI {}

    impl Block {
        pub fn add_heap_values<'a>(&'a mut self, list: &mut Vec<&'a mut Term>) {
            for item in self.statements.iter_mut() {
                item.add_heap_values(list);
            }
        }
    }

    impl Statement {
        pub fn add_heap_values<'a>(&'a mut self, list: &mut Vec<&'a mut Term>) {
            match self {
                Statement::Return(ReturnStatement { item }) => item.add_heap_values(list),
                Statement::Expression(ex) => ex.add_heap_values(list),
            }
        }
    }

    impl Expression {
        pub fn add_heap_values<'a>(&'a mut self, list: &mut Vec<&'a mut Term>) {
            match self {
                Expression::Term(term) => term.add_heap_values(list),
                Expression::FunctionCall(function_call) => function_call.add_heap_values(list),
            }
        }
    }

    impl Term {
        pub fn add_heap_values<'a>(&'a mut self, list: &mut Vec<&'a mut Term>) {
            match self {
                Term::String(_) => list.push(self),
                _ => {}
            }
        }
    }

    impl FunctionCall {
        pub fn add_heap_values<'a>(&'a mut self, list: &mut Vec<&'a mut Term>) {
            if let Some(x) = &mut self.params {
                x.add_heap_values(list);
            }
        }
    }

    impl FunctionArguments {
        pub fn add_heap_values<'a>(&'a mut self, list: &mut Vec<&'a mut Term>) {
            for item in self.arguments.iter_mut() {
                item.add_heap_values(list);
            }
        }
    }
}

pub fn parse(input: &str) -> Result<ast::Script, anyhow::Error> {
    let mut parse_tree = ScriptParser::parse(Rule::script, input)?;
    let syntax_tree = Script::from_pest(&mut parse_tree)?;
    Ok(syntax_tree)
}

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

#[cfg(test)]
use pretty_assertions::assert_eq;

#[test]
fn parse_simple() {
    let input = r#"
    
fn foo() {
    return 1234;
}

fn bar() {
    return "hallo";
}

fn echo(i) {
    return i;
}

echo(1234)
echo(echo(98765))
    "#;

    //    let mut parse_tree = ScriptParser::parse(Rule::script, input).unwrap();
    //    let syntax_tree = Script::from_pest(&mut parse_tree).unwrap();

    let syntax_tree = parse(input).unwrap();

    let expected = Script {
        program: Program {
            lines: vec![
                Line::FunctionDef(FunctionDef {
                    ident: Ident {
                        ident: "foo".to_string(),
                    },
                    params: None,
                    body: Block {
                        statements: vec![Statement::Return(ReturnStatement {
                            item: Term::Number(Integer { value: 1234 }),
                        })],
                    },
                }),
                Line::FunctionDef(FunctionDef {
                    ident: Ident {
                        ident: "bar".to_string(),
                    },
                    params: None,
                    body: Block {
                        statements: vec![Statement::Return(ReturnStatement {
                            item: Term::String(StringLiteral {
                                value: "hallo".to_string(),
                            }),
                        })],
                    },
                }),
                Line::FunctionDef(FunctionDef {
                    ident: Ident {
                        ident: "echo".to_string(),
                    },
                    params: Some(Arguments {
                        args: vec![Ident {
                            ident: "i".to_string(),
                        }],
                    }),
                    body: Block {
                        statements: vec![Statement::Return(ReturnStatement {
                            item: Term::Ident(Ident {
                                ident: "i".to_string(),
                            }),
                        })],
                    },
                }),
                Line::FunctionCall(FunctionCall {
                    ident: Ident {
                        ident: "echo".to_string(),
                    },
                    params: Some(FunctionArguments {
                        arguments: vec![Expression::Term(Term::Number(Integer { value: 1234 }))],
                    }),
                }),
                Line::FunctionCall(FunctionCall {
                    ident: Ident {
                        ident: "echo".to_string(),
                    },
                    params: Some(FunctionArguments {
                        arguments: vec![Expression::FunctionCall(FunctionCall {
                            ident: Ident {
                                ident: "echo".to_string(),
                            },
                            params: Some(FunctionArguments {
                                arguments: vec![Expression::Term(Term::Number(Integer {
                                    value: 98765,
                                }))],
                            }),
                        })],
                    }),
                }),
            ],
        },
        _eoi: EOI {},
    };

    assert_eq!(syntax_tree, expected);
}

#[test]
fn convert_script_to_commands() {
    let mut script = Script {
        program: Program {
            lines: vec![
                Line::FunctionDef(FunctionDef {
                    ident: Ident {
                        ident: "foo".to_string(),
                    },
                    params: None,
                    body: Block {
                        statements: vec![Statement::Return(ReturnStatement {
                            item: Term::Number(Integer { value: 1234 }),
                        })],
                    },
                }),
                Line::FunctionDef(FunctionDef {
                    ident: Ident {
                        ident: "bar".to_string(),
                    },
                    params: None,
                    body: Block {
                        statements: vec![Statement::Return(ReturnStatement {
                            item: Term::String(StringLiteral {
                                value: "hallo".to_string(),
                            }),
                        })],
                    },
                }),
                Line::FunctionDef(FunctionDef {
                    ident: Ident {
                        ident: "echo".to_string(),
                    },
                    params: Some(Arguments {
                        args: vec![Ident {
                            ident: "i".to_string(),
                        }],
                    }),
                    body: Block {
                        statements: vec![Statement::Return(ReturnStatement {
                            item: Term::Ident(Ident {
                                ident: "i".to_string(),
                            }),
                        })],
                    },
                }),
                Line::FunctionCall(FunctionCall {
                    ident: Ident {
                        ident: "echo".to_string(),
                    },
                    params: Some(FunctionArguments {
                        arguments: vec![Expression::Term(Term::Number(Integer { value: 1234 }))],
                    }),
                }),
                // Line::FunctionCall(FunctionCall {
                //     ident: Ident {
                //         ident: "echo".to_string(),
                //     },
                //     params: Some(FunctionArguments {
                //         arguments: vec![Expression::FunctionCall(FunctionCall {
                //             ident: Ident {
                //                 ident: "echo".to_string(),
                //             },
                //             params: Some(FunctionArguments {
                //                 arguments: vec![Expression::Term(Term::Number(Integer {
                //                     value: 98765,
                //                 }))],
                //             }),
                //         })],
                //     }),
                // }),
            ],
        },
        _eoi: EOI {},
    };

    let program = script_to_program(&mut script).unwrap();

    println!("{}", shitty_types::format_program(&program));

    let expected_program = r#"b :.main
bar_0: db "hallo"
foo:
    push #1234
    ret
bar:
    push :bar_0
    ret
echo:
    pop r1
    push r1
    ret
.main:
    push #1234
    call :echo
    pop r0
    "#;

    let expected_program = shitty_parser::parse_from_str(expected_program).unwrap();

    // dbg!(program);

    // panic!()
    assert_eq!(expected_program, program);
}
