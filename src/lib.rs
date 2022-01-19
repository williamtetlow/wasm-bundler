mod bundler;
mod filesystem;
mod log;
mod utils;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use bundler::Bundler;
use filesystem::InMemFileSystem;
use swc_common::FileName;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub struct WasmBundler {
    fs: Arc<Mutex<InMemFileSystem>>,
    entries: HashMap<String, FileName>,
    bundler: Bundler,
}

#[wasm_bindgen]
impl WasmBundler {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        utils::set_panic_hook();

        let fs = Arc::new(Mutex::new(InMemFileSystem::new()));

        WasmBundler {
            entries: HashMap::new(),
            bundler: Bundler::new(Arc::clone(&fs)),
            fs,
        }
    }

    #[wasm_bindgen(js_name = "addEntry")]
    pub fn add_entry(&mut self, name: &str, file_path: &str) {
        self.entries
            .insert(name.into(), FileName::Real(file_path.into()));
    }

    #[wasm_bindgen(js_name = "saveFile")]
    pub fn save_file(&mut self, filename: &str, content: &str) {
        self.fs.lock().unwrap().save(filename, content);
    }

    #[wasm_bindgen(js_name = "bundle")]
    pub fn bundle(&mut self) -> String {
        match self.bundler.bundle(HashMap::clone(&self.entries)) {
            Ok(result) => result,
            Err(err) => err.to_string(),
        }
    }
}
