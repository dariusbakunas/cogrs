use cogrs_plugins::create_connection_plugin_exports;
use ssh_lib::Ssh;
use std::collections::HashMap;

create_connection_plugin_exports!(
    Ssh,
    "ssh",
    HashMap::from([("cogrs-plugin", "1.2.3"), ("cogrs-schema", "2.0.0"),])
);
