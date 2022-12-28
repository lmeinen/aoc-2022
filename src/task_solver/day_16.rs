use anyhow::{bail, Context, Result};

use itertools::Itertools;
use log::{debug, info};
use petgraph::{algo::dijkstra, Directed, Graph};
use regex::Regex;
use std::{
    cmp,
    collections::{HashMap, HashSet},
    fs::File,
    hash::Hash,
    io::{BufRead, BufReader},
};

use super::util;

#[derive(Debug)]
struct Valve {
    flow_rate: u32,
    tunnels: Vec<String>,
}

#[derive(Debug, Clone)]
struct ValvePruned {
    flow_rate: u32,
    valve_distances: HashMap<String, u32>, // shortest distances to all valves with non-zero flow rates
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct State {
    time: u32,
    curr_valve: String,
    open_valves: Vec<String>,
}

impl State {
    fn open_valve(&self, dist: u32, valve: String, max_time: u32) -> Option<State> {
        if self.time + dist + 1 < max_time && !self.open_valves.contains(&valve) {
            let mut open_valves = self.open_valves.clone();
            open_valves.push(valve.to_owned());
            Some(State {
                time: self.time + dist + 1,
                curr_valve: valve,
                open_valves,
            })
        } else {
            None
        }
    }

    fn current_flow(&self, tunnel_system: &HashMap<String, ValvePruned>) -> u32 {
        self.open_valves
            .iter()
            .fold(0u32, |acc, v| acc + tunnel_system.get(v).unwrap().flow_rate)
    }

    fn timestep(&self) -> State {
        State {
            time: self.time + 1,
            curr_valve: self.curr_valve.clone(),
            open_valves: self.open_valves.clone(),
        }
    }

    fn init() -> Self {
        State {
            time: 0u32,
            curr_valve: "AA".to_owned(),
            open_valves: vec![],
        }
    }
}

type TunnelSystem = HashMap<String, Valve>;

fn complement_valves(
    tunnel_system: &HashMap<String, ValvePruned>,
    valves: &HashSet<String>,
) -> HashSet<String> {
    debug!("complement of set: {:?}", valves);
    let complement = tunnel_system
        .keys()
        .filter(|&v| !valves.contains(v))
        .cloned()
        .collect();
    debug!("is: {:?}", complement);
    complement
}

fn find_max_score(
    tunnel_system: &HashMap<String, ValvePruned>,
    state: State,
    seen_states: &mut HashMap<State, u32>,
    max_time: u32,
    allowed_valves: &HashSet<String>,
    with_elephants: bool,
) -> u32 {
    if with_elephants {
        debug!("considering valve set {:?}", allowed_valves);
        find_max_score(
            tunnel_system,
            State::init(),
            seen_states,
            max_time,
            allowed_valves,
            false,
        ) + find_max_score(
            tunnel_system,
            State::init(),
            seen_states,
            max_time,
            &complement_valves(tunnel_system, &allowed_valves),
            false,
        )
    } else if let Some(max_score) = seen_states.get(&state) {
        *max_score
    } else {
        let max_score = if state.time == max_time {
            0u32
        } else {
            let mut max_score = 0u32;

            debug!("t {} - got here", state.time);

            let mut next_states = Vec::new();
            for (valve_id, dist) in tunnel_system
                .get(&state.curr_valve)
                .unwrap()
                .valve_distances
                .iter()
                .filter(|(k, _)| allowed_valves.contains(*k))
            {
                if let Some(next_state) = state.open_valve(*dist, valve_id.to_owned(), max_time) {
                    next_states.push(next_state);
                }
            }

            if next_states.is_empty() {
                // all reachable valves are open
                max_score = state.current_flow(tunnel_system)
                    + find_max_score(
                        tunnel_system,
                        state.timestep(),
                        seen_states,
                        max_time,
                        allowed_valves,
                        with_elephants,
                    );
            } else {
                while let Some(next_state) = next_states.pop() {
                    let time_passed = next_state.time - state.time;
                    let score = find_max_score(
                        tunnel_system,
                        next_state,
                        seen_states,
                        max_time,
                        allowed_valves,
                        with_elephants,
                    );
                    max_score = cmp::max(
                        max_score,
                        score + time_passed * state.current_flow(tunnel_system),
                    );
                }
            }

            max_score
        };
        seen_states.insert(state, max_score);
        max_score
    }
}

pub fn solve(task: u8, input: String) -> Result<()> {
    let tunnel_system = init(input).context("failed to instantiate parser")?;

    let ts = prune_tunnelsystem(tunnel_system).context("failed to prune tunnel system")?;

    let (max_time, with_elephants, allowed_valves_sets) = match task {
        1 => (
            30,
            false,
            vec![ts.keys().cloned().collect::<HashSet<String>>()],
        ),
        2 => (26, true, valve_subsets(&ts)),
        _ => bail!("task doesn't exist!"),
    };

    let mut seen_states = HashMap::new();
    debug!("subsets: {:?}", allowed_valves_sets);

    let max_score = allowed_valves_sets
        .iter()
        .map(|subset| {
            find_max_score(
                &ts,
                State::init(),
                &mut seen_states,
                max_time,
                &subset,
                with_elephants,
            )
        })
        .max()
        .unwrap();

    info!("max released pressure: {}", max_score);

    Ok(())
}

fn init(input: String) -> Result<TunnelSystem> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut valve_system = HashMap::new();

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
        valve_system.insert(id, Valve { flow_rate, tunnels });
        line.clear();
    }

    Ok(valve_system)
}

fn valve_subsets(tunnel_system: &HashMap<String, ValvePruned>) -> Vec<HashSet<String>> {
    let valves = tunnel_system.keys().cloned().collect::<Vec<String>>();
    let num_valves = valves.len();
    (1..num_valves / 2 + 1)
        .flat_map(|l| valves.iter().combinations(l))
        .map(|valve_vec| valve_vec.into_iter().cloned().collect::<HashSet<String>>())
        .collect::<Vec<HashSet<String>>>()
}

fn prune_tunnelsystem(tunnel_system: TunnelSystem) -> Result<HashMap<String, ValvePruned>> {
    let mut graph = Graph::<String, (), Directed>::new();

    let mut new_tunnel_system = HashMap::new();

    // initialize graph
    let mut valve_to_index = HashMap::new();
    for valve_id in tunnel_system.keys() {
        valve_to_index.insert(valve_id, graph.add_node(valve_id.to_owned()));
    }

    for (valve_id, valve) in tunnel_system.iter() {
        let from_id = valve_to_index.get(valve_id).unwrap();
        for neighbour in valve.tunnels.iter() {
            let to_id = valve_to_index.get(neighbour).unwrap();
            graph.add_edge(*from_id, *to_id, ());
        }
    }

    for (valve_id, valve) in tunnel_system.iter() {
        let from_id = valve_to_index.get(valve_id).unwrap();
        if valve.flow_rate > 0 || valve_id == "AA" {
            let mut new_valve = ValvePruned {
                flow_rate: valve.flow_rate,
                valve_distances: HashMap::new(),
            };
            let sp_map = dijkstra(&graph, *from_id, None, |_| 1);
            for (to_id, len) in sp_map {
                let valve_to_id = &graph[to_id];
                if len > 0 && tunnel_system.get(valve_to_id).unwrap().flow_rate > 0 {
                    new_valve
                        .valve_distances
                        .insert(valve_to_id.to_owned(), len);
                }
            }
            new_tunnel_system.insert(valve_id.to_owned(), new_valve);
        }
    }

    Ok(new_tunnel_system)
}
