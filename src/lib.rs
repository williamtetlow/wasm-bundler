mod utils;
use anyhow::{bail, Error};
use std::collections::HashMap;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use std::{io, rc};
use swc_atoms::js_word;
use swc_bundler::{Bundler, Hook, Load, ModuleData, ModuleRecord, Resolve};
use swc_common::{sync::Lrc, FileLoader, FileName, FilePathMapping, Globals, SourceMap, Span};
use swc_ecma_ast::*;
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};

use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

extern crate web_sys;

// A macro to provide `println!(..)`-style syntax for `console.log` logging.
macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
pub struct WasmBundler {
    files: HashMap<String, String>,
    entrypoint: String,
}

#[wasm_bindgen]
impl WasmBundler {
    pub fn new(entrypoint: String) -> Self {
        WasmBundler {
            files: HashMap::new(),
            entrypoint,
        }
    }

    pub fn add_file(&mut self, filename: String, content: String) {
        self.files.insert(filename, content);
    }

    pub fn update_file(&mut self, filename: String, content: String) {
        self.files.remove(&filename);
        self.files.insert(filename, content);
    }

    pub fn bundle(&self) -> String {
        let globals = Box::leak(Box::new(Globals::default()));

        let source_map = SourceMap::with_file_loader(
            Box::new(InMemFileLoader::from(self.files.clone())),
            FilePathMapping::empty(),
        );

        let cm = Lrc::new(source_map);

        let mut bundler = Bundler::new(
            globals,
            cm.clone(),
            Loader { cm: cm.clone() },
            PathResolver,
            swc_bundler::Config {
                require: true,
                external_modules: vec![],
                ..Default::default()
            },
            Box::new(Noop),
        );

        let mut entries = HashMap::default();

        entries.insert(
            "main".to_string(),
            FileName::Real(self.entrypoint.clone().into()),
        );

        let mut bundles = bundler.bundle(entries).expect("failed to bundle");

        let bundle = bundles.pop().unwrap();

        let mut buf = vec![];

        let mut emitter = Emitter {
            cfg: swc_ecma_codegen::Config {
                ..Default::default()
            },
            cm: cm.clone(),
            comments: None,
            wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
        };

        emitter.emit_module(&bundle.module).unwrap();

        String::from_utf8_lossy(&buf).to_string()
    }
}

pub struct InMemFileLoader {
    files: HashMap<String, String>,
}

impl InMemFileLoader {
    pub fn new() -> InMemFileLoader {
        InMemFileLoader {
            files: HashMap::from([
                (
                    String::from("main.js"),
                    String::from(
                        "import { A, FOO } from './a';

                console.log(A, FOO);",
                    ),
                ),
                (
                    String::from("./a.js"),
                    String::from(
                        "export const FOO = 1;


                export class A {
                    foo() {
                    }
                }",
                    ),
                ),
            ]),
        }
    }

    pub fn from(files: HashMap<String, String>) -> InMemFileLoader {
        InMemFileLoader { files }
    }
}

impl FileLoader for InMemFileLoader {
    fn file_exists(&self, path: &Path) -> bool {
        self.files.contains_key(
            path.file_name()
                .expect("no filename in path")
                .to_str()
                .expect("to parse os str"),
        )
    }

    fn abs_path(&self, path: &Path) -> Option<PathBuf> {
        if path.is_absolute() {
            Some(path.to_path_buf())
        } else {
            Some(PathBuf::from("/"))
        }
    }

    fn read_file(&self, path: &Path) -> io::Result<String> {
        // need to work out how to convert option to
        log!("{}", path.as_os_str().to_str().expect("bla"));

        match self
            .files
            .get::<str>(path.as_os_str().to_str().expect("bla"))
        {
            Some(value) => io::Result::Ok(value.to_string()),
            None => io::Result::Err(std::io::Error::new(std::io::ErrorKind::NotFound, "oops")),
        }
    }
}

pub struct Loader {
    pub cm: Lrc<SourceMap>,
}

impl Load for Loader {
    fn load(&self, f: &FileName) -> Result<ModuleData, Error> {
        let fm = match f {
            FileName::Real(path) => self.cm.load_file(path)?,
            _ => unreachable!(),
        };

        let lexer = Lexer::new(
            Syntax::Es(EsConfig {
                ..Default::default()
            }),
            EsVersion::Es2021,
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        let module = parser
            .parse_module()
            .unwrap_or_else(|_| panic!("failed to parse"));

        Ok(ModuleData {
            fm,
            module,
            helpers: Default::default(),
        })
    }
}

struct PathResolver;

impl Resolve for PathResolver {
    fn resolve(&self, base: &FileName, module_specifier: &str) -> Result<FileName, Error> {
        assert!(
            module_specifier.starts_with('.'),
            "We are not using node_modules within this example"
        );

        let base = match base {
            FileName::Real(v) => v,
            _ => unreachable!(),
        };

        Ok(FileName::Real(
            base.parent()
                .unwrap()
                .join(module_specifier)
                .with_extension("js"),
        ))
    }
}

struct Noop;

impl Hook for Noop {
    fn get_import_meta_props(&self, _: Span, _: &ModuleRecord) -> Result<Vec<KeyValueProp>, Error> {
        unimplemented!()
    }
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm-bundler!");
}

#[wasm_bindgen]
pub fn bundle() -> String {
    utils::set_panic_hook();

    let globals = Box::leak(Box::new(Globals::default()));

    let source_map =
        SourceMap::with_file_loader(Box::new(InMemFileLoader::new()), FilePathMapping::empty());

    let cm = Lrc::new(source_map);

    let mut bundler = Bundler::new(
        globals,
        cm.clone(),
        Loader { cm: cm.clone() },
        PathResolver,
        swc_bundler::Config {
            require: true,
            external_modules: vec![],
            ..Default::default()
        },
        Box::new(Noop),
    );

    let mut entries = HashMap::default();

    entries.insert("main".to_string(), FileName::Real("main.js".into()));

    let mut bundles = bundler.bundle(entries).expect("failed to bundle");

    let bundle = bundles.pop().unwrap();

    let mut buf = vec![];

    let mut emitter = Emitter {
        cfg: swc_ecma_codegen::Config {
            ..Default::default()
        },
        cm: cm.clone(),
        comments: None,
        wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
    };

    emitter.emit_module(&bundle.module).unwrap();

    String::from_utf8_lossy(&buf).to_string()
}
