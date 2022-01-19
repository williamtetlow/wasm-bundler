use anyhow::Error;
use swc_bundler::{Bundler as SwcBundler, Hook, Load, ModuleData, ModuleRecord, Resolve};
use swc_common::{sync::Lrc, FileName, FilePathMapping, Globals, SourceMap, Span};
use swc_ecma_ast::{EsVersion, KeyValueProp};
use swc_ecma_codegen::{text_writer::JsWriter, Emitter};
use swc_ecma_parser::{lexer::Lexer, EsConfig, Parser, StringInput, Syntax};

use crate::filesystem::{InMemFileLoader, InMemFileSystem};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct Bundler {
    fs: Arc<Mutex<InMemFileSystem>>,
}

impl Bundler {
    pub fn new(fs: Arc<Mutex<InMemFileSystem>>) -> Self {
        Bundler { fs }
    }

    pub fn bundle(&self, entries: HashMap<String, FileName>) -> Result<String, Error> {
        let globals = Box::leak(Box::new(Globals::default()));

        let source_map = SourceMap::with_file_loader(
            Box::new(InMemFileLoader::new(Arc::clone(&self.fs))),
            FilePathMapping::empty(),
        );

        let cm = Lrc::new(source_map);

        let mut bundler = SwcBundler::new(
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

        let result = bundler.bundle(entries);

        match result {
            Ok(mut bundles) => {
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

                let code = String::from_utf8_lossy(&buf).to_string();

                Ok(code)
            }
            Err(err) => Err(err),
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
