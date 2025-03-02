use cogrs_plugins::create_shell_plugin_exports;
use sh_lib::Sh;
use std::collections::HashMap;

create_shell_plugin_exports!(
    Sh,
    "sh",
    HashMap::from([("cogrs-plugin", "1.2.3"), ("cogrs-schema", "2.0.0"),])
);
