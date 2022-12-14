use anyhow::{anyhow, Context, Result};

use log::{debug, info, warn};

use std::{
    cmp,
    collections::{HashMap, VecDeque},
    fs::File,
    io::{BufRead, BufReader},
    ops::Range,
};

pub fn solve(task: u8, input: String) -> Result<()> {
    let mut rock_structure =
        RockStructure::init(input, task).context("failed to instantiate parser")?;

    debug!("parsed rock structure: {:?}", rock_structure.structures);

    while rock_structure.next().is_some() {}

    rock_structure.draw_cave();

    info!(
        "units of sand that don't flow into the abyss: {}",
        rock_structure.grains.len()
    );

    Ok(())
}

struct RockStructure {
    structures: HashMap<u32, Vec<Range<u32>>>, // obstacle intervals per column
    grains: Vec<(u32, u32)>,                   // list of grains of sand added
    curr_path: VecDeque<u32>, // sequence of columns that specify the current falling path
    has_floor: bool,          // floor or abyss
    min_x: u32,               // for drawing purposes
    max_x: u32,
    max_y: u32, // highest y coordinate
}

impl RockStructure {
    fn init(input: String, task: u8) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let mut in_reader = BufReader::new(in_file);
        let mut line = String::new();

        let mut structures = HashMap::new();
        let mut max_y = 0u32;
        let mut min_x = 500u32;
        let mut max_x = 500u32;

        // parse cave structure
        loop {
            let bytes_read = in_reader
                .read_line(&mut line)
                .expect("Failed to read line in input file");
            if bytes_read == 0 || line == "\n" {
                break; // EOF
            }

            let mut prev_point = None;
            for point in line.trim().split(" -> ") {
                let coordinates = point.split(',').collect::<Vec<&str>>();
                let x_coord = coordinates[0]
                    .parse::<u32>()
                    .context("failed to parse x coordinate")?;
                let y_coord = coordinates[1]
                    .parse::<u32>()
                    .context("failed to parse y coordinate")?;
                max_y = cmp::max(y_coord, max_y);
                min_x = cmp::min(x_coord, min_x);
                max_x = cmp::max(x_coord, max_x);
                if let Some((prev_x, prev_y)) = prev_point {
                    if prev_x == x_coord {
                        // same column
                        Self::add_rocks_internal(
                            &mut structures,
                            x_coord,
                            cmp::min(prev_y, y_coord)..cmp::max(prev_y, y_coord) + 1,
                        )
                        .context("failed to add rocks")?;
                    } else {
                        // same row
                        for x in cmp::min(prev_x, x_coord)..cmp::max(prev_x, x_coord) + 1 {
                            Self::add_rocks_internal(&mut structures, x, y_coord..y_coord + 1)
                                .context("failed to add rocks")?;
                        }
                    }
                }
                prev_point = Some((x_coord, y_coord));
            }
            line.clear();
        }

        // set initial path
        let curr_x = 500;
        let mut curr_path = VecDeque::new();
        curr_path.push_back(curr_x);

        let mut has_floor = false;
        if task == 2 {
            // set cave floor
            has_floor = true;
            max_y = max_y + 2;
        }

        Ok(RockStructure {
            structures,
            grains: Vec::new(),
            curr_path,
            has_floor,
            min_x,
            max_x,
            max_y,
        })
    }

    fn into_the_abyss(&self, x: u32, y: u32) -> bool {
        !self.has_floor
            && if let Some(obstacles) = self.structures.get(&x) {
                obstacles.iter().all(|i| i.end < y)
            } else {
                true
            }
    }

    fn next_move(&self, x: u32, y: u32) -> Option<(u32, u32)> {
        if self.check_pos_internal(x, y + 1) {
            Some((x, y + 1))
        } else if self.check_pos_internal(x - 1, y + 1) {
            Some((x - 1, y + 1))
        } else if self.check_pos_internal(x + 1, y + 1) {
            Some((x + 1, y + 1))
        } else {
            None
        }
    }

    fn check_pos_internal(&self, x: u32, y: u32) -> bool {
        if self.has_floor && y == self.max_y {
            return false;
        }
        if let Some(obstacles) = self.structures.get(&x) {
            for interval in obstacles.iter() {
                if interval.contains(&y) {
                    return false;
                }
            }
        }
        true
    }

    fn add_sand(&mut self, x: u32, y: u32) -> Result<()> {
        self.grains.push((x, y));
        self.min_x = cmp::min(self.min_x, x);
        self.max_x = cmp::max(self.max_x, x);
        if let Some(curr_obstacles) = self.structures.get_mut(&x) {
            // extend obstacle
            for interval in curr_obstacles.iter_mut() {
                if interval.start == y + 1 {
                    info!("adding grain of sand in position ({},{})", x, y);
                    interval.start = y;
                    return Ok(());
                }
            }
        }
        if y + 1 == self.max_y && self.has_floor {
            if let Some(curr_obstacles) = self.structures.get_mut(&x) {
                curr_obstacles.push(y..y + 1);
            } else {
                self.structures.insert(x, vec![y..y + 1]);
            }
            Ok(())
        } else {
            Err(anyhow!("can't add sand here"))
        }
    }

    fn add_rocks_internal(
        structures: &mut HashMap<u32, Vec<Range<u32>>>,
        x: u32,
        y_interval: Range<u32>,
    ) -> Result<()> {
        if let Some(curr_obstacles) = structures.get_mut(&x) {
            // add/extend obstacle
            for interval in curr_obstacles.iter_mut() {
                if interval.start <= y_interval.end && y_interval.start <= interval.end {
                    interval.start = cmp::min(interval.start, y_interval.start);
                    interval.end = cmp::max(interval.end, y_interval.end);
                    return Ok(());
                }
            }
            curr_obstacles.push(y_interval);
        } else {
            structures.insert(x, vec![y_interval]);
        }

        Ok(())
    }

    fn draw_cave(&self) {
        let min_x = self.min_x - 1;
        let max_x = self.max_x + 1;

        let max_y = self.max_y;

        for y in 0..max_y + 1 {
            let mut line = String::new();
            for x in min_x..max_x + 1 {
                if self.grains.contains(&(x, y)) {
                    line.push('o');
                } else if self.check_pos_internal(x, y) {
                    if x == 500 && y == 0 {
                        line.push('+');
                    } else if y < self.curr_path.len() as u32
                        && *self.curr_path.get(y as usize).unwrap() == x
                    {
                        line.push('~');
                    } else {
                        line.push('.');
                    }
                } else {
                    line.push('#');
                }
            }
            info!("{}", &line);
        }
    }
}

impl Iterator for RockStructure {
    type Item = (); // sequence of columns that specifiy the current falling path

    fn next(&mut self) -> Option<Self::Item> {
        let mut curr_x = self
            .curr_path
            .pop_back()
            .expect("current path can't be empty");
        let mut curr_y = self.curr_path.len() as u32;
        debug!("current path head is ({},{})", curr_x, curr_y);

        while let Some((new_x, new_y)) = self.next_move(curr_x, curr_y) {
            self.curr_path.push_back(curr_x);
            curr_x = new_x;
            curr_y = new_y;
            debug!("moving path head to ({},{})", curr_x, curr_y);
            if self.into_the_abyss(curr_x, curr_y) {
                info!("INTO THE ABYSS");
                while self.curr_path.len() < self.max_y as usize + 1 {
                    self.curr_path.push_back(curr_x);
                }
                return None;
            }
        }
        self.add_sand(curr_x, curr_y)
            .expect("found path, but failed to add sand");

        if curr_x == 500 && curr_y == 0 {
            // stacked to the top
            info!("cave is filled now");
            return None;
        }

        Some(())
    }
}
