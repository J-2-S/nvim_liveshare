use crate::types::*;
use anyhow::Result;
use mlua::prelude::*;
use std::io::Read;
use std::sync::Arc;
use tokio::sync::{Mutex, OnceCell};
static BROADCASTER: OnceCell<Arc<Mutex<tokio::sync::broadcast::Sender<Message>>>> =
    OnceCell::const_new();

pub async fn get_subscriber() -> tokio::sync::broadcast::Receiver<Message> {
    let broadcast = BROADCASTER
        .get_or_init(async || Arc::new(Mutex::new(tokio::sync::broadcast::channel(10).0)))
        .await
        .lock()
        .await;
    broadcast.subscribe()
}

fn file_diff(original: String, new: String) -> Change {
    let original_lines: Vec<&str> = original.lines().collect();
    let new_lines: Vec<&str> = new.lines().collect();

    let mut start_line = 0;
    let mut start_col = 0;
    let mut end_col = original_lines.last().map_or(0, |line| line.len());

    // Find the first line that differs
    for i in 0..original_lines.len().min(new_lines.len()) {
        if original_lines[i] != new_lines[i] {
            start_line = i;
            start_col = original_lines[i]
                .chars()
                .zip(new_lines[i].chars())
                .take_while(|(a, b)| a == b)
                .count() as u32;
            break;
        }
    }

    // Find the last line that matches (from the end)
    let mut orig_idx = original_lines.len();
    let mut new_idx = new_lines.len();
    while orig_idx > start_line && new_idx > start_line {
        if original_lines[orig_idx - 1] != new_lines[new_idx - 1] {
            break;
        }
        orig_idx -= 1;
        new_idx -= 1;
    }

    // Compute ending line and column in original text
    let end_line = orig_idx;
    end_col = if end_line == 0 {
        0
    } else {
        original_lines[end_line - 1].len()
    };

    // Reconstruct the content slice that replaces the old text
    let replacement = new_lines[start_line..new_idx].join("\n");

    Change {
        content: replacement,
        start: Position {
            row: start_line as u32,
            col: start_col,
        },
        end: Position {
            row: end_line as u32,
            col: end_col as u32,
        },
    }
}

pub fn draft_change(_: &mlua::Lua, (buffer_content, file_path): (String, String)) -> LuaResult<()> {
    //this function has to be sync it runs outside the runtime
    let mut file_content = Vec::new();
    std::fs::File::open(&file_path)?.read_to_end(&mut file_content)?;
    let file_content = String::from_utf8(file_content).map_err(mlua::Error::external)?;
    let change = file_diff(file_content, buffer_content);
    let mut changes = Vec::new();
    changes.push(change);

    let message = Message {
        method: Method::Push,
        file: file_path,
        changes,
    };
    BROADCASTER
        .get()
        .ok_or(LuaError::external("BROADCASTER was not set"))?
        .blocking_lock()
        .send(message)
        .map_err(mlua::Error::external)?;
    Ok(())
}

pub fn create_dir(_: &Lua, dir: String) -> LuaResult<()> {
    let message: Message = Message {
        method: Method::CreateDir,
        file: dir,
        changes: Vec::new(),
    };
    BROADCASTER
        .get()
        .ok_or(LuaError::external("BROADCASTER was not set"))?
        .blocking_lock()
        .send(message)
        .map_err(mlua::Error::external)?;
    Ok(())
}

pub fn create_file(_: &Lua, (file_name, dir_path): (String, String)) -> LuaResult<()> {
    let message: Message = Message {
        method: Method::CreateFile,
        file: dir_path + &file_name,
        changes: Vec::new(),
    };
    BROADCASTER
        .get()
        .ok_or(LuaError::external("BROADCASTER was not set"))?
        .blocking_lock()
        .send(message)
        .map_err(mlua::Error::external)?;
    Ok(())
}
