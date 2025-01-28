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
    let actual_names: Vec<String> = groups.iter().map(|g| g.name.clone()).collect();

    if actual_names != expected_groups {
        panic!(
            "Group mismatch:\n  Expected: {:?}\n  Found:    {:?}",
            expected_groups, actual_names
        );
    }
}

fn validate_hosts(
    inventory_manager: &InventoryManager,
    pattern: &str,
    limit: Option<&str>,
    expected_hosts: &[&str],
) -> Result<()> {
    let hosts = inventory_manager.filter_hosts(pattern, limit)?;
    let actual_names: Vec<String> = hosts.iter().map(|h| h.name.clone()).collect();

    if actual_names != expected_hosts {
        panic!(
            "Host mismatch for pattern '{}', limit: '{}':\n  Expected: {:?}\n  Found:    {:?}",
            pattern,
            limit.unwrap_or("None"),
            expected_hosts,
            actual_names
        );
    }
    Ok(())
}

#[test]
fn test_basic_inventory_no_limits() -> Result<()> {
    let inventory_manager = setup_inventory_manager("basic.yaml")?;

    validate_groups(
        &inventory_manager,
        &["ungrouped", "webservers", "dbservers"],
    );

    validate_hosts(
        &inventory_manager,
        "all",
        None,
        &[
            "mail.example.com",
            "foo.example.com",
            "bar.example.com",
            "one.example.com",
            "two.example.com",
            "three.example.com",
        ],
    )?;

    validate_hosts(
        &inventory_manager,
        "webservers",
        None,
        &["foo.example.com", "bar.example.com"],
    )?;

    Ok(())
}

#[test]
fn test_basic_relationships_with_limits() -> Result<()> {
    let inventory_manager = setup_inventory_manager("basic_relationships.yaml")?;

    validate_hosts(
        &inventory_manager,
        "all",
        Some("foo*,bar*"),
        &["foo.example.com", "bar.example.com"],
    )?;

    validate_hosts(
        &inventory_manager,
        "prod",
        Some("!webservers"),
        &["one.example.com"],
    )?;

    validate_hosts(
        &inventory_manager,
        "all",
        Some("webservers[-1], prod[1]"),
        &["bar.example.com", "one.example.com"],
    )?;

    Ok(())
}

#[test]
fn test_basic_relationships_no_limits() -> Result<()> {
    let inventory_manager = setup_inventory_manager("basic_relationships.yaml")?;

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

    validate_hosts(
        &inventory_manager,
        "all",
        None,
        &[
            "mail.example.com",
            "foo.example.com",
            "bar.example.com",
            "one.example.com",
            "three.example.com",
        ],
    )?;

    validate_hosts(
        &inventory_manager,
        "prod",
        None,
        &["foo.example.com", "one.example.com"],
    )?;

    Ok(())
}
