use crate::callback::CallbackPlugin;
use crate::plugin_type::PluginType;
use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use once_cell::sync::Lazy;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PluginLoader {
    cached_callback_plugins: Mutex<Option<Vec<Arc<dyn CallbackPlugin>>>>,
}

impl PluginLoader {
    fn new() -> Self {
        PluginLoader {
            cached_callback_plugins: Mutex::new(None),
        }
    }

    /// Determines the platform-specific plugin file extension
    fn get_plugin_extension() -> &'static str {
        if cfg!(target_os = "windows") {
            "dll"
        } else if cfg!(target_os = "macos") {
            "dylib"
        } else {
            "so"
        }
    }

    /// Attempts to retrieve the plugin type
    unsafe fn get_plugin_type(&self, lib: &Library) -> Result<u64> {
        let plugin_type_fn: Symbol<unsafe extern "C" fn() -> u64> = lib
            .get(b"plugin_type")
            .with_context(|| "Failed to retrieve `plugin_type` function")?;
        Ok(plugin_type_fn())
    }

    /// Checks whether a given file path corresponds to a valid plugin file
    fn is_valid_plugin_file(&self, path: &Path, plugin_extension: &str) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map_or(false, |ext| ext == plugin_extension)
    }

    /// Attempts to create a callback plugin
    unsafe fn create_callback_plugin(
        &self,
        lib: &Library,
        path: &Path,
    ) -> Result<Arc<dyn CallbackPlugin>> {
        let create_plugin_fn: Symbol<fn() -> Arc<dyn CallbackPlugin>> =
            lib.get(b"create_plugin").with_context(|| {
                format!("Missing `create_plugin` function in plugin at {:?}", path)
            })?;
        Ok(create_plugin_fn())
    }

    /// Loads an individual plugin from a file path.
    unsafe fn load_callback_plugin(&self, path: &Path) -> Result<Option<Arc<dyn CallbackPlugin>>> {
        let lib = Library::new(path).with_context(|| "Failed to load plugin")?;

        let plugin_type_value = self.get_plugin_type(&lib)?;
        match PluginType::from_u64(plugin_type_value) {
            Some(PluginType::Callback) => {
                let plugin = self.create_callback_plugin(&lib, path)?;
                Ok(Some(plugin))
            }
            _ => {
                println!("Skipping unsupported plugin type at {:?}", path);
                Ok(None)
            }
        }
    }

    pub async fn get_callback_plugins(&self) -> Result<Vec<Arc<dyn CallbackPlugin>>> {
        if let Some(cached) = self.get_cached_callback_plugins().await {
            return Ok(cached);
        }

        let mut plugins: Vec<Arc<dyn CallbackPlugin>> = Vec::new();
        // TODO: make this configurable
        let plugin_dir = "/Users/darius/Programming/cogrs/dist/minimal-apple_x86_64-apple-darwin";

        let plugin_extension = Self::get_plugin_extension();

        let entries =
            fs::read_dir(plugin_dir).with_context(|| "Failed to read plugin directory")?;

        for entry in entries {
            let path = entry
                .with_context(|| "Failed to read directory entry")?
                .path();
            if self.is_valid_plugin_file(&path, &plugin_extension) {
                // Load the plugin and register it if valid
                if let Some(plugin) = unsafe { self.load_callback_plugin(&path) }? {
                    plugins.push(plugin);
                }
            }
        }

        self.cache_callback_plugins(plugins.clone()).await;

        Ok(plugins)
    }

    /// Retrieves the cached plugins if available.
    async fn get_cached_callback_plugins(&self) -> Option<Vec<Arc<dyn CallbackPlugin>>> {
        let cache = self.cached_callback_plugins.lock().await;
        cache.as_ref().cloned()
    }

    /// Caches the provided plugins.
    async fn cache_callback_plugins(&self, plugins: Vec<Arc<dyn CallbackPlugin>>) {
        let mut cache = self.cached_callback_plugins.lock().await;
        *cache = Some(plugins);
    }
}

static PLUGIN_LOADER: Lazy<Mutex<PluginLoader>> = Lazy::new(|| Mutex::new(PluginLoader::new()));

impl PluginLoader {
    // Provide access to the singleton instance
    pub fn instance() -> &'static Mutex<PluginLoader> {
        &PLUGIN_LOADER
    }
}
