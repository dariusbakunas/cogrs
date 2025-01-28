use anyhow::Result;
use cogrs::inventory::manager::InventoryManager;
use std::path::PathBuf;

fn setup_inventory_manager(inventory_file: &str) -> Result<InventoryManager> {
    let inventory_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/inventory");
    let inventory_path = inventory_dir.join(inventory_file);
    let mut inventory_manager = InventoryManager::new();
    let sources = vec![inventory_path.to_str().unwrap().to_string()];
    inventory_manager.parse_sources(Some(&sources))?;
    Ok(inventory_manager)
}

fn validate_groups(inventory_manager: &InventoryManager, expected_groups: &[&str]) {
    let groups = inventory_manager.list_groups();
    assert_eq!(groups.len(), expected_groups.len());
    for (i, &expected_group) in expected_groups.iter().enumerate() {
        assert_eq!(groups[i].name, expected_group);
    }
}

fn validate_hosts(
    inventory_manager: &InventoryManager,
    filter: &str,
    expected_hosts: &[&str],
) -> Result<()> {
    let hosts = inventory_manager.filter_hosts(filter, None)?;
    assert_eq!(hosts.len(), expected_hosts.len());
    for (i, &expected_host) in expected_hosts.iter().enumerate() {
        assert_eq!(hosts[i].name, expected_host);
    }
    Ok(())
}

#[test]
fn test_basic_inventory_no_limits() -> Result<()> {
    let inventory_manager = setup_inventory_manager("basic.yaml")?;

    // Validate groups
    validate_groups(
        &inventory_manager,
        &["ungrouped", "webservers", "dbservers"],
    );

    // Validate hosts without limits
    validate_hosts(
        &inventory_manager,
        "all",
        &[
            "mail.example.com",
            "foo.example.com",
            "bar.example.com",
            "one.example.com",
            "two.example.com",
            "three.example.com",
        ],
    )?;

    // Validate webservers group hosts
    validate_hosts(
        &inventory_manager,
        "webservers",
        &["foo.example.com", "bar.example.com"],
    )?;

    Ok(())
}

#[test]
fn test_basic_relationships_no_limits() -> Result<()> {
    let inventory_manager = setup_inventory_manager("basic_relationships.yaml")?;

    // Validate groups
    validate_groups(
        &inventory_manager,
        &[
            "ungrouped",
            "webservers",
            "dbservers",
            "east",
            "west",
            "prod",
            "test",
        ],
    );

    // Validate hosts without limits
    validate_hosts(
        &inventory_manager,
        "all",
        &[
            "mail.example.com",
            "foo.example.com",
            "bar.example.com",
            "one.example.com",
            "three.example.com",
        ],
    )?;

    // Validate prod hosts
    validate_hosts(
        &inventory_manager,
        "prod",
        &["foo.example.com", "one.example.com"],
    )?;

    Ok(())
}
