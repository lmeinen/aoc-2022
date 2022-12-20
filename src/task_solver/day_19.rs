use anyhow::{bail, Context, Result};

use log::{debug, info};
use regex::Regex;

use std::{
    cmp,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use super::util;

#[derive(Debug)]
struct Blueprint {
    id: u32,
    robot_costs: Vec<Vec<u32>>, // cost in (ore, clay, obsidian)
}

impl Blueprint {
    fn max_geodes(
        &self,
        state: State,
        max_time: u32,
        seen_states: &mut HashMap<State, u32>,
    ) -> u32 {
        if let Some(max_score) = seen_states.get(&state) {
            *max_score
        } else {
            let mut new_states = Vec::new();
            for robot in 0..self.robot_costs.len() {
                if let Some(new_state) = state.build_robot(self, robot, max_time) {
                    new_states.push(new_state);
                }
            }

            let max_score = if new_states.len() == 0 {
                // can't build any more robots
                let final_state = state.advance_by(max_time - state.time);
                *final_state
                    .resources
                    .last()
                    .expect("final state resources are empty")
            } else {
                let mut max_score = 0u32;
                while let Some(s) = new_states.pop() {
                    max_score = cmp::max(max_score, self.max_geodes(s, max_time, seen_states));
                }
                max_score
            };

            seen_states.insert(state, max_score);

            max_score
        }
    }

    fn max_required_resource(&self, robot: usize) -> u32 {
        self.robot_costs.iter().map(|c| c[robot]).max().unwrap()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct State {
    time: u32,
    resources: Vec<u32>,
    robots: Vec<u32>,
}

impl State {
    fn advance_by(&self, t: u32) -> State {
        let mut new_resources = Vec::new();
        for i in 0..self.robots.len() {
            new_resources.push(self.resources[i] + self.robots[i] * t);
        }
        new_resources.push(self.resources[self.robots.len()]);
        State {
            time: self.time + t,
            resources: new_resources,
            robots: self.robots.clone(),
        }
    }

    fn is_useless_branch(&self, blueprint: &Blueprint, robot: usize, max_time: u32) -> bool {
        let rem_time = max_time - self.time;
        robot < self.robots.len()
            && self.robots[robot] * rem_time + self.resources[robot]
                >= blueprint.max_required_resource(robot) * rem_time
    }

    /// advances time until the robot has been built (i.e. including gathering the required resources) - returns None if the robots required to gather the resources haven't been built yet
    fn build_robot(&self, blueprint: &Blueprint, robot: usize, max_time: u32) -> Option<State> {
        if self.is_useless_branch(blueprint, robot, max_time) {
            None
        } else {
            let robot_costs = &blueprint.robot_costs[robot];
            let mut max_timesteps = 0;
            for (i, c) in robot_costs.iter().enumerate() {
                let required = (*c as i32) - (self.resources[i] as i32);
                if required > 0 {
                    if self.robots[i] == 0 {
                        return None;
                    } else {
                        let req_timesteps = (required as f64 / self.robots[i] as f64).ceil() as u32;
                        max_timesteps = cmp::max(max_timesteps, req_timesteps);
                    }
                }
            }

            let mut new_state = self.advance_by(max_timesteps + 1);
            robot_costs
                .iter()
                .enumerate()
                .for_each(|(i, c)| new_state.resources[i] -= c);

            if new_state.time <= max_time {
                if robot < self.robots.len() {
                    new_state.robots[robot] += 1;
                } else {
                    new_state.resources[robot] += max_time - new_state.time;
                }
                Some(new_state)
            } else {
                None
            }
        }
    }
}

pub fn solve(_task: u8, input: String) -> Result<()> {
    let blueprint_list = parse_input(input).context("failed to parse input")?;

    let (max_time, mut quality_level, num_blueprints) = match _task {
        1 => (24, 0u32, blueprint_list.len()),
        2 => (32, 1u32, 3),
        _ => bail!("task doesn't exist!"),
    };

    for i in 0..cmp::min(blueprint_list.len(), num_blueprints) {
        let blueprint = &blueprint_list[i];
        debug!("considering blueprint {:?}", blueprint);
        let max_geodes = blueprint.max_geodes(
            State {
                time: 0u32,
                resources: vec![0, 0, 0, 0],
                robots: vec![1, 0, 0],
            },
            max_time,
            &mut HashMap::new(),
        );

        info!(
            "largest number of geodes you could open with blueprint {} in {} minutes is: {}",
            blueprint.id, max_time, max_geodes
        );

        match _task {
            1 => quality_level += blueprint.id * max_geodes,
            2 => quality_level *= max_geodes,
            _ => bail!("task doesn't exist"),
        }
    }

    info!("quality level: {}", quality_level);

    Ok(())
}

fn parse_input(input: String) -> Result<Vec<Blueprint>> {
    // open input file
    let in_file = File::open(input).context(format!("Failed to read input"))?;

    // uses a reader buffer
    let mut in_reader = BufReader::new(in_file);
    let mut line = String::new();

    let mut blueprint_list = Vec::new();

    let re_blueprint = Regex::new(r"Blueprint (?P<blueprint_id>\d+):").unwrap();
    let re_ore_robot = Regex::new(r"Each ore robot costs (?P<ore_robot>\d+) ore.").unwrap();
    let re_clay_robot = Regex::new(r"Each clay robot costs (?P<clay_robot>\d+) ore.").unwrap();
    let re_obsidian_robot = Regex::new(r"Each obsidian robot costs (?P<obsidian_robot_ore>\d+) ore and (?P<obsidian_robot_clay>\d+) clay.").unwrap();
    let re_geode_robot = Regex::new(r"Each geode robot costs (?P<geode_robot_ore>\d+) ore and (?P<geode_robot_obsidian>\d+) obsidian.").unwrap();

    while in_reader
        .read_line(&mut line)
        .expect("Failed to read input file")
        != 0
        && line != "\n"
    {
        if !re_blueprint.is_match(&line)
            || !re_ore_robot.is_match(&line)
            || !re_clay_robot.is_match(&line)
            || !re_obsidian_robot.is_match(&line)
            || !re_geode_robot.is_match(&line)
        {
            bail!("line didn't contain full blueprint");
        }

        let parse_int = &|s: &str| {
            s.parse::<u32>()
                .expect(&format!("failed to parse int from {}", s))
        };
        let blueprint_id = util::capture_and_parse(&re_blueprint, &line, "blueprint_id", parse_int);
        let ore_robot = util::capture_and_parse(&re_ore_robot, &line, "ore_robot", parse_int);
        let clay_robot = util::capture_and_parse(&re_clay_robot, &line, "clay_robot", parse_int);
        let obsidian_robot_ore =
            util::capture_and_parse(&re_obsidian_robot, &line, "obsidian_robot_ore", parse_int);
        let obsidian_robot_clay =
            util::capture_and_parse(&re_obsidian_robot, &line, "obsidian_robot_clay", parse_int);
        let geode_robot_ore =
            util::capture_and_parse(&re_geode_robot, &line, "geode_robot_ore", parse_int);
        let geode_robot_obsidian =
            util::capture_and_parse(&re_geode_robot, &line, "geode_robot_obsidian", parse_int);

        blueprint_list.push(Blueprint {
            id: blueprint_id,
            robot_costs: vec![
                vec![ore_robot, 0, 0],
                vec![clay_robot, 0, 0],
                vec![obsidian_robot_ore, obsidian_robot_clay, 0],
                vec![geode_robot_ore, 0, geode_robot_obsidian],
            ],
        });
        line.clear();
    }

    Ok(blueprint_list)
}
