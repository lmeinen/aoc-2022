use anyhow::{anyhow, Result};

mod day_1;
mod day_10;
mod day_11;
mod day_12;
mod day_13;
mod day_14;
mod day_15;
mod day_16;
mod day_17;
mod day_18;
mod day_19;
mod day_2;
mod day_20;
mod day_3;
mod day_4;
mod day_5;
mod day_6;
mod day_7;
mod day_8;
mod day_9;
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
        9 => day_9::solve(task, input),
        10 => day_10::solve(task, input),
        11 => day_11::solve(task, input),
        12 => day_12::solve(task, input),
        13 => day_13::solve(task, input),
        14 => day_14::solve(task, input),
        15 => day_15::solve(task, input),
        16 => day_16::solve(task, input),
        17 => day_17::solve(task, input),
        18 => day_18::solve(task, input),
        19 => day_19::solve(task, input),
        20 => day_20::solve(task, input),
        _ => Err(anyhow!("Haven't solved any tasks for this day, yet! Are you sure we're this far into December already?"))
    }
}
