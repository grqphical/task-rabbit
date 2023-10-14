use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    pub name: String,
    pub author: String,
    pub default_task: Option<String>
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String
}

pub type EnvVars = Vec<EnvVar>;
pub type Commands = Vec<String>;
pub type Tasks = HashMap<String, Task>;

#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub vars: Option<EnvVars>,
    pub commands: Commands
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskConfig {
    pub info: Info,
    pub tasks: Tasks
}