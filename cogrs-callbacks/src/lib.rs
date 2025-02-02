use serde_json::Value;

#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub enum EventType {
    PlaybookOnStart(String),
    PlaybookOnPlayStart(String),
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
    ($plugin_name:ident, [$($event:expr),*], $handler:block) => {
        pub struct $plugin_name;

        impl CallbackPlugin for $plugin_name {
            fn get_interested_events(&self) -> Vec<EventType> {
                vec![$($event),*]
            }

            fn on_event(&self, event: &EventType, data: Option<&serde_json::Value>) {
                if let Err(e) = (|| -> Result<(), Box<dyn std::error::Error>> {
                    $handler
                })() {
                    eprintln!("Error in plugin '{}': {:?}", stringify!($plugin_name), e);
                }
            }

        }

        #[no_mangle]
        pub fn create_plugin() -> Box<dyn CallbackPlugin> {
            Box::new($plugin_name)
        }
    };
}
