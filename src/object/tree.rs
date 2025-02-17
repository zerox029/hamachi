use std::cmp::PartialEq;
use std::fmt::{Display};
use std::str::FromStr;
use crate::object::{Hash, ObjectType};

pub(crate) struct Tree {
    size: usize,
    entries: Vec<Entry>,
}

#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) mode: Mode,
    pub(crate) filename: String,
    pub(crate) object_type: ObjectType,
    pub(crate) hash: Hash,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:0>6} {} {}\t{}", self.mode as u32, self.object_type, &self.hash.to_string(), self.filename)
    }
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Debug)]
pub(crate) enum Mode {
    REGULAR = 100644,
    EXECUTABLE = 100755,
    SYMBOLIC = 120000,
    DIRECTORY = 40000,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "100644" => Ok(Mode::REGULAR),
            "100755" => Ok(Mode::EXECUTABLE),
            "120000" => Ok(Mode::SYMBOLIC),
            "40000" => Ok(Mode::DIRECTORY),
            _ => Err(()),
        }
    }
}

