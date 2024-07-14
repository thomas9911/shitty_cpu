use std::collections::HashMap;

use from_pest::FromPest;
use pest::Parser;
use pest_derive::Parser;
use shitty_types::{Argument, Command};

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

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::script))]
    pub struct Script {
        pub program: Program,
        pub _eoi: EOI,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::program))]
    pub struct Program {
        pub lines: Vec<Line>,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::line))]
    pub enum Line {
        FunctionCall(FunctionCall),
        FunctionDef(FunctionDef),
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::function_call))]
    pub struct FunctionCall {
        pub ident: Ident,
        pub params: Option<FunctionArguments>,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::inner_call_function_arguments))]
    pub struct FunctionArguments {
        pub arguments: Vec<Expression>,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::function_def))]
    pub struct FunctionDef {
        pub ident: Ident,
        pub params: Option<Arguments>,
        pub body: Block,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::ident))]
    pub struct Ident {
        #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
        pub ident: String,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::inner_function_arguments))]
    pub struct Arguments {
        pub args: Vec<Ident>,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::block))]
    pub struct Block {
        pub statements: Vec<Statement>,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::inner_statement))]
    pub enum Statement {
        Return(ReturnStatement),
        Expression(Expression),
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::expression))]
    pub enum Expression {
        Term(Term),
        FunctionCall(FunctionCall),
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::return_statement))]
    pub struct ReturnStatement {
        pub ident: Term,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::term))]
    pub enum Term {
        Ident(Ident),
        Number(Integer),
        String(StringLiteral),
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::integer))]
    pub struct Integer {
        #[pest_ast(outer(with(span_into_str), with(str::parse), with(Result::unwrap)))]
        pub value: u64,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::inner_string))]
    pub struct StringLiteral {
        #[pest_ast(outer(with(span_into_str), with(str::to_string)))]
        pub value: String,
    }

    #[derive(Debug, FromPest, PartialEq)]
    #[pest_ast(rule(Rule::EOI))]
    pub struct EOI {}
}

pub fn parse(input: &str) -> Result<ast::Script, anyhow::Error> {
    let mut parse_tree = ScriptParser::parse(Rule::script, input)?;
    let syntax_tree = Script::from_pest(&mut parse_tree)?;
    Ok(syntax_tree)
}

pub fn script_to_program(script: &Script) -> Result<shitty_types::Program, anyhow::Error> {
    let mut commands: Vec<(Command, [Argument; 2])> = Vec::new();

    let mut defined_functions = HashMap::new();


    for line in script.program.lines.iter() {
        match line {
            Line::FunctionDef(x) => {
                defined_functions.insert(x.ident.ident.as_str(), x);
            }
            _ => {

            }
        }
    }

    Ok(commands.into_iter().enumerate().map(|(index, command)| (index as shitty_types::Integer, command)).collect())
}

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
                            ident: Term::Number(Integer { value: 1234 }),
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
                            ident: Term::String(StringLiteral {
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
                            ident: Term::Ident(Ident {
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
    let script = Script {
        program: Program {
            lines: vec![
                Line::FunctionDef(FunctionDef {
                    ident: Ident {
                        ident: "foo".to_string(),
                    },
                    params: None,
                    body: Block {
                        statements: vec![Statement::Return(ReturnStatement {
                            ident: Term::Number(Integer { value: 1234 }),
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
                            ident: Term::String(StringLiteral {
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
                            ident: Term::Ident(Ident {
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

    script_to_program(&script).unwrap();

    panic!()
}