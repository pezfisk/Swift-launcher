use include_dir::{Dir, include_dir};
use std::collections::HashMap;
use std::env;
use std::option::Option;
use std::path::PathBuf;
use wasmtime::component::{Component, Linker, ResourceTable, bindgen};
use wasmtime::{Config, Engine, Store};
use wasmtime_wasi::{DirPerms, FilePerms, WasiCtx, WasiCtxBuilder, WasiCtxView, WasiView};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

use rayon::current_thread_index;

bindgen!({ world: "plugin-world", path: "plugin.wit" });

static BUILTIN_PLUGINS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/built_in_plugins");

pub struct MyState {
    wasi: WasiCtx,
    http_ctx: WasiHttpCtx,
    table: ResourceTable,
}
pub struct PluginManager {
    engine: Engine,
    linker: Linker<MyState>,
    plugins: HashMap<char, Component>,
}

impl WasiView for MyState {
    fn ctx(&mut self) -> WasiCtxView<'_> {
        WasiCtxView {
            ctx: &mut self.wasi,
            table: &mut self.table,
        }
    }
}
impl WasiHttpView for MyState {
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http_ctx
    }

    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let mut config = Config::new();
        config.wasm_component_model(true);
        // config.async_support(true);
        let engine = Engine::new(&config).expect("WASM engine failed");

        let mut linker = Linker::new(&engine);
        wasmtime_wasi::p2::add_to_linker_sync(&mut linker).expect("Failed to add WASI");
        wasmtime_wasi_http::add_only_http_to_linker_sync(&mut linker)
            .expect("Failed to add WASI_HTTP");
        Self {
            engine: engine.clone(),
            linker,
            plugins: HashMap::new(),
        }
    }

    pub fn load_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut plugin_sources = Vec::new();

        // 2. Load User Plugins (from disk)
        let user_path = format!(
            "{}/swift/plugins",
            std::env::var("XDG_CONFIG_HOME")
                .unwrap_or_else(|_| format!("{}/.config", std::env::var("HOME").unwrap()))
        );

        for file in BUILTIN_PLUGINS.files() {
            // println!("Loading: {:?}", file);
            // self.register(file.contents())?;

            // println!("{:?}", file.path());
            let path = PathBuf::from(format!("{}/{}", user_path, file.path().display()));
            plugin_sources.push((path.to_path_buf(), file.contents().to_vec()))
        }

        if let Ok(entries) = std::fs::read_dir(&user_path) {
            for entry in entries.flatten() {
                // println!("Loading {:?}", entry);
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("wasm") {
                    // println!("Loaded");
                    // self.register(&std::fs::read(entry.path())?)?;
                    plugin_sources.push((path, std::fs::read(entry.path())?))
                }
            }
        }

        let engine = self.engine.clone();
        let compiled_results: Vec<_> = plugin_sources
            .into_iter()
            .map(|(path, bytes)| {
                println!("Processing {:?} at {:?}", path, current_thread_index());
                let cwasm_path = path.with_extension("cwasm");

                if cwasm_path.exists() {
                    println!("Using cwasm");
                    unsafe { Component::deserialize_file(&engine, &cwasm_path) }
                } else {
                    println!("Compiling module");
                    let component = Component::from_binary(&engine, &bytes)?;

                    let _ = std::fs::write(cwasm_path, engine.precompile_component(&bytes)?);
                    Ok(component)
                }
            })
            .collect();

        for result in compiled_results {
            self.register(result?)?;
        }

        Ok(())
    }

    fn register(&mut self, component: Component) -> Result<(), Box<dyn std::error::Error>> {
        let mut store = self.create_store();
        let world = PluginWorld::instantiate(&mut store, &component, &self.linker)?;
        if let Ok(t) = world.swift_launcher_runner().call_get_trigger(&mut store)
            && let Some(c) = t.chars().next() {
                self.plugins.insert(c, component);
            }
        Ok(())
    }

    pub fn run_trigger(
        &self,
        trigger: char,
        input: &str,
    ) -> Option<Vec<exports::swift::launcher::runner::ActionItem>> {
        let comp = self.plugins.get(&trigger)?;
        let mut store = self.create_store();
        let world = PluginWorld::instantiate(&mut store, comp, &self.linker).ok()?;
        world
            .swift_launcher_runner()
            .call_handle(&mut store, input)
            .ok()
    }

    fn create_store(&self) -> Store<MyState> {
        let config_dir = format!("{}/.config/swift", env::var("HOME").expect("Could not find home directory"));
        println!("{}", config_dir);
        
        let mut builder = WasiCtxBuilder::new();
        builder.inherit_stdio();

        builder.inherit_network();

        builder
            .preopened_dir("/", "/", DirPerms::READ, FilePerms::READ)
            .expect("Failed to preopen /");

        builder.preopened_dir(config_dir, "/config", DirPerms::READ, FilePerms::READ)
               .expect("Failed to preopen /config");

        Store::new(
            &self.engine,
            MyState {
                wasi: builder.build(),
                http_ctx: WasiHttpCtx::new(),
                table: ResourceTable::new(),
            },
        )
    }
}
