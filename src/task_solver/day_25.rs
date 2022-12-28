use anyhow::{bail, Context, Result};
use log::{debug, info};

use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
};

pub fn solve(_task: u8, input: String) -> Result<()> {
    let snafu_numbers = parse_input(input).context("failed to parse input")?;

    let mut sum = 0u64;
    for snafu in snafu_numbers.iter() {
        sum += snafu_to_decimal(snafu).context("failed to parse snafu number")?;
    }

    info!(
        "SNAFU number to supply to Bob's console: {}",
        dec_to_snafu(sum).context("failed to convert dec to snafu")?
    );

    Ok(())
}

fn snafu_to_decimal(snafu: &str) -> Result<u64> {
    debug!("converting SNAFU number {}", snafu);
    let mut res = 0i64;
    for (i, d) in snafu.chars().enumerate() {
        res += 5i64.pow((snafu.len() - (i + 1)) as u32)
            * match d {
                '2' => 2,
                '1' => 1,
                '0' => 0,
                '-' => -1,
                '=' => -2,
                _ => bail!("unknown digit {}", d),
            };
    }
    Ok(res as u64)
}

fn dec_to_snafu(mut dec: u64) -> Result<String> {
    let mut snafu = VecDeque::new();
    let mut pow = 1;
    while dec > 0 {
        let next_pow = 5 * pow;
        let val = (dec % next_pow) / pow;
        snafu.push_front(match val {
            0 => '0',
            1 => {
                dec -= 1 * pow;
                '1'
            }
            2 => {
                dec -= 2 * pow;
                '2'
            }
            3 => {
                dec += 2 * pow;
                '='
            }
            4 => {
                dec += pow;
                '-'
            }
            _ => bail!("accidentaly broke maths - please come back later"),
        });
        pow = next_pow;
    }
    Ok(snafu.into_iter().collect())
}

fn parse_input(input: String) -> Result<Vec<String>> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut snafu_numbers = Vec::new();

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        snafu_numbers.push(line.trim().to_owned());
        line.clear();
    }

    Ok(snafu_numbers)
}

#[cfg(test)]
mod tests {
    use crate::task_solver::day_25::{dec_to_snafu, snafu_to_decimal};

    #[test]
    fn snafu_to_dec_test() {
        let mut snafu = "1=-0-2";
        assert_eq!(snafu_to_decimal(snafu).unwrap(), 1747);

        snafu = "12111";
        assert_eq!(snafu_to_decimal(snafu).unwrap(), 906);

        snafu = "2=0=";
        assert_eq!(snafu_to_decimal(snafu).unwrap(), 198);
    }

    #[test]
    fn dec_to_snafu_test() {
        let mut dec = 1;
        assert_eq!(dec_to_snafu(dec).unwrap(), "1");

        dec = 5;
        assert_eq!(dec_to_snafu(dec).unwrap(), "10");

        dec = 2022;
        assert_eq!(dec_to_snafu(dec).unwrap(), "1=11-2");
    }
}
