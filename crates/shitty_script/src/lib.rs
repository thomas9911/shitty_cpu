mod ast;
mod convert;
mod parser;

pub use crate::ast::Script;
pub use convert::script_to_program;
pub use parser::parse;

#[cfg(test)]
use pretty_assertions::assert_eq;

#[cfg(test)]
use crate::ast::{
    Arguments, Block, Expression, FunctionArguments, FunctionCall, FunctionDef, Ident, Integer,
    Line, Program, ReturnStatement, Statement, StringLiteral, Term, EOI,
};

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
