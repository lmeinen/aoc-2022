use anyhow::{anyhow, Context, Ok, Result};

use log::{debug, info};
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use super::util::{self, SortedList};

pub fn solve(task: u8, input: String) -> Result<()> {
    let mut parser = MonkeyParser::init(input, task).context("failed to instantiate parser")?;

    debug!("parsed {} monkeys", parser.monkey_map.len(),);

    let n_rounds = match task {
        1 => 20,
        2 => 10000,
        _ => return Err(anyhow!("task doesn't exist!")),
    };

    let monkey_business = parser
        .nth(n_rounds - 1)
        .expect("failed to iterate over 20 rounds")
        .fold(1u64, &|x, y| *x as u64 * y);

    info!(
        "level of monkey business after {} rounds: {}",
        n_rounds, monkey_business
    );

    Ok(())
}

struct Monkey {
    item_list: Vec<u32>,
    operation: Box<dyn Fn(u32) -> u32>,
    test: u32,
    if_true: u32,
    if_false: u32,
    no_inspections: u32,
}

struct MonkeyParser {
    monkey_map: HashMap<u32, Monkey>,
    worry_congruences: HashMap<u32, HashMap<u32, u32>>, // { item_id -> { mod_val -> curr_val } }
    is_task_1: bool,
    round: u32,
}

fn apply_operation(
    worry_congruences: &mut HashMap<u32, HashMap<u32, u32>>,
    item_id: u32,
    operation: &dyn Fn(u32) -> u32,
    is_task_1: bool,
) {
    let item_congruences = worry_congruences.get_mut(&item_id).unwrap();
    for (m, val) in item_congruences.iter_mut() {
        let mut op_val = operation(*val);
        if is_task_1 {
            debug!("finding mod inv of {} for {}", 3, *m as i32);
            let mod_inv = modinverse::modinverse(3, *m as i32).unwrap();
            debug!("mod inv: {}", mod_inv);
            op_val = op_val * mod_inv as u32;
        }
        *val = op_val % m;
    }
}

impl MonkeyParser {
    fn init(input: String, task: u8) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let mut in_reader = BufReader::new(in_file);
        let mut line = String::new();

        let re_monkey = Regex::new(r"Monkey (?P<monkey_no>\d+):").unwrap();
        let re_starting_items =
            Regex::new(r"Starting items: (?P<start_list>(?:\d+(?:, )?)+)").unwrap();
        let re_operation =
            Regex::new(r"Operation: new = old (?P<op>[+*]) (?P<val>old|\d+)").unwrap();
        let re_test = Regex::new(r"Test: divisible by (?P<val>\d+)").unwrap();
        let re_if_true = Regex::new(r"If true: throw to monkey (?P<monkey_no>\d+)").unwrap();
        let re_if_false = Regex::new(r"If false: throw to monkey (?P<monkey_no>\d+)").unwrap();

        let mut monkey_map = HashMap::new();

        // can construct worry_congruences map after parsing monkeys
        let mut item_count = 0u32;
        let mut item_vals = HashMap::new(); // { item_id -> initial worry_level }
        let mut mod_vals = Vec::new(); // [ test values ]

        loop {
            while in_reader
                .read_line(&mut line)
                .expect("failed to read line in input file")
                != 0
                && !(re_monkey.is_match(&line)
                    && re_starting_items.is_match(&line)
                    && re_operation.is_match(&line)
                    && re_test.is_match(&line)
                    && re_if_true.is_match(&line)
                    && re_if_false.is_match(&line))
            {}

            if line.trim().is_empty() {
                return Ok(MonkeyParser {
                    monkey_map,
                    worry_congruences: item_vals
                        .drain()
                        .map(|(item_id, init_val): (u32, u32)| {
                            (
                                item_id,
                                mod_vals.iter().map(|m: &u32| (*m, init_val % m)).collect(),
                            )
                        })
                        .collect(),
                    is_task_1: task == 1,
                    round: 0u32,
                });
            } else if !(re_monkey.is_match(&line)
                && re_starting_items.is_match(&line)
                && re_operation.is_match(&line)
                && re_test.is_match(&line)
                && re_if_true.is_match(&line)
                && re_if_false.is_match(&line))
            {
                return Err(anyhow!("missing monkey data!"));
            } else {
                let monkey_no =
                    util::capture_and_parse(&re_monkey, &line, "monkey_no", &|s: &str| {
                        s.parse::<u32>().expect("failed to parse monkey number")
                    });
                let mut item_list =
                    util::capture_and_parse(&re_starting_items, &line, "start_list", &|s: &str| {
                        s.split(',')
                            .map(|n| {
                                n.trim()
                                    .parse::<u32>()
                                    .expect("failed to parse worry level")
                            })
                            .collect::<Vec<u32>>()
                    });
                for item_val in item_list.iter_mut() {
                    item_vals.insert(item_count, *item_val);
                    *item_val = item_count;
                    item_count += 1;
                }
                let test = util::capture_and_parse(&re_test, &line, "val", &|s: &str| {
                    s.parse::<u32>().expect("failed to parse test number")
                });
                mod_vals.push(test);
                let if_true =
                    util::capture_and_parse(&re_if_true, &line, "monkey_no", &|s: &str| {
                        s.parse::<u32>().expect("failed to parse if_true number")
                    });
                let if_false =
                    util::capture_and_parse(&re_if_false, &line, "monkey_no", &|s: &str| {
                        s.parse::<u32>().expect("failed to parse if_false number")
                    });
                let operation_name =
                    util::capture_and_parse(&re_operation, &line, "op", &|s| s.to_owned());
                let operation_val =
                    util::capture_and_parse(&re_operation, &line, "val", &|s| s.to_owned());
                let monkey = Monkey {
                    item_list,
                    test,
                    if_true,
                    if_false,
                    no_inspections: 0u32,
                    operation: if operation_name == "+" {
                        if operation_val == "old" {
                            Box::new(|n: u32| n + n)
                        } else {
                            let val = operation_val
                                .parse::<u32>()
                                .expect("couldn't parse number from op val");
                            Box::new(move |n: u32| n + val)
                        }
                    } else if operation_name == "*" {
                        if operation_val == "old" {
                            Box::new(|n: u32| n * n)
                        } else {
                            let val = operation_val
                                .parse::<u32>()
                                .expect("couldn't parse number from op val");
                            Box::new(move |n: u32| n * val)
                        }
                    } else {
                        return Err(anyhow!("unknown operation {}", operation_name));
                    },
                };
                if let Some(_) = monkey_map.insert(monkey_no, monkey) {
                    return Err(anyhow!("map contained duplicate monkey!"));
                } else {
                    line.clear();
                }
            }
        }
    }
}

impl Iterator for MonkeyParser {
    type Item = SortedList<u32>; // [no of item inspections per monkey in this round]

    fn next(&mut self) -> Option<Self::Item> {
        self.round += 1;
        debug!("Running round {}", self.round);

        // visit monkeys in order
        let mut monkey_order = self.monkey_map.keys().cloned().collect::<Vec<u32>>();
        monkey_order.sort_unstable();

        // cannot hold multiple mutable references to monkey_map --> tmp store for moved items
        let mut passed_items: HashMap<u32, Vec<u32>> = HashMap::new();

        let mut inspection_list = util::SortedList::new(2);

        for monkey_no in monkey_order.iter() {
            let active_monkey = self
                .monkey_map
                .get_mut(monkey_no)
                .expect("monkey doesn't exist!");
            let item_list = &mut active_monkey.item_list;
            let mut passed_item_list = passed_items.remove(monkey_no).unwrap_or(vec![]);
            let item_list_full = item_list.drain(..).chain(passed_item_list.drain(..));

            for item_id in item_list_full {
                active_monkey.no_inspections += 1;
                apply_operation(
                    &mut self.worry_congruences,
                    item_id,
                    &active_monkey.operation,
                    self.is_task_1,
                );
                let new_owner;
                if *self
                    .worry_congruences
                    .get(&item_id)
                    .unwrap()
                    .get(&active_monkey.test)
                    .unwrap()
                    == 0u32
                {
                    new_owner = active_monkey.if_true;
                } else {
                    new_owner = active_monkey.if_false;
                }

                if passed_items.contains_key(&new_owner) {
                    passed_items.get_mut(&new_owner).unwrap().push(item_id);
                } else {
                    passed_items.insert(new_owner, vec![item_id]);
                }
            }

            inspection_list.insert(active_monkey.no_inspections);
        }

        for (monkey_no, item_list) in passed_items.iter_mut() {
            self.monkey_map
                .get_mut(monkey_no)
                .unwrap()
                .item_list
                .append(item_list);
        }

        Some(inspection_list)
    }
}
