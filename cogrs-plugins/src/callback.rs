use serde_json::Value;
use std::sync::Arc;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum EventType {
    RunnerOnFailed,
    RunnerOnOk,
    RunnerOnSkipped,
    RunnerOnUnreachable,
    OnFileDiff,
    PlaybookOnStart,
    PlaybookOnPlayStart,
    PlaybookOnHandlerTaskStart,
    PlaybookOnTaskStart,
}

pub trait CallbackPlugin: Send + Sync {
    /// The list of events the plugin is interested in handling.
    fn get_interested_events(&self) -> Vec<EventType>;

    /// Called when an event triggers that the plugin has registered for.
    fn on_event(&self, event: &EventType, data: Option<&Value>);
}

#[macro_export]
macro_rules! create_callback_plugin {
    // Macro expects the plugin name, events it handles, and methods to implement
    ($plugin_name:ident, $plugin_name_str: expr, [$($event:expr),*], $handler:expr) => {
        pub struct $plugin_name;

        impl CallbackPlugin for $plugin_name {
            fn get_interested_events(&self) -> Vec<EventType> {
                vec![$($event),*]
            }

            fn on_event(&self, event: &EventType, data: Option<&serde_json::Value>) {
                if let Err(e) = (|| -> Result<(), Arc<dyn std::error::Error>> {
                    $handler(event, data)
                })() {
                    eprintln!("Error in plugin '{}': {:?}", stringify!($plugin_name), e);
                }
            }

        }

        #[no_mangle]
        pub fn create_plugin() -> Arc<dyn CallbackPlugin> {
            Arc::new($plugin_name)
        }

        #[no_mangle]
        pub extern "C" fn plugin_type() -> u64 {
            PluginType::Callback.id()
        }

        #[no_mangle]
        pub extern "C" fn plugin_name() -> *const u8 {
            $plugin_name_str.as_ptr()
        }
    };
}
