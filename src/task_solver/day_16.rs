use anyhow::{bail, Context, Result};

use log::{debug, info};
use regex::Regex;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use super::util;

pub fn solve(_task: u8, input: String) -> Result<()> {
    let (tunnel_system, mut valve_states) = init(input).context("failed to instantiate parser")?;

    debug!("parsed tunnel system {:#?}", tunnel_system);

    // at each valve, we either open it or move on --> two states CHILLING and OPENED
    // a state consists of a list of open valves, and the sum of released pressure up to that point
    // initial state is {[], 0} at AA, None at all others
    // for a valve, compute the next CHILLING state by looking at CHILLING and OPENED of connect valves
    //      --> take the list of valves with the largest sum of released pressure
    // for a valve, compute the next OPENED state by looking at its CHILLING state

    let total_time = 30;

    for t in 0..total_time {
        let mut next_state = HashMap::new();
        for (id, valve) in tunnel_system.iter() {
            if let Some(curr_state) = valve_states.get(id) {
                let mut curr_best = curr_state.clone();

                // compute state for staying at / opening current valve
                if let Some((steps, open_valves, s)) = &mut curr_best.as_mut() {
                    *s += flow_rate_sum(open_valves, &tunnel_system);
                    if !open_valves.contains(id) {
                        steps.push(Operation::Open(id.to_owned()));
                        open_valves.push(id.to_owned());
                    } else {
                        steps.push(Operation::Null);
                    }
                }

                // score that state
                let mut curr_score = get_score(&curr_best, total_time - t, &tunnel_system);

                for (_, other_state) in valve_states
                    .iter()
                    .filter(|(id, _)| valve.tunnels.contains(id))
                {
                    // compute state for move from other valve
                    let other_state_next = if let Some((steps, open_valves, s)) = other_state {
                        let mut steps = steps.to_vec();
                        steps.push(Operation::Move(id.to_owned()));
                        Some((
                            steps,
                            open_valves.clone(),
                            s + flow_rate_sum(open_valves, &tunnel_system),
                        ))
                    } else {
                        None
                    };

                    // score that state
                    let other_score = get_score(&other_state_next, total_time - t, &tunnel_system);

                    if match (curr_score, other_score) {
                        (None, Some(_)) => true,
                        (Some(x), Some(y)) => x < y,
                        _ => false,
                    } {
                        // pick better one
                        curr_best = other_state_next;
                        curr_score = other_score;
                    }
                }
                debug!(
                    "Minute {} - Valve {}: {:?} --> {:?}",
                    t,
                    id,
                    curr_state.as_ref().map(|(_, _, s)| s),
                    curr_best.as_ref().map(|(_, _, s)| s)
                );
                next_state.insert(id.to_owned(), curr_best);
            } else {
                bail!("no state for valve {}", id);
            }
        }
        valve_states = next_state;
    }

    let mut max_state_opt = None;
    for (_, s) in valve_states {
        if let Some((_, _, v)) = s {
            if let Some((_, _, m)) = max_state_opt {
                if v > m {
                    max_state_opt = s;
                }
            } else {
                max_state_opt = s;
            }
        }
    }

    let max_state = max_state_opt.unwrap();
    debug_output(&max_state.0, total_time, &tunnel_system);

    info!("max released pressure: {}", max_state.2);

    Ok(())
}

#[derive(Debug, Clone)]
enum Operation {
    Null,
    Open(String),
    Move(String),
}

#[derive(Debug)]
struct Valve {
    flow_rate: u32,
    tunnels: Vec<String>,
}

type TunnelSystem = HashMap<String, Valve>;

type State = Option<(Vec<Operation>, Vec<String>, u32)>;
type ValveStates = HashMap<String, State>;

fn init(input: String) -> Result<(TunnelSystem, ValveStates)> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut valve_system = HashMap::new();

    let re_sensor = Regex::new(r"Valve (?P<id>[A-Z]{2}) has flow rate=(?P<flow_rate>\d+); tunnel(?:s)? lead(?:s)? to valve(?:s)? (?P<tunnels>([A-Z]{2}(:?, )?)+)").unwrap();
    while in_reader
        .read_line(&mut line)
        .expect("Failed to read line in input file")
        != 0
        && line != "\n"
    {
        let id = util::capture_and_parse(&re_sensor, &line, "id", &|s| s.to_owned());
        let flow_rate = util::capture_and_parse(&re_sensor, &line, "flow_rate", &|s| {
            s.parse::<u32>().expect("failed to parse flow_rate")
        });
        let tunnels = util::capture_and_parse(&re_sensor, &line, "tunnels", &|s| {
            s.split(", ").map(|s| s.to_owned()).collect::<Vec<String>>()
        });
        valve_system.insert(id, Valve { flow_rate, tunnels });
        line.clear();
    }

    let mut valve_states = HashMap::new();
    for id in valve_system.keys() {
        valve_states.insert(
            id.to_owned(),
            if id == "AA" {
                Some((vec![], vec![], 0u32))
            } else {
                None
            },
        );
    }

    Ok((valve_system, valve_states))
}

fn flow_rate_sum(s: &Vec<String>, tunnel_system: &TunnelSystem) -> u32 {
    let mut c = 0u32;
    for id in s {
        c += tunnel_system.get(id).unwrap().flow_rate;
    }
    c
}

fn get_score(s: &State, time_remaining: u32, tunnel_system: &TunnelSystem) -> Option<u32> {
    if let Some((_, open_valves, curr_sum)) = s {
        let sum = flow_rate_sum(open_valves, tunnel_system);
        Some(curr_sum + sum * time_remaining)
    } else {
        None
    }
}

fn debug_output(ops: &Vec<Operation>, total_time: u32, tunnel_system: &TunnelSystem) {
    let mut open_valves = Vec::new();
    for t in 0..total_time {
        debug!("== Minute {} ==", t + 1);
        if open_valves.is_empty() {
            debug!("No valves are open.");
        } else if open_valves.len() == 1 {
            debug!(
                "Valve {} is open, releasing {} pressure.",
                open_valves[0],
                flow_rate_sum(&open_valves, tunnel_system)
            );
        } else {
            let line = open_valves.join(", ");
            debug!(
                "Valves {} are open, releasing {} pressure.",
                line,
                flow_rate_sum(&open_valves, tunnel_system)
            );
        }
        let op = &ops[t as usize];
        match op {
            Operation::Null => {}
            Operation::Move(id) => debug!("You move to valve {}.", id),
            Operation::Open(id) => {
                debug!("You open valve {}.", id);
                open_valves.push(id.to_owned());
            }
        }
        debug!("");
    }
}
