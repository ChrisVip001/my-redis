use bincode;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    String(String),
    Char(Vec<char>),
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Command {
    Add,
    Delete,
}

#[derive(Serialize, Deserialize)]
pub struct Entry {
    meta: Meta, 
    key: String, 
    value: Value,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    command: Command,
    key_size: u64,
    value_size: u64,
}

const USIZE_SIZE: usize = std::mem::size_of::<usize>();
const ENTRY_META_SIZE: usize = USIZE_SIZE * 2 + 4;

impl Entry {
    pub fn size(&self) -> usize {
        ENTRY_META_SIZE + self.meta.key_size + self.meta.value_size
    }
}