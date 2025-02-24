use crate::callback::CallbackPlugin;
use crate::connection::ConnectionPlugin;
use crate::plugin_type::PluginType;
use crate::shell::ShellPlugin;
use anyhow::{bail, Context, Result};
use libloading::{Library, Symbol};
use log::warn;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PluginLoader {
    plugin_paths: HashMap<PluginType, Vec<PathBuf>>,
    cached_callback_plugins: Mutex<Option<Vec<Arc<dyn CallbackPlugin>>>>,
}

impl PluginLoader {
    fn new() -> Self {
        PluginLoader {
            cached_callback_plugins: Mutex::new(None),
            plugin_paths: HashMap::new(),
        }
    }

    pub fn set_plugin_paths(&mut self, plugin_paths: HashMap<PluginType, Vec<PathBuf>>) {
        self.plugin_paths = plugin_paths;
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
                warn!("Skipping non-callback plugin at {:?}", path);
                Ok(None)
            }
        }
    }

    async fn read_plugin_directory(&self, path: &Path) -> Result<Vec<fs::DirEntry>> {
        fs::read_dir(path)
            .with_context(|| format!("Failed to read directory {:?}", path))
            .map(|entries| entries.filter_map(Result::ok).collect())
    }

    async fn get_plugin_path(&self, entry: &fs::DirEntry, base_path: &Path) -> Result<PathBuf> {
        entry
            .path()
            .canonicalize()
            .with_context(|| format!("Failed to get canonical path for entry in {:?}", base_path))
    }

    pub async fn get_connection_plugin(&mut self, name: &str) -> Result<Box<dyn ConnectionPlugin>> {
        let plugin_extension = Self::get_plugin_extension();

        if let Some(paths) = self.plugin_paths.get(&PluginType::Connection) {
            for path in paths {
                let entries = match self.read_plugin_directory(path).await {
                    Ok(entries) => entries,
                    Err(e) => {
                        warn!("Failed to read plugin directory {:?}: {}", path, e);
                        continue;
                    }
                };

                for entry in entries {
                    let plugin_path = match self.get_plugin_path(&entry, path).await {
                        Ok(path) => path,
                        Err(e) => {
                            warn!("Skipping invalid directory entry in {:?}: {}", path, e);
                            continue;
                        }
                    };

                    if self.is_valid_plugin_file(&plugin_path, &plugin_extension) {
                        if let Some(plugin) =
                            unsafe { self.load_connection_plugin(&plugin_path, name)? }
                        {
                            return Ok(plugin);
                        }
                    }
                }
            }
        }

        bail!("Connection plugin {} not found", name);
    }

    // Generalized plugin loader implementation
    unsafe fn load_named_plugin<T, F>(
        &self,
        path: &Path,
        name: &str,
        expected_plugin_type: PluginType,
        create_fn_symbol: &[u8],
        name_fn: F,
    ) -> Result<Option<Box<T>>>
    where
        T: ?Sized,
        F: FnOnce(&Library) -> Result<Symbol<fn() -> *const c_char>>, // Function to retrieve the `plugin_name` symbol
    {
        let lib =
            Library::new(path).with_context(|| format!("Failed to load plugin at {:?}", path))?;
        let plugin_type_value = self.get_plugin_type(&lib)?;

        if PluginType::from_u64(plugin_type_value)
            .is_some_and(|plugin_type| plugin_type == expected_plugin_type)
        {
            // Retrieve the plugin name
            let plugin_name_fn = name_fn(&lib)?;
            let plugin_name = CStr::from_ptr(plugin_name_fn()).to_str()?;

            if plugin_name.eq(name) {
                // Retrieve create_plugin function and construct the plugin
                let create_plugin_fn: Symbol<fn() -> Box<T>> =
                    lib.get(create_fn_symbol).with_context(|| {
                        format!("Missing `create_plugin` function in plugin at {:?}", path)
                    })?;
                return Ok(Some(create_plugin_fn()));
            } else {
                warn!(
                    "Skipping plugin at {:?}, name {} does not match {}",
                    path, plugin_name, name
                );
            }
        } else {
            warn!("Skipping non-{} plugin at {:?}", expected_plugin_type, path);
        }

        Ok(None)
    }

    unsafe fn load_shell_plugin(
        &self,
        path: &Path,
        name: &str,
    ) -> Result<Option<Box<dyn ShellPlugin>>> {
        self.load_named_plugin(path, name, PluginType::Shell, b"create_plugin", |lib| {
            lib.get(b"plugin_name").map_err(anyhow::Error::from)
        })
    }

    unsafe fn load_connection_plugin(
        &self,
        path: &Path,
        name: &str,
    ) -> Result<Option<Box<dyn ConnectionPlugin>>> {
        self.load_named_plugin(
            path,
            name,
            PluginType::Connection,
            b"create_plugin",
            |lib| lib.get(b"plugin_name").map_err(anyhow::Error::from),
        )
    }

    pub async fn get_callback_plugins(&self) -> Result<Vec<Arc<dyn CallbackPlugin>>> {
        if let Some(cached) = self.get_cached_callback_plugins().await {
            return Ok(cached);
        }

        let mut plugins: Vec<Arc<dyn CallbackPlugin>> = Vec::new();
        let plugin_extension = Self::get_plugin_extension();

        if let Some(paths) = self.plugin_paths.get(&PluginType::Callback) {
            for path in paths {
                let entries = match self.read_plugin_directory(path).await {
                    Ok(entries) => entries,
                    Err(e) => {
                        warn!("Failed to read plugin directory {:?}: {}", path, e);
                        continue;
                    }
                };

                for entry in entries {
                    let plugin_path = match self.get_plugin_path(&entry, path).await {
                        Ok(path) => path,
                        Err(e) => {
                            warn!("Skipping invalid directory entry in {:?}: {}", path, e);
                            continue;
                        }
                    };
                    if self.is_valid_plugin_file(&plugin_path, &plugin_extension) {
                        // Load the plugin and register it if valid
                        if let Some(plugin) = unsafe { self.load_callback_plugin(&plugin_path) }? {
                            plugins.push(plugin);
                        }
                    }
                }
            }
        } else {
            warn!("No callback plugin paths configured, no callback plugins will be loaded.");
            return Ok(plugins);
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
