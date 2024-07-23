use from_pest::FromPest;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "script.pest"]
struct ScriptParser;

use crate::Script;

pub fn parse(input: &str) -> Result<Script, anyhow::Error> {
    let mut parse_tree = ScriptParser::parse(Rule::script, input)?;
    let syntax_tree = Script::from_pest(&mut parse_tree)?;
    Ok(syntax_tree)
}
