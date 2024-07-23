use crate::parser::Rule;
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
