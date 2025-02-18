use crate::inventory::host::Host;
use crate::inventory::manager::InventoryManager;
use crate::playbook::play::Play;
use crate::playbook::task::Task;
use crate::vars::variable::Variable;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct VariableManager<'a> {
    playbook_dir: PathBuf,
    inventory_manager: &'a InventoryManager,
}

impl<'a> VariableManager<'a> {
    pub fn new(inventory_manager: &'a InventoryManager) -> Self {
        VariableManager {
            playbook_dir: inventory_manager.get_base_dir().to_path_buf(),
            inventory_manager,
        }
    }

    fn combine_and_track(
        &self,
        vars: &mut HashMap<String, Variable>,
        new_vars: &HashMap<String, Variable>,
    ) {
        if new_vars.is_empty() {
            return;
        }

        // TODO: see if we need to add tracking of sources here

        for (key, value) in new_vars {
            vars.insert(key.clone(), value.clone());
        }
    }

    /// Returns the variables, with optional "context" given via the parameters
    /// for the play, host, and task (which could possibly result in different
    /// sets of variables being returned due to the additional context).
    ///
    /// # Order of Precedence:
    /// 1. `play->roles->get_default_vars` - (if there is a play context)
    /// 2. `group_vars_files[host]` - (if there is a host context)
    /// 3. `host_vars_files[host]` - (if there is a host context)
    /// 4. `host->get_vars` - (if there is a host context)
    /// 5. `fact_cache[host]` - (if there is a host context)
    /// 6. `play vars` - (if there is a play context)
    /// 7. `play vars_files` - (if there’s no host context, ignoring file names that cannot be templated)
    /// 8. `task->get_vars` - (if there is a task context)
    /// 9. `vars_cache[host]` - (if there is a host context)
    /// 10. `extra vars`
    ///
    /// # Parameters:
    /// - `play`: Optional context for play-specific variables.
    /// - `host`: Optional context for host-specific variables.
    /// - `task`: Optional context for task-specific variables.
    /// - `include_hostvars`: Indicates whether to include host variables.
    /// - `use_cache`: Use cached variables if available.
    ///
    /// # Returns:
    /// A `HashMap` containing the combined set of variables, respecting the provided
    /// precedence rules and available context.
    pub fn get_vars(
        &self,
        play: Option<&Play>,
        host: Option<&Host>,
        task: Option<&Task>,
        include_hostvars: bool,
        use_cache: bool,
    ) -> HashMap<String, Variable> {
        let mut all_vars = HashMap::new();

        let magic_vars = self.get_magic_vars(play, host, task, include_hostvars);

        if let Some(play) = play {
            // get role defaults (lowest precedence)
            for role in play.roles() {
                // TODO: process roles
            }
        }

        if let Some(task) = task {
            // TODO: process task
        }

        if let Some(host) = host {
            // TODO: process host
        }

        if let Some(play) = play {
            self.combine_and_track(&mut all_vars, play.vars());

            let var_files = play.vars_files();
            for var_file in var_files {
                // TODO: process var files
            }

            for role in play.roles() {
                // TODO: process play roles
            }
        }

        // next, we merge in the vars from the role, which will specifically
        // follow the role dependency chain, and then we merge in the tasks
        // vars (which will look at parent blocks/task includes)
        if let Some(task) = task {
            // TODO: process task vars
        }

        // next, we merge in the vars cache (include vars) and nonpersistent
        // facts cache (set_fact/register), in that order
        if let Some(host) = host {
            // TODO
        }

        // next, we merge in role params and task include params
        if let Some(task) = task {
            // TODO
        }

        // add extra vars
        // TODO: load and merge extra vars

        // TODO: check for any reserved vars

        self.combine_and_track(&mut all_vars, &magic_vars);

        // special case for the 'environment' magic variable, as someone
        // may have set it as a variable and we don't want to stomp on it
        if let Some(task) = task {
            // TODO:
            //all_vars.insert(String::from("environment"), task.environment())
        }

        all_vars
    }

    fn get_magic_vars(
        &self,
        play: Option<&Play>,
        host: Option<&Host>,
        task: Option<&Task>,
        include_hostvars: bool,
    ) -> HashMap<String, Variable> {
        let mut magic_vars: HashMap<String, Variable> = HashMap::new();

        magic_vars.insert(
            String::from("playbook_dir"),
            Variable::Path(self.playbook_dir.clone()),
        );

        if let Some(play) = play {
            // TODO: get all role names and assign to cogrs_role_names
            // TODO: handle task role if any
            magic_vars.insert(
                String::from("cogrs_play_name"),
                Variable::String(play.name().to_string()),
            );
        }

        if let Some(task) = task {
            // TODO: get magic vars for task
        }

        // TODO: set groups var

        magic_vars
    }
}
