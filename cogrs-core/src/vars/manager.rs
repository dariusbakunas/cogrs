use crate::inventory::manager::InventoryManager;
use crate::playbook::play::Play;
use crate::playbook::task::Task;
use crate::vars::variable::Variable;
use indexmap::IndexMap;
use log::debug;

pub struct VariableManager;

impl VariableManager {
    pub fn new() -> Self {
        Self
    }

    pub fn get_vars(
        &self,
        play: &Play,
        task: Option<&Task>,
        inventory_manager: &InventoryManager,
    ) -> IndexMap<String, Variable> {
        let all_vars = IndexMap::new();
        debug!("Getting vars for play {}", play.name);

        let magic_vars = self.get_magic_vars(Some(play), task, inventory_manager, true);

        all_vars
    }

    fn get_magic_vars(
        &self,
        play: Option<&Play>,
        task: Option<&Task>,
        inventory_manager: &InventoryManager,
        include_hostvars: bool,
    ) -> IndexMap<String, Variable> {
        let mut magic_vars: IndexMap<String, Variable> = IndexMap::new();

        magic_vars.insert(
            String::from("playbook_dir"),
            Variable::Path(inventory_manager.get_base_dir().to_path_buf()),
        );

        if let Some(play) = play {
            // TODO: get all role names and assign to cogrs_role_names
            // TODO: handle task role if any
        }

        magic_vars
    }
}

impl Default for VariableManager {
    fn default() -> Self {
        Self::new()
    }
}
