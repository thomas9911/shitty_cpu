use std::{fs::File, io::BufReader, path::PathBuf};

use pico_args::Arguments;
use shitty_parser;
use shitty_types::Integer;

fn main() -> Result<(), anyhow::Error> {
    let mut args = Arguments::from_env();

    match args.subcommand() {
        Ok(Some(x)) if x == "run" => run(&mut args)?,
        _ => println!("Invalid command"),
    };

    Ok(())
}

fn run(args: &mut Arguments) -> Result<(), anyhow::Error> {
    let file: PathBuf = args.free_from_str()?;
    let mut file = File::open(file)?;
    let input = BufReader::new(&mut file);
    let program = shitty_parser::parse(input).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut rt = shitty_runtime::Runtime::new(program);
    rt.run().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("{}", rt.output());

    Ok(())
}
