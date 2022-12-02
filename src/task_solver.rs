use anyhow::{anyhow, Result};

mod day_1;
mod day_2;
mod util;

pub fn solve_task(day: u8, task: u8, input: String) -> Result<()> {
    match day {
        1 => day_1::solve(task, input),
        2 => day_2::solve(task, input),
        _ => Err(anyhow!("Haven't solved any tasks for this day, yet! Are you sure we're this far into December already?"))
    }
}
