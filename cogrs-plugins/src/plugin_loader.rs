use crate::callback::CallbackPlugin;
use crate::plugin_type::PluginType;
use anyhow::Result;
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PluginLoader {
    cached_callback_plugins: Mutex<Option<Vec<Arc<dyn CallbackPlugin>>>>,
}

impl PluginLoader {
    // Define methods or associated functions if needed
    fn new() -> Self {
        PluginLoader {
            cached_callback_plugins: Mutex::new(None),
        }
    }

    pub async fn get_callback_plugins(&self) -> Result<Vec<Arc<dyn CallbackPlugin>>> {
        {
            let cache = self.cached_callback_plugins.lock().await;
            if let Some(cached) = &*cache {
                return Ok(cached.clone());
            }
        }

        let mut plugins: Vec<Arc<dyn CallbackPlugin>> = Vec::new();
        // TODO: make this configurable
        let plugin_dir = "/Users/darius/Programming/cogrs/dist/minimal-apple_x86_64-apple-darwin";

        use libloading::{Library, Symbol};
        use std::fs;

        let plugin_extension = if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        };

        for entry in fs::read_dir(plugin_dir).expect("Invalid plugin directory") {
            let path = entry.expect("Failed to read entry").path();
            if path.extension().and_then(|e| e.to_str()) == Some(plugin_extension) {
                unsafe {
                    let lib = Library::new(&path).expect("Failed to load plugin");

                    let plugin_type_fn: Symbol<unsafe extern "C" fn() -> u64> =
                        lib.get(b"plugin_type").unwrap();
                    let plugin_type_value = plugin_type_fn();

                    // Decode the numeric value into the PluginType enum
                    let plugin_type = match PluginType::from_u64(plugin_type_value) {
                        Some(pt) => pt,
                        None => {
                            println!("Skipping unknown plugin type {}", plugin_type_value);
                            continue; // Skip loading this plugin
                        }
                    };

                    match plugin_type {
                        PluginType::Callback => {
                            let create_callback: Symbol<fn() -> Arc<dyn CallbackPlugin>> =
                                match lib.get(b"create_plugin") {
                                    Ok(symbol) => symbol,
                                    Err(_) => {
                                        println!(
                                        "Skipping plugin {:?}: missing `create_plugin` function.",
                                        path
                                    );
                                        continue;
                                    }
                                };

                            let plugin = create_callback();

                            // Register the plugin for events
                            plugins.push(plugin);
                        }
                        _ => {
                            println!("Skipping OtherPlugin: Not supported.");
                        }
                    }
                }
            }
        }

        let mut cache = self.cached_callback_plugins.lock().await;
        *cache = Some(plugins.clone());

        Ok(plugins)
    }
}

static PLUGIN_LOADER: Lazy<Mutex<PluginLoader>> = Lazy::new(|| Mutex::new(PluginLoader::new()));

impl PluginLoader {
    // Provide access to the singleton instance
    pub fn instance() -> &'static Mutex<PluginLoader> {
        &PLUGIN_LOADER
    }
}
