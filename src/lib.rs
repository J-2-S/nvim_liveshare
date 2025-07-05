mod local;
mod remote;
mod types;
use crate::{
    local::draft_change,
    remote::get_incoming_stream,
    types::{Config, Message},
};
use anyhow::Result;
use mlua::prelude::*;
use std::sync::OnceLock;
use tokio::{
    runtime::{Builder, Runtime},
    sync::{Mutex, mpsc, watch},
};
static RUNTIME: OnceLock<Runtime> = OnceLock::new();
static CWD: Mutex<Option<String>> = Mutex::const_new(None);
fn start_tokio_runtime() -> Result<()> {
    let _ = RUNTIME.set(
        Builder::new_multi_thread()
            .enable_all()
            .max_blocking_threads(0) //We don't need any all blocking code should
            //be run in lua
            .build()?,
    );
    Ok(())
}
fn get_runtime() -> &'static Runtime {
    RUNTIME.get().unwrap()
}

fn poll(lua: &Lua, _: ()) -> LuaResult<LuaValue> {
    let stream = get_incoming_stream().ok_or(LuaError::external("Failed to pool dead session"))?;
    match stream.blocking_lock().try_recv() {
        Err(error) => {
            if matches!(error, mpsc::error::TryRecvError::Empty) {
                return Ok(LuaValue::Nil);
            } else {
                return Err(LuaError::external(error));
            }
        }
        Ok(value) => Ok(lua.to_value(&value)?),
    }
}
fn share(lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    let config: Config = lua.from_value(lua.globals().get("__LIVE_SHARE__")?)?;
    let controler = lua.create_table()?;
    controler.set("draft_changes", lua.create_function(draft_change)?)?;
    let (tx, rx) = watch::channel(false);
    controler.set(
        "stop",
        lua.create_function(move |_, _: ()| tx.send(true).map_err(LuaError::external))?,
    )?;
    controler.set("poll", lua.create_function(poll)?)?;
    get_runtime().spawn(remote::start_server(config, rx));
    Ok(controler)
}
fn connect(lua: &Lua, _: ()) -> LuaResult<LuaTable> {
    todo!()
}
#[mlua::lua_module]
fn rust_client(lua: &Lua) -> LuaResult<LuaTable> {
    start_tokio_runtime().map_err(LuaError::external)?; //Start the tokio runtime we need for async
    let table = lua.create_table()?; // We create the module table
    let config = lua.to_value(&Config::default())?;
    table.set("config", config)?;
    table.set("share", lua.create_function(share)?)?;
    table.set("connect", lua.create_function(connect)?)?;
    lua.globals().set("__LIVE_SHARE__", table)?;
    let cwd: String = lua.load("vim.fn.getcwd()").eval()?;
    *CWD.blocking_lock() = Some(cwd);
    lua.globals().get("__LIVE_SHARE__")
}
