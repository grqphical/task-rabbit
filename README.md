# taskrabbit - A simple, multiplatform make system

## Installation From Source

Clone this repo and ensure you have `rustc` and `cargo` installed. Then run:
```bash
$ cargo install --path .
```

## Quick Start Guide

Create a new file in a directory with your project called `taskrabbit.toml`. And add the follow info:
```toml
# Info just is used for basic metadata and settings
[info]
name = "Test"
author = "grqphical"
# This defines what task will run if you call task-rabbit with no args
# You can also specify platform specific default tasks like this:
# default_windows_task = "build_win"
default_task = "build"

# This is an example of a basic task
# Tasks can be assigned their own enviroment variables as well as platforms they can run on
[tasks.build]
commands = ["echo I was run with Taskrabbit!"]
```

To run the task you can use:
```bash
$ task-rabbit build
```
However since we have defined `build` as our default task, we can omit the build argument and Taskrabbit will automatically run it still.

For more info please visit the Wiki

## License

Taskrabbit is licensed under the MIT license