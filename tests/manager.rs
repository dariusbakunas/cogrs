use anyhow::Result;
use cogrs::inventory::manager::InventoryManager;
use std::path::PathBuf;

#[test]
fn test_basic_inventory_parsing() -> Result<()> {
    let inventory_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inventory");
    let basic_inventory = inventory_dir.join("basic.yaml");
    let mut inventory_manager = InventoryManager::new();
    let sources = vec![basic_inventory.to_str().unwrap().to_string()];
    inventory_manager.parse_sources(Some(&sources))?;

    let groups = inventory_manager.list_groups();
    assert_eq!(groups.len(), 3);
    assert_eq!(groups[0].name, "ungrouped");
    assert_eq!(groups[1].name, "webservers");
    assert_eq!(groups[2].name, "dbservers");

    // test without limit
    let hosts = inventory_manager.filter_hosts("all", None)?;

    assert_eq!(hosts.len(), 6);
    assert_eq!(hosts[0].name, "mail.example.com");
    assert_eq!(hosts[1].name, "foo.example.com");
    assert_eq!(hosts[2].name, "bar.example.com");
    assert_eq!(hosts[3].name, "one.example.com");
    assert_eq!(hosts[4].name, "two.example.com");
    assert_eq!(hosts[5].name, "three.example.com");

    Ok(())
}
