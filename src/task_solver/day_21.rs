use anyhow::{bail, Context, Result};

use log::info;
use regex::Regex;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use super::util;

#[derive(Clone)]
enum Operation {
    ADD(String, String),
    SUB(String, String),
    MUL(String, String),
    DIV(String, String),
}

impl Operation {
    fn lhs(&self) -> &str {
        self.operands().0
    }

    fn rhs(&self) -> &str {
        self.operands().1
    }

    fn operands(&self) -> (&str, &str) {
        match self {
            Self::ADD(x, y) => (x, y),
            Self::SUB(x, y) => (x, y),
            Self::MUL(x, y) => (x, y),
            Self::DIV(x, y) => (x, y),
        }
    }

    fn compute(&self, lhs: i64, rhs: i64) -> i64 {
        match self {
            Operation::ADD(_, _) => lhs + rhs,
            Operation::SUB(_, _) => lhs - rhs,
            Operation::MUL(_, _) => lhs * rhs,
            Operation::DIV(_, _) => lhs / rhs,
        }
    }

    fn invert(&self, res: i64, lhs: Option<i64>, rhs: Option<i64>) -> i64 {
        match self {
            Operation::ADD(_, _) => {
                if let Some(x) = lhs {
                    res - x
                } else {
                    res - rhs.unwrap()
                }
            }
            Operation::SUB(_, _) => {
                if let Some(x) = lhs {
                    x - res
                } else {
                    res + rhs.unwrap()
                }
            }
            Operation::MUL(_, _) => {
                if let Some(x) = lhs {
                    res / x
                } else {
                    res / rhs.unwrap()
                }
            }
            Operation::DIV(_, _) => {
                if let Some(x) = lhs {
                    x / res
                } else {
                    res * rhs.unwrap()
                }
            }
        }
    }
}

struct Monkey {
    value: Option<i64>,
    op: Option<Operation>,
    lhs: Option<i64>,
    rhs: Option<i64>,
}

fn get_val(
    monkey_map: &mut HashMap<String, Monkey>,
    monkey_id: &str,
    need_human: bool,
) -> Option<i64> {
    if need_human && monkey_id == "humn" {
        None
    } else {
        let m = monkey_map.get(monkey_id).unwrap();
        if let Some(val) = m.value {
            Some(val)
        } else {
            let op = m.op.clone().unwrap();
            let lhs = get_val(monkey_map, op.lhs(), need_human);
            let rhs = get_val(monkey_map, op.rhs(), need_human);
            let m = monkey_map.get_mut(monkey_id).unwrap();
            if lhs.is_some() && rhs.is_some() {
                // cache result
                let val = Some(op.compute(lhs.unwrap(), rhs.unwrap()));
                m.value = val;
                val
            } else {
                m.lhs = lhs;
                m.rhs = rhs;
                None
            }
        }
    }
}

fn solve_chain(monkey_map: &HashMap<String, Monkey>, monkey_id: &str, res: i64) -> i64 {
    if monkey_id == "humn" {
        res
    } else {
        let m = monkey_map.get(monkey_id).unwrap();
        let op = m.op.clone().unwrap();
        let next_res = op.invert(res, m.lhs, m.rhs);
        let next_m = if m.lhs.is_none() { op.lhs() } else { op.rhs() };
        solve_chain(monkey_map, next_m, next_res)
    }
}

pub fn solve(task: u8, input: String) -> Result<()> {
    let mut monkey_map = parse_input(input).context("failed to parse input")?;

    let root_val = get_val(&mut monkey_map, "root", task == 2);

    if let Some(val) = root_val {
        info!("computed root value: {}", val);
    } else {
        let root = monkey_map.get("root").unwrap();
        let (m, res) = if root.lhs.is_none() {
            (root.op.as_ref().unwrap().lhs(), root.rhs.unwrap())
        } else {
            (root.op.as_ref().unwrap().rhs(), root.lhs.unwrap())
        };
        let humn_val = solve_chain(&monkey_map, m, res);
        info!("number to yell to pass root's equality test: {}", humn_val);
    }

    Ok(())
}

fn parse_input(input: String) -> Result<HashMap<String, Monkey>> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut monkeys = HashMap::new();
    let re_monkey = Regex::new(r"(?P<monkey_id>[a-z]{4}): (?:(?P<val>\d+)|(?P<lhs>[a-z]{4}) (?P<op>[+\-*\\/]) (?P<rhs>[a-z]{4}))").unwrap();

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        let monkey_id = util::capture_and_parse(&re_monkey, &line, "monkey_id", &|s| s.to_owned());
        let monkey = if let Some(val) =
            util::try_capture_and_parse(&re_monkey, &line, "val", &|s| {
                s.parse::<i64>().expect("failed to parse value to i64")
            }) {
            Monkey {
                value: Some(val),
                op: None,
                lhs: None,
                rhs: None,
            }
        } else {
            let lhs = util::capture_and_parse(&re_monkey, &line, "lhs", &|s| s.to_owned());
            let rhs = util::capture_and_parse(&re_monkey, &line, "rhs", &|s| s.to_owned());
            let op =
                util::capture_and_parse(&re_monkey, &line, "op", &|s| s.chars().next().unwrap());
            let monkey_op = match op {
                '+' => Operation::ADD(lhs, rhs),
                '-' => Operation::SUB(lhs, rhs),
                '*' => Operation::MUL(lhs, rhs),
                '/' => Operation::DIV(lhs, rhs),
                _ => bail!("unknown operation type: {}", op),
            };
            Monkey {
                value: None,
                op: Some(monkey_op),
                lhs: None,
                rhs: None,
            }
        };

        monkeys.insert(monkey_id.to_owned(), monkey);

        line.clear();
    }

    Ok(monkeys)
}
