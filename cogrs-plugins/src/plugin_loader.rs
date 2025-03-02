use crate::callback::CallbackPlugin;
use crate::connection::ConnectionPlugin;
use crate::plugin_type::PluginType;
use crate::shell::ShellPlugin;
use anyhow::{bail, Context, Result};
use libloading::{Library, Symbol};
use log::warn;
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::{c_char, CStr};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct PluginLoader {
    // plugins that we usually get by type and name
    named_plugin_paths: HashMap<PluginType, HashMap<String, PathBuf>>,
    unnamed_plugin_paths: HashMap<PluginType, Vec<PathBuf>>,
}

impl PluginLoader {
    fn new() -> Self {
        PluginLoader {
            named_plugin_paths: HashMap::new(),
            unnamed_plugin_paths: HashMap::new(),
        }
    }

    pub async fn init(&mut self, plugin_paths: HashMap<PluginType, Vec<PathBuf>>) -> Result<()> {
        let plugin_extension = Self::get_plugin_extension();

        for (plugin_type, paths) in plugin_paths {
            for path in paths {
                let entries = match self.read_plugin_directory(&path).await {
                    Ok(entries) => entries,
                    Err(e) => {
                        warn!("Failed to read plugin directory {:?}: {}", path, e);
                        continue;
                    }
                };

                for entry in entries {
                    let plugin_path = match self.get_plugin_path(&entry, &path).await {
                        Ok(path) => path,
                        Err(e) => {
                            warn!("Skipping invalid directory entry in {:?}: {}", &path, e);
                            continue;
                        }
                    };

                    if self.is_valid_plugin_file(&plugin_path, &plugin_extension) {
                        let actual_plugin_type = unsafe {
                            let lib = Library::new(plugin_path.to_path_buf())
                                .with_context(|| "Failed to load plugin")?;

                            self.get_plugin_type(&lib)?
                        };

                        let actual_plugin_type = PluginType::from_u64(actual_plugin_type);

                        if let Some(actual_plugin_type) = actual_plugin_type {
                            if actual_plugin_type != plugin_type {
                                warn!(
                                    "Skipping {} plugin at {:?} (wrong type, expected {})",
                                    actual_plugin_type, &plugin_path, plugin_type
                                );
                                continue;
                            }

                            // verify plugin versions
                            let plugin_versions = unsafe {
                                let lib = Library::new(plugin_path.to_path_buf())
                                    .with_context(|| "Failed to load plugin")?;
                                self.get_plugin_versions(&lib)?
                            };

                            let version_constraints = Self::get_version_constraints();

                            // Verify versions
                            Self::verify_plugin_versions(plugin_versions, version_constraints)
                                .with_context(|| {
                                    format!(
                                        "Plugin at {:?} contains incompatible library versions",
                                        plugin_path
                                    )
                                })?;

                            match actual_plugin_type {
                                PluginType::Callback => {
                                    self.unnamed_plugin_paths
                                        .entry(PluginType::Callback)
                                        .and_modify(|paths| paths.push(plugin_path.to_path_buf()))
                                        .or_insert_with(|| vec![plugin_path]);
                                }
                                PluginType::Connection | PluginType::Shell => {
                                    let plugin_name =
                                        unsafe { self.get_plugin_name(&plugin_path)? };
                                    self.named_plugin_paths
                                        .entry(actual_plugin_type.clone())
                                        .or_insert_with(HashMap::new)
                                        .insert(plugin_name, plugin_path.to_path_buf());
                                }
                                _ => {
                                    warn!(
                                        "Skipping {} plugin at {:?} (not implemented)",
                                        plugin_type, &plugin_path
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
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

    fn get_version_constraints() -> HashMap<String, VersionReq> {
        HashMap::from([
            (
                "cogrs-plugin".to_string(),
                VersionReq::parse("^1.2").unwrap(),
            ),
            (
                "cogrs-schema".to_string(),
                VersionReq::parse("^2.0").unwrap(),
            ),
        ])
    }

    unsafe fn get_plugin_versions(&self, lib: &Library) -> Result<HashMap<String, Version>> {
        let versions_fn: Symbol<unsafe extern "C" fn() -> *const c_char> = lib
            .get(b"cogrs_versions")
            .with_context(|| "Failed to retrieve `cogrs_versions` function")?;

        let versions_cstr = CStr::from_ptr(versions_fn());
        let versions_json: HashMap<String, String> = serde_json::from_str(versions_cstr.to_str()?)
            .with_context(|| {
                "Failed to parse `cogrs_versions` output as a `HashMap<String, String>`"
            })?;

        // Parse each version string into a `semver::Version`
        versions_json
            .into_iter()
            .map(|(key, value)| {
                Version::parse(&value)
                    .map(|version| (key.clone(), version))
                    .map_err(|err| anyhow::anyhow!("Failed to parse version for {}: {}", key, err))
            })
            .collect()
    }

    fn verify_plugin_versions(
        plugin_versions: HashMap<String, Version>,
        version_constraints: HashMap<String, VersionReq>,
    ) -> Result<()> {
        for (lib, plugin_version) in plugin_versions {
            if let Some(constraint) = version_constraints.get(&lib) {
                if !constraint.matches(&plugin_version) {
                    bail!(
                        "Incompatible version for library `{}`: expected {}, found {}",
                        lib,
                        constraint,
                        plugin_version
                    );
                }
            } else {
                warn!(
                    "Plugin declares unexpected library `{}` with version {}",
                    lib, plugin_version
                );
            }
        }
        Ok(())
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

    pub async fn get_named_plugin<T>(
        &mut self,
        plugin_type: PluginType,
        name: &str,
        load_plugin_fn: unsafe fn(&PluginLoader, &Path, &str) -> Result<Option<Box<T>>>,
    ) -> Result<Box<T>>
    where
        T: ?Sized,
    {
        let plugin_path = self
            .named_plugin_paths
            .get(&plugin_type)
            .and_then(|paths| paths.get(name).map(|path| path.to_path_buf()));

        if let Some(path) = plugin_path {
            if let Some(plugin) = unsafe { load_plugin_fn(self, &path, name)? } {
                return Ok(plugin);
            }
        }

        bail!("{} plugin {} not found", plugin_type, name);
    }

    pub async fn get_connection_plugin(&mut self, name: &str) -> Result<Box<dyn ConnectionPlugin>> {
        self.get_named_plugin(
            PluginType::Connection,
            name,
            PluginLoader::load_connection_plugin,
        )
        .await
    }

    pub async fn get_shell_plugin(&mut self, name: &str) -> Result<Box<dyn ShellPlugin>> {
        self.get_named_plugin(PluginType::Shell, name, PluginLoader::load_shell_plugin)
            .await
    }

    unsafe fn get_plugin_name(&self, path: &PathBuf) -> Result<String> {
        let lib =
            Library::new(path).with_context(|| format!("Failed to load plugin at {:?}", path))?;
        let plugin_name_fn: Symbol<unsafe extern "C" fn() -> *const c_char> = lib
            .get(b"plugin_name")
            .with_context(|| "Failed to retrieve `plugin_name` function")?;
        Ok(CStr::from_ptr(plugin_name_fn()).to_str()?.to_string())
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
        let mut plugins: Vec<Arc<dyn CallbackPlugin>> = Vec::new();

        if let Some(paths) = self.unnamed_plugin_paths.get(&PluginType::Callback) {
            for plugin_path in paths {
                if let Some(plugin) = unsafe { self.load_callback_plugin(&plugin_path) }? {
                    plugins.push(plugin);
                }
            }
        } else {
            warn!("No callback plugin paths configured, no callback plugins will be loaded.");
            return Ok(plugins);
        }

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
