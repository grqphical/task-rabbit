use clap::Parser;
use colored::*;
use models::TaskConfig;
use toml::Value;
use std::{
    collections::HashMap,
    path::Path,
    process::{Command, Stdio},
};

mod models;

const TASKS_FILE: &str = "taskrabbit.toml";
const SUPPORTED_PLATFORMS: [&str; 3] = ["windows", "macos", "linux"];

#[derive(Parser, Debug, Clone)]
#[command(author = "grqphical", version = env!("CARGO_PKG_VERSION"), about = "A simple way to create easy to run tasks in a repository")]
struct Args {
    /// Task to run. Uses default specified in TOML file if none is provided
    task_name: Option<String>,

    /// Lists all tasks
    #[arg(short, long)]
    list: bool,
}

/// Gracefully handles an error
///
/// # Arguments
///
/// `message` - Information about the error that occured
pub fn handle_error(message: &str) -> ! {
    eprintln!("{} {}", "ERROR:".red().bold(), message);
    std::process::exit(1);
}

/// Converts a TOML value to a string representation
fn toml_value_to_string(value: &Value) -> String {
    match value {
        Value::String(str) => return str.clone(),
        Value::Integer(int) => return int.to_string(),
        Value::Float(float) => return float.to_string(),
        Value::Boolean(bool) => return bool.to_string(),
        Value::Datetime(datetime) => return datetime.to_string(),
        Value::Array(_) => handle_error("Arrays not supported as variables"),
        Value::Table(_) => handle_error("Tables not supported as variables")
    }
}

/// Decides which task to run based on what arguments were given. If a task was given check if it exists
/// otherwise return an error. If no value was given check if there was a default build task specified and if so run that.
/// Otherwise, return an error
///
/// # Arguments
///
/// `task_config` - The parsed configuration
///
/// `task` - The argument given in the CLI
fn select_task(task_config: &TaskConfig, task: Option<String>) -> String {
    let selected_task: String;
    
    if task == None {
        if std::env::consts::OS == "linux" {
            if task_config.info.default_linux_task == None {
                handle_error("No default task specified")
            }
            selected_task = task_config.info.default_linux_task.clone().unwrap();
        } else if std::env::consts::OS == "windows" {
            if task_config.info.default_windows_task == None {
                handle_error("No default task specified")
            }
            selected_task = task_config.info.default_windows_task.clone().unwrap();
        } else if std::env::consts::OS == "macos" {
            if task_config.info.default_macos_task == None {
                handle_error("No default task specified")
            }
            selected_task = task_config.info.default_macos_task.clone().unwrap();
        } else {
            handle_error("No default task specified")
        }
    } else {
        selected_task = task.unwrap().to_string();
    }

    if !task_config.tasks.contains_key(&selected_task) {
        handle_error("Task not found");
    }

    return selected_task;
}

/// Runs all the commands in a task
///
/// # Arguments
///
/// `task` - Name of task to run
///
/// `task_config` - The parsed configuration
fn run_task(task_name: String, task_config: &TaskConfig) {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    let task = task_config.tasks.get(&task_name).unwrap();

    if let Some(platforms) = &task.platforms_supported {
        for platform in platforms {
            if !SUPPORTED_PLATFORMS.contains(&platform.to_lowercase().as_str()) {
                handle_error(&format!("Invalid platform for task '{}'", task_name))
            }
        }

        if !platforms.contains(&std::env::consts::OS.to_string()) {
            handle_error(&format!(
                "Task not supported on this platform. Supported platforms are {:?}",
                platforms
            ))
        }
    }

    for var in task.env_vars.clone().unwrap_or(vec![]).as_slice() {
        env_vars.insert(var.name.to_string(), var.value.to_string());
    }

    if let Some(dotenv_file) = &task.dotenv_file {
        let read_result = std::fs::read_to_string(dotenv_file);
        let data: String;
        match read_result {
            Ok(file_data) => data = file_data,
            Err(_) => handle_error(&format!("Could not read enviroment variables from {}", dotenv_file))
        }

        let lines = data.split("\n");

        for line in lines {
            let split: Vec<&str> = line.split("=").into_iter().collect();

            let name = split[0];
            let value = split[1];

            env_vars.insert(name.to_string(), value.to_string());
        }
    }

    if cfg!(windows) {
        let mut env = Command::new("cmd");
        env.arg("/c")
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit());
        env.envs(env_vars);

        for mut command in task.commands.clone() {
            for (name, value) in &task_config.variables {
                command = command.replace(&format!("$({})", name), &toml_value_to_string(value));
            }
            env.args(command.split(" "));
            let result = env.spawn();

            match result {
                Ok(mut child) => {
                    let status = child.wait().unwrap();
                    if !status.success() {
                        handle_error(&format!(
                            "Command failed. Exit Code ({})",
                            status.code().unwrap_or(1)
                        ))
                    }
                }
                Err(err) => handle_error(&format!("Could not run command. {}", err.to_string())),
            }
        }
    } else {
        for mut command in task.commands.clone() {
            for (name, value) in &task_config.variables {
                command = command.replace(&format!("$({})", name), &value.to_string());
            }
            let mut split: Vec<&str> = command.split(" ").into_iter().collect();
            let cmd = split.remove(0);

            let mut env = Command::new(cmd);
            env.stdout(Stdio::inherit())
                .stdin(Stdio::inherit())
                .stderr(Stdio::inherit());
            env.envs(&env_vars);

            env.args(split);
            let result = env.spawn();

            match result {
                Ok(mut child) => {
                    let status = child.wait().unwrap();
                    if !status.success() {
                        handle_error(&format!(
                            "Command failed. Exit Code ({})",
                            status.code().unwrap_or(1)
                        ))
                    }
                }
                Err(err) => handle_error(&format!("Could not run command. {}", err.to_string())),
            }
        }
    }
}

/// Lists all tasks that can be run.
/// It will highlight the default task for the current platform in cyan
/// as well as show what platforms are supported for each task
fn list_tasks(task_config: &TaskConfig) {
    println!("Tasks Available:");
    for task in task_config.tasks.keys() {
        if std::env::consts::OS == "windows" {
            if *task
                == task_config
                    .info
                    .default_windows_task
                    .clone()
                    .unwrap_or(String::from(""))
            {
                println!("  {}", task.cyan().bold());
            } else {
                println!("  {}", task);
            }
        } else if std::env::consts::OS == "linux" {
            if *task
                == task_config
                    .info
                    .default_windows_task
                    .clone()
                    .unwrap_or(String::from(""))
            {
                println!("  {}", task.cyan().bold());
            } else {
                println!("  {}", task);
            }
        } else if std::env::consts::OS == "macos" {
            if *task
                == task_config
                    .info
                    .default_windows_task
                    .clone()
                    .unwrap_or(String::from(""))
            {
                println!("  {}", task.cyan().bold());
            } else {
                println!("  {}", task);
            }
        }
    }
}

fn main() {
    let args = Args::parse();

    if !Path::new(TASKS_FILE).exists() {
        handle_error("Could not find 'taskrabbit.toml' in current directory")
    }

    let data = std::fs::read_to_string(TASKS_FILE).unwrap();
    let mut task_config: models::TaskConfig = toml::from_str(&data).unwrap();

    // If user used shorthand default_task, assign it to all platforms
    if let Some(default_task) = &task_config.info.default_task {
        task_config.info.default_linux_task = Some(default_task.clone());
        task_config.info.default_macos_task = Some(default_task.clone());
        task_config.info.default_windows_task = Some(default_task.clone());
    }

    // If the user wanted to just list the available tasks, list them without running any
    if args.list {
        list_tasks(&task_config);
        return;
    }

    let task: String = select_task(&task_config, args.task_name);

    run_task(task, &task_config);
}
