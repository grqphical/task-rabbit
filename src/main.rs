use clap::Parser;
use std::{
    collections::HashMap,
    path::Path,
    process::{Command, Stdio},
};
use colored::*;

mod models;

const TASKS_FILE: &str = "taskrabbit.toml";

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
    eprintln!("{} {}", "ERROR:".red().bold(),message);
    std::process::exit(1);
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
fn select_task(task_config: &models::TaskConfig, task: Option<String>) -> String {
    let selected_task: String;
    if task == None {
        if task_config.info.default_task == None {
            handle_error("No default task sepcified")
        }
        selected_task = task_config.info.default_task.clone().unwrap();
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
fn run_task(task: String, task_config: &models::TaskConfig) {
    let mut env_vars: HashMap<String, String> = HashMap::new();
    let task = task_config.tasks.get(&task).unwrap();

    for var in task.vars.clone().unwrap_or(vec![]).as_slice() {
        env_vars.insert(var.name.to_string(), var.value.to_string());
    }

    if cfg!(windows) {
        let mut env = Command::new("cmd");
        env.arg("/c")
            .stdout(Stdio::inherit())
            .stdin(Stdio::inherit())
            .stderr(Stdio::inherit());
        env.envs(env_vars);

        for command in task.commands.clone() {
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
        for command in task.commands.clone() {
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

fn main() {
    let args = Args::parse();

    if !Path::new(TASKS_FILE).exists() {
        handle_error("Could not find 'taskrabbit.toml' in current directory")
    }

    let data = std::fs::read_to_string(TASKS_FILE).unwrap();
    let task_config: models::TaskConfig = toml::from_str(&data).unwrap();

    if args.list {
        println!("Tasks Available:");
        for task in task_config.tasks.keys() {
            if *task == task_config.info.default_task.clone().unwrap_or(String::from("")) {
                println!("  {}", task.cyan().bold());
            } else {
                println!("  {}", task);
            }
        }
        return;
    }

    let task: String = select_task(&task_config, args.task_name);

    run_task(task, &task_config);
}
