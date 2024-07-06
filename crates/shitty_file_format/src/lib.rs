use std::fs::{read, write};
use std::path::Path;

use serde::{Deserialize, Serialize};
use shitty_types::Program;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FileStructure {
    pub version: usize,
    pub program: Program,
}

impl FileStructure {
    pub fn new(program: Program) -> Self {
        FileStructure {
            version: 0,
            program,
        }
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        ciborium::into_writer(&self, &mut buffer)?;
        write(path, buffer)?;
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<FileStructure, Box<dyn std::error::Error>> {
        let data = read(path)?;
        let value = ciborium::from_reader(data.as_slice())?;
        Ok(value)
    }
}

#[test]
fn save_and_load() {
    use shitty_types::{Argument, Command};

    let program = maplit::btreemap! {
        1 => (Command::Move, [Argument::Register(0), Argument::Raw(7)]),
        2 => (Command::Move, [Argument::Register(1), Argument::Raw(2)]),
        3 => (Command::Add, [Argument::Register(0), Argument::Register(1)]),
    };
    let path = "tmp.bin";

    let file = FileStructure::new(program);
    file.save(path).unwrap();
    let file2 = FileStructure::load(path).unwrap();
    // std::fs::remove_file(path).unwrap();

    assert_eq!(file, file2);
}
