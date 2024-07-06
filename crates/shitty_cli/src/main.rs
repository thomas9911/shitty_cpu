use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use pico_args::Arguments;
use shitty_file_format::FileStructure;
use shitty_parser;

fn main() -> Result<(), anyhow::Error> {
    let mut args = Arguments::from_env();

    match args.subcommand() {
        Ok(Some(x)) if x == "run" => run(&mut args)?,
        Ok(Some(x)) if x == "compile" => compile(&mut args)?,
        Ok(Some(x)) if x == "exec" => exec(&mut args)?,
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

fn compile(args: &mut Arguments) -> Result<(), anyhow::Error> {
    let input_path: PathBuf = args.free_from_str()?;
    let output_path: PathBuf = args.free_from_str()?;

    let mut in_file = File::open(input_path)?;
    let input = BufReader::new(&mut in_file);
    let program = shitty_parser::parse(input).map_err(|e| anyhow::anyhow!("{}", e))?;

    let file = FileStructure::new(program);
    file.save(output_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(())
}

fn exec(args: &mut Arguments) -> Result<(), anyhow::Error> {
    let file_path: PathBuf = args.free_from_str()?;

    let file = FileStructure::load(file_path).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut rt = shitty_runtime::Runtime::new(file.program);
    rt.run().map_err(|e| anyhow::anyhow!("{}", e))?;
    println!("{}", rt.output());
    Ok(())
}
