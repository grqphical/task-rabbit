use std::collections::HashMap;
use serde::Deserialize;
use toml::Value;

pub type EnvVars = Vec<EnvVar>;
pub type Commands = Vec<String>;
pub type Tasks = HashMap<String, Task>;
pub type Variables = HashMap<String, Value>;

#[derive(Debug, Clone, Deserialize)]
pub struct Info {
    pub name: String,
    pub author: String,
    pub default_windows_task: Option<String>,
    pub default_linux_task: Option<String>,
    pub default_macos_task: Option<String>,
    pub default_task: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String
}

#[derive(Debug, Clone, Deserialize)]
pub struct Task {
    pub env_vars: Option<EnvVars>,
    pub commands: Commands,
    pub platforms_supported: Option<Vec<String>>,
    pub dotenv_file: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TaskConfig {
    pub info: Info,
    pub tasks: Tasks,
    pub variables: Variables
}