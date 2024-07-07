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

    pub fn dump(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = Vec::new();
        ciborium::into_writer(&self, &mut buffer)?;
        Ok(buffer)
    }

    pub fn to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let data = self.dump()?;
        write(path, data)?;
        Ok(())
    }

    pub fn load(data: &[u8]) -> Result<FileStructure, Box<dyn std::error::Error>> {
        Ok(ciborium::from_reader(data)?)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<FileStructure, Box<dyn std::error::Error>> {
        let data = read(path)?;
        Self::load(data.as_slice())
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

    let file = FileStructure::new(program);
    let data = file.dump().unwrap();
    let file2 = FileStructure::load(&data).unwrap();

    assert_eq!(file, file2);
}

#[test]
fn from_to_path() {
    use shitty_types::{Argument, Command};

    let program = maplit::btreemap! {
        1 => (Command::Move, [Argument::Register(0), Argument::Raw(7)]),
        2 => (Command::Move, [Argument::Register(1), Argument::Raw(2)]),
        3 => (Command::Add, [Argument::Register(0), Argument::Register(1)]),
    };
    let path = "tmp.bin";

    let file = FileStructure::new(program);
    file.to_path(path).unwrap();
    let file2 = FileStructure::from_path(path).unwrap();
    std::fs::remove_file(path).unwrap();

    assert_eq!(file, file2);
}
