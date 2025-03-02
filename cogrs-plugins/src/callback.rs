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
    ($plugin_name:ident, $plugin_name_str: expr, $versions:expr, [$($event:expr),*], $handler:expr) => {
        use serde_json::json;

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

        #[no_mangle]
        pub extern "C" fn cogrs_versions() -> *const std::os::raw::c_char {
            let versions = serde_json::to_string(&$versions)
            .unwrap();

            std::ffi::CString::new(versions).unwrap().into_raw()
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin_type::PluginType;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_get_interested_events() {
        struct MockHandlerPlugin;

        impl CallbackPlugin for MockHandlerPlugin {
            fn get_interested_events(&self) -> Vec<EventType> {
                vec![EventType::RunnerOnFailed, EventType::RunnerOnOk]
            }

            fn on_event(&self, _event: &EventType, _data: Option<&Value>) {
                // Do nothing
            }
        }

        let plugin = MockHandlerPlugin;
        let events = plugin.get_interested_events();

        assert!(events.contains(&EventType::RunnerOnFailed));
        assert!(events.contains(&EventType::RunnerOnOk));
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_on_event_execution() {
        struct MockHandlerPlugin;

        impl CallbackPlugin for MockHandlerPlugin {
            fn get_interested_events(&self) -> Vec<EventType> {
                vec![EventType::RunnerOnOk]
            }

            fn on_event(&self, event: &EventType, data: Option<&Value>) {
                assert_eq!(event, &EventType::RunnerOnOk);
                if let Some(json) = data {
                    assert_eq!(json["key"], "value");
                }
            }
        }

        let plugin = MockHandlerPlugin;

        // Trigger the event
        let data = json!({"key": "value"});
        plugin.on_event(&EventType::RunnerOnOk, Some(&data));
    }

    #[test]
    fn test_macro_generated_plugin() {
        fn custom_handler(
            event: &EventType,
            data: Option<&Value>,
        ) -> Result<(), Arc<dyn std::error::Error>> {
            if let Some(json) = data {
                assert_eq!(event, &EventType::RunnerOnOk);
                assert_eq!(json["key"], "value");
            }
            Ok(())
        }

        create_callback_plugin!(
            TestPlugin,
            "test_plugin",
            HashMap::from([("cogrs-plugin", "1.2.3"),]),
            [EventType::RunnerOnOk],
            custom_handler
        );

        let plugin = TestPlugin;
        let events = plugin.get_interested_events();
        assert!(events.contains(&EventType::RunnerOnOk));
        assert_eq!(events.len(), 1);

        // Test on_event
        let data = json!({"key": "value"});
        plugin.on_event(&EventType::RunnerOnOk, Some(&data));
    }
}
