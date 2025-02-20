use std::path::PathBuf;

/// This struct is used to consolidate the connection information for
/// hosts in a play and child tasks, where the task may override some
/// connection/authentication information.
#[derive(Clone, Debug)]
pub struct PlayContext {
    any_errors_fatal: bool,
    become_exe: Option<String>,
    become_flags: Option<String>,
    become_method: Option<String>,
    become_user: Option<String>,
    check_mode: bool,
    connection: String,
    connection_user: Option<String>,
    diff_mode: bool,
    force_handlers: bool,
    no_log: bool,
    private_key_file: Option<PathBuf>,
    start_at_task: Option<String>,
    step: bool,
    throttle: usize,
    connection_timeout: Option<u64>,
    use_become: bool,
}

impl PlayContext {
    fn new(
        any_errors_fatal: bool,
        become_exe: Option<String>,
        become_flags: Option<String>,
        become_method: Option<String>,
        become_user: Option<String>,
        check_mode: bool,
        connection: String,
        connection_user: Option<String>,
        diff_mode: bool,
        force_handlers: bool,
        no_log: bool,
        private_key_file: Option<PathBuf>,
        start_at_task: Option<String>,
        step: bool,
        throttle: usize,
        connection_timeout: Option<u64>,
        use_become: bool,
    ) -> Self {
        PlayContext {
            any_errors_fatal,
            become_exe,
            become_flags,
            become_method,
            become_user,
            check_mode,
            connection,
            connection_user,
            diff_mode,
            force_handlers,
            no_log,
            private_key_file,
            start_at_task,
            step,
            throttle,
            connection_timeout,
            use_become,
        }
    }
}

pub struct PlayContextBuilder {
    any_errors_fatal: bool,
    become_exe: Option<String>,
    become_flags: Option<String>,
    become_method: Option<String>,
    become_user: Option<String>,
    check_mode: bool,
    connection: String,
    connection_user: Option<String>,
    diff_mode: bool,
    force_handlers: bool,
    no_log: bool,
    private_key_file: Option<PathBuf>,
    start_at_task: Option<String>,
    step: bool,
    throttle: usize,
    connection_timeout: Option<u64>,
    use_become: bool,
}

impl PlayContextBuilder {
    pub fn new() -> Self {
        PlayContextBuilder {
            any_errors_fatal: false,
            become_exe: None,
            become_flags: None,
            become_method: None,
            become_user: None,
            check_mode: false,
            connection: String::from("ssh"),
            connection_user: None,
            diff_mode: false,
            force_handlers: false,
            no_log: false,
            private_key_file: None,
            start_at_task: None,
            step: false,
            throttle: 0,
            connection_timeout: Some(10),
            use_become: false,
        }
    }

    pub fn connection_timeout(mut self, timeout: Option<u64>) -> Self {
        self.connection_timeout = timeout;
        self
    }

    pub fn connection_user(mut self, user: Option<&str>) -> Self {
        self.connection_user = user.map(|u| String::from(u));
        self
    }

    pub fn private_key_file(mut self, file: Option<&PathBuf>) -> Self {
        self.private_key_file = file.cloned();
        self
    }

    pub fn build(self) -> PlayContext {
        PlayContext::new(
            self.any_errors_fatal,
            self.become_exe,
            self.become_flags,
            self.become_method,
            self.become_user,
            self.check_mode,
            self.connection,
            self.connection_user,
            self.diff_mode,
            self.force_handlers,
            self.no_log,
            self.private_key_file,
            self.start_at_task,
            self.step,
            self.throttle,
            self.connection_timeout,
            self.use_become,
        )
    }
}
