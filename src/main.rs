use anyhow::Result;
use clap::{arg, command, Parser};
use log::info;

mod task_solver;

/// Program to compute solution of AOC tasks
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Day in the advent of code calendar
    #[arg(short, long)]
    day: u8,
    /// Task number on that day (either 1 or 2)
    #[arg(short, long)]
    task: u8,
    /// Path to the input file
    #[arg(short, long)]
    input: String,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();

    info!(
        "Solving AOC task {}-{} with input {}",
        args.day, args.task, args.input
    );

    task_solver::solve_task(args.day, args.task, args.input)
}
