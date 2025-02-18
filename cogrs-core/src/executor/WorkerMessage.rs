pub enum WorkerMessage {
    Callback(String),
    Display(String),
    Prompt(String),
}
