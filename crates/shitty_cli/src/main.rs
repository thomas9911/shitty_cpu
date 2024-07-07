use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use anyhow::{anyhow, Context};
use pico_args::Arguments;
use shitty_file_format::FileStructure;
use shitty_parser;
use std::process::ExitCode;

const HELP_MESSAGE: &str = r#"
    Usage: shitty_cli <subcommand>

    Subcommands:

    run [options] <program>
        options:
            -o, --open <file>
            --output-as-status-code : return the output as statuscode

    compile <input_file> <output_file>
    
    exec [options] <file>
        options:
            --output-as-status-code : return the output as statuscode
"#;

fn main() -> Result<ExitCode, anyhow::Error> {
    let mut args = Arguments::from_env();

    if args.contains(["-h", "--help"]) {
        return help();
    }

    match args.subcommand() {
        Ok(Some(x)) if x == "run" => run(&mut args),
        Ok(Some(x)) if x == "compile" => compile(&mut args),
        Ok(Some(x)) if x == "exec" => exec(&mut args),
        Ok(Some(x)) if x == "help" => help(),
        _ => {
            anyhow::bail!("Invalid command")
        }
    }
}

fn help() -> Result<ExitCode, anyhow::Error> {
    eprintln!("{}", HELP_MESSAGE);
    Ok(ExitCode::SUCCESS)
}

fn run(args: &mut Arguments) -> Result<ExitCode, anyhow::Error> {
    let file: Option<PathBuf> = args.opt_value_from_str(["-o", "--open"])?;
    let output_as_status_code = args.contains("--output-as-status-code");
    let program_text: Option<String> = args.opt_free_from_str()?;

    let program = match (file, program_text) {
        (Some(_), Some(_)) => return Err(anyhow!("Cannot specify both -o and a file")),
        (None, None) => return Err(anyhow!("Must specify either -o or a file")),
        (Some(path), None) => {
            let mut file = File::open(path)?;
            let input = BufReader::new(&mut file);
            shitty_parser::parse(input).map_err(|e| anyhow::anyhow!("{}", e))?
        }
        (None, Some(input)) => {
            shitty_parser::parse_from_str(&input).map_err(|e| anyhow::anyhow!("{}", e))?
        }
    };

    let mut rt = shitty_runtime::Runtime::new(program);
    rt.run().map_err(|e| anyhow::anyhow!("{}", e))?;

    if output_as_status_code {
        let status: u8 = rt.output().try_into().context("parsing status code")?;
        return Ok(ExitCode::from(status));
    }
    println!("{}", rt.output());

    Ok(ExitCode::SUCCESS)
}

fn compile(args: &mut Arguments) -> Result<ExitCode, anyhow::Error> {
    let input_path: PathBuf = args.free_from_str()?;
    let output_path: PathBuf = args.free_from_str()?;

    let mut in_file = File::open(input_path)?;
    let input = BufReader::new(&mut in_file);
    let program = shitty_parser::parse(input).map_err(|e| anyhow::anyhow!("{}", e))?;

    let file = FileStructure::new(program);
    file.to_path(output_path)
        .map_err(|e| anyhow::anyhow!("{}", e))?;

    Ok(ExitCode::SUCCESS)
}

fn exec(args: &mut Arguments) -> Result<ExitCode, anyhow::Error> {
    let output_as_status_code = args.contains("--output-as-status-code");
    let file_path: PathBuf = args.free_from_str()?;

    let file = FileStructure::from_path(file_path).map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut rt = shitty_runtime::Runtime::new(file.program);
    rt.run().map_err(|e| anyhow::anyhow!("{}", e))?;
    if output_as_status_code {
        let status: u8 = rt.output().try_into().context("parsing status code")?;
        return Ok(ExitCode::from(status));
    }
    println!("{}", rt.output());
    Ok(ExitCode::SUCCESS)
}

#[test]
fn run_from_text() {
    let mut args = Arguments::from_vec(vec!["mov r0 #24".into()]);
    assert!(run(&mut args).is_ok());

    let mut args = Arguments::from_vec(vec!["invalid r0 #24".into()]);
    assert!(run(&mut args).is_err());
}

#[test]
fn run_from_file() {
    use std::ffi::OsString;
    use std::io::Write;

    let dir = tempfile::tempdir().unwrap();

    let file_path = dir.path().join("my-temp.s");
    let mut file = File::create(&file_path).unwrap();
    writeln!(file, "mov r0 #94").unwrap();

    let mut args = Arguments::from_vec(vec!["-o".into(), OsString::from(&file_path)]);
    assert!(run(&mut args).is_ok());
    let mut args = Arguments::from_vec(vec!["--open".into(), OsString::from(file_path)]);
    assert!(run(&mut args).is_ok());
}
