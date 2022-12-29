use anyhow::{bail, Context, Result};

use itertools::Itertools;
use log::info;
use regex::Regex;
use std::{
    cell::RefCell,
    collections::{BTreeSet, HashMap, HashSet, VecDeque},
    fs::File,
    io::{BufRead, BufReader},
    rc::Rc,
};

use super::util;

type NodeRef = Rc<RefCell<Node>>;
type State = (u32, String, BTreeSet<String>);

#[derive(Debug)]
struct Node {
    id: String,
    flow_rate: u32,
    paths: Vec<(u32, NodeRef)>,
}

fn max_score(
    node: &NodeRef,
    t: u32,
    closed: &mut BTreeSet<String>,
    visited: &mut HashMap<State, u32>,
) -> u32 {
    let state = (t, node.borrow().id.to_owned(), closed.clone());
    if visited.contains_key(&state) {
        return visited[&state];
    }

    let mut score = 0;
    for (path_len, next) in node.borrow().paths.iter() {
        let next_id = next.borrow().id.to_owned();
        if t > *path_len + 1 && closed.remove(&next_id) {
            let next_score = (t - path_len - 1) * next.borrow().flow_rate
                + max_score(next, t - path_len - 1, closed, visited);
            closed.insert(next_id);
            if next_score > score {
                score = next_score;
            }
        }
    }
    visited.insert(state, score.clone());
    score
}

pub fn solve(task: u8, input: String) -> Result<()> {
    let (start, mut id_list) = init(input).context("failed to instantiate parser")?;
    let score = match task {
        1 => max_score(&start, 30, &mut id_list, &mut HashMap::new()),
        2 => {
            let mut visited_states = HashMap::new();
            let mut visited_sets = HashSet::new();
            let mut max = 0u32;
            for mut subset in id_list
                .iter()
                .powerset()
                .map(|subset| subset.into_iter().cloned().collect::<BTreeSet<String>>())
            {
                if !visited_sets.contains(&subset) {
                    let mut complement = id_list.difference(&subset).cloned().collect();
                    let score = max_score(&start, 26, &mut complement, &mut visited_states)
                        + max_score(&start, 26, &mut subset, &mut visited_states);
                    visited_sets.insert(subset);
                    visited_sets.insert(complement);
                    if score > max {
                        max = score;
                    }
                }
            }
            max
        }
        _ => bail!("task doesn't exist"),
    };
    info!("max released pressure: {}", score);
    Ok(())
}

fn init(input: String) -> Result<(NodeRef, BTreeSet<String>)> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut valve_system = HashMap::new();
    let mut connections = HashMap::new();

    let re_sensor = Regex::new(r"Valve (?P<id>[A-Z]{2}) has flow rate=(?P<flow_rate>\d+); tunnel(?:s)? lead(?:s)? to valve(?:s)? (?P<tunnels>([A-Z]{2}(:?, )?)+)").unwrap();
    while in_reader
        .read_line(&mut line)
        .context("Failed to read line in input file")?
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
        connections.insert(id.to_owned(), tunnels);
        if id == "AA" || flow_rate > 0 {
            valve_system.insert(
                id.to_owned(),
                Rc::new(RefCell::new(Node {
                    id: id.to_owned(),
                    flow_rate,
                    paths: vec![],
                })),
            );
        }
        line.clear();
    }

    Ok((
        connect(&valve_system, &connections),
        valve_system.keys().cloned().collect(),
    ))
}

fn connect(
    valve_system: &HashMap<String, NodeRef>,
    connections: &HashMap<String, Vec<String>>,
) -> NodeRef {
    for id in valve_system.keys() {
        connect_id(valve_system, connections, id);
    }
    Rc::clone(&valve_system["AA"])
}

fn connect_id(
    valve_system: &HashMap<String, NodeRef>,
    connections: &HashMap<String, Vec<String>>,
    from_id: &str,
) {
    let node = &valve_system[from_id];
    let mut to_visit = VecDeque::new();
    let mut visited = HashSet::new();
    to_visit.push_back((0, from_id));
    while let Some((path_len, to_id)) = to_visit.pop_front() {
        visited.insert(to_id);
        if to_id != from_id && valve_system.contains_key(to_id) {
            node.borrow_mut()
                .paths
                .push((path_len, Rc::clone(&valve_system[to_id])));
        }
        for id in connections[to_id].iter() {
            if !visited.contains(id as &str) {
                to_visit.push_back((path_len + 1, id));
            }
        }
    }
}
