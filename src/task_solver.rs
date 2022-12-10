use anyhow::{anyhow, Result};

mod day_1;
mod day_2;
mod day_3;
mod day_4;
mod day_5;
mod day_6;
mod day_7;
mod day_8;
mod util;

pub fn solve_task(day: u8, task: u8, input: String) -> Result<()> {
    match day {
        1 => day_1::solve(task, input),
        2 => day_2::solve(task, input),
        3 => day_3::solve(task, input),
        4 => day_4::solve(task, input),
        5 => day_5::solve(task, input),
        6 => day_6::solve(task, input),
        7 => day_7::solve(task, input),
        8 => day_8::solve(task, input),
        _ => Err(anyhow!("Haven't solved any tasks for this day, yet! Are you sure we're this far into December already?"))
    }
}
