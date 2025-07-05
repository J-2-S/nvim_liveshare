use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub hostname: String,
    pub port: u16,
    // Add more options as needed
}
impl Default for Config {
    fn default() -> Self {
        Self {
            hostname: "0.0.0.0".into(),
            port: 6969,
        }
    }
}
#[derive(Serialize, Deserialize, PartialEq, PartialOrd, Clone, Debug)]
pub struct Position {
    pub row: u32,
    pub col: u32,
}
pub const SOF: Position = Position {
    row: u32::MIN,
    col: u32::MIN,
};
pub const EOF: Position = Position {
    row: u32::MAX,
    col: u32::MAX,
};
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Change {
    pub content: String,
    pub start: Position,
    pub end: Position,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Method {
    Push,
    Exit,
    CreateFile,
    CreateDir,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub method: Method,
    pub file: String,
    pub changes: Vec<Change>,
}
