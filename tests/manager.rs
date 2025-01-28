use anyhow::Result;
use cogrs::inventory::manager::InventoryManager;
use rstest::rstest;
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
    inventory: &str,
    pattern: &str,
    limit: Option<&str>,
    expected_hosts: &[&str],
) -> Result<()> {
    let hosts = inventory_manager.filter_hosts(pattern, limit)?;
    let actual_names: Vec<String> = hosts.iter().map(|h| h.name.clone()).collect();

    if actual_names != expected_hosts {
        panic!(
            "Host mismatch for inventory: '{}', pattern '{}', limit: '{}':\n  Expected: {:?}\n  Found:    {:?}",
            inventory,
            pattern,
            limit.unwrap_or("None"),
            expected_hosts,
            actual_names
        );
    }
    Ok(())
}

#[rstest]
#[case("basic.yaml", "all", None, vec!["mail.example.com", "foo.example.com", "bar.example.com", "one.example.com", "two.example.com", "three.example.com"])]
#[case("basic.yaml", "webservers", None, vec!["foo.example.com", "bar.example.com"])]
#[case("basic.yaml", "dbservers", None, vec!["one.example.com", "two.example.com", "three.example.com"])]
#[case("basic.yaml", "webservers", Some("bar*"), vec!["bar.example.com"])]
#[case("basic.yaml", "dbservers", Some("!two.example.com"), vec!["one.example.com", "three.example.com"])]
#[case("basic.yaml", "two.example.com", None, vec!["two.example.com"])]
#[case("basic.yaml", "dbservers[0:1]", None, vec!["one.example.com", "two.example.com"])]
#[case("basic_relationships.yaml", "one.example.com, two.example.com", Some("east"), vec!["one.example.com"])]
#[case("basic_relationships.yaml", "all", None, vec!["mail.example.com", "foo.example.com", "bar.example.com", "one.example.com", "three.example.com"])]
#[case("basic_relationships.yaml", "all", Some("foo*, bar*"), vec!["foo.example.com", "bar.example.com"])]
#[case("basic_relationships.yaml", "prod", None, vec!["foo.example.com", "one.example.com"])]
#[case("basic_relationships.yaml", "prod", Some("!webservers"), vec!["one.example.com"])]
#[case("basic_relationships.yaml", "all", Some("webservers[-1], prod[1]"), vec!["bar.example.com", "one.example.com"])]
#[case("basic_relationships.yaml", "prod,&dbservers", None, vec!["one.example.com"])]
#[case("basic_relationships.yaml", "webservers,&prod", Some("foo*"), vec!["foo.example.com"])]
fn validate_host_patterns_and_limits(
    #[case] inventory: &str,
    #[case] pattern: &str,
    #[case] limit: Option<&str>,
    #[case] expected_hosts: Vec<&str>,
) -> Result<()> {
    let inventory_manager = setup_inventory_manager(inventory).unwrap();
    validate_hosts(
        &inventory_manager,
        inventory,
        pattern,
        limit,
        expected_hosts.as_slice(),
    )?;

    Ok(())
}

#[rstest]
#[case("basic.yaml", vec!["ungrouped", "webservers", "dbservers"])]
#[case("basic_relationships.yaml", vec!["ungrouped", "webservers", "dbservers", "east", "west", "prod", "test"])]
fn validate_inventory_groups(
    #[case] inventory: &str,
    #[case] expected_groups: Vec<&str>,
) -> Result<()> {
    let inventory_manager = setup_inventory_manager(inventory)?;

    validate_groups(&inventory_manager, expected_groups.as_slice());

    Ok(())
}
