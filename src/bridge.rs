// bridges rust to lua

use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use mlua::{Function, Lua, Table, Value};

use crate::storage::Store;

struct Extension {
    prefix: String,
    add: Option<Function>,
    get: Option<Function>,
}

pub struct ExtensionManager {
    #[allow(dead_code)]
    lua: Lua,
    extensions: Vec<Extension>,
}

impl ExtensionManager {
    // scans folder for .lua files and runs each one
    pub fn load(directory: &str, store: Rc<RefCell<Store>>) -> ExtensionManager {
        let lua = Lua::new();
        register_query_functions(&lua, store);

        let mut extensions = Vec::new();

        let entries = match fs::read_dir(directory) {
            Ok(entries) => entries,
            Err(_) => return ExtensionManager { lua, extensions },
        };

        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(_) => continue,
            };

            let path = entry.path();
            let is_lua_file = path.extension().and_then(|ext| ext.to_str()) == Some("lua");
            if !is_lua_file {
                continue;
            }

            let code = match fs::read_to_string(&path) {
                Ok(code) => code,
                Err(_) => continue,
            };

            let table: Table = match lua.load(&code).eval() {
                Ok(table) => table,
                Err(_) => continue,
            };

            let prefix: String = match table.get("prefix") {
                Ok(prefix) => prefix,
                Err(_) => continue,
            };
            let add: Option<Function> = table.get("add").unwrap_or(None);
            let get: Option<Function> = table.get("get").unwrap_or(None);

            extensions.push(Extension { prefix, add, get });
        }

        ExtensionManager { lua, extensions }
    }

    /*
    the next functions deal with extensions and prefixes.
    given a key, gets its prefix and verifies which lua
    extension applies to it, and then runs its validations:
    - ADD - checks if value is correct
    - GET - formats value output
    */
    fn find_extension(&self, key: &str) -> Option<&Extension> {
        for extension in &self.extensions {
            if key.starts_with(&extension.prefix) {
                return Some(extension);
            }
        }
        None
    }

    pub fn on_add(&self, key: &str, value: &str) -> Result<String, String> {
        let extension = match self.find_extension(key) {
            Some(extension) => extension,
            None => return Ok(value.to_string()),
        };
        let function = match &extension.add {
            Some(function) => function,
            None => return Ok(value.to_string()),
        };
        call_extension_function(function, key, value)
    }

    pub fn on_get(&self, key: &str, value: &str) -> Result<String, String> {
        let extension = match self.find_extension(key) {
            Some(extension) => extension,
            None => return Ok(value.to_string()),
        };
        let function = match &extension.get {
            Some(function) => function,
            None => return Ok(value.to_string()),
        };
        call_extension_function(function, key, value)
    }
}

fn call_extension_function(function: &Function, key: &str, value: &str) -> Result<String, String> {
    let result: mlua::Result<Value> = function.call((key.to_string(), value.to_string()));

    match result {
        Ok(Value::Nil) => Ok(value.to_string()),
        Ok(Value::String(text)) => Ok(text.to_string_lossy()),
        Ok(_) => Err("a extensão retornou um valor que não é texto".to_string()),
        Err(mlua::Error::RuntimeError(message)) => Err(strip_traceback(message)),
        Err(other) => Err(other.to_string()),
    }
}

// *AI: mlua appends a debug "stack traceback" after the original error message
// when a Lua `error(...)` escapes all the way to Rust. we only want to show
// the message the extension actually wrote, so everything from there on is
// cut off.
fn strip_traceback(message: String) -> String {
    match message.split_once("\nstack traceback:") {
        Some((original_message, _)) => original_message.to_string(),
        None => message,
    }
}

fn register_query_functions(lua: &Lua, store: Rc<RefCell<Store>>) {
    let store_for_read = Rc::clone(&store);
    let read_function = lua
        .create_function(move |_, key: String| {
            let database = store_for_read.borrow();
            Ok(database.get(&key).cloned())
        })
        .expect("failed to register the db_read function");
    lua.globals()
        .set("db_read", read_function)
        .expect("failed to expose db_read to Lua");

    let store_for_search = Rc::clone(&store);
    let search_function = lua
        .create_function(move |_, value: String| {
            let database = store_for_search.borrow();
            Ok(database.find_key_by_value(&value))
        })
        .expect("failed to register the db_find_key function");
    lua.globals()
        .set("db_find_key", search_function)
        .expect("failed to expose db_find_key to Lua");
}
