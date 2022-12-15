use anyhow::{bail, Context, Result};

use log::{debug, info};
use regex::Regex;

use std::{
    cmp,
    collections::HashSet,
    fs::File,
    io::{BufRead, BufReader},
};

use super::util;

pub fn solve(_task: u8, input: String) -> Result<()> {
    let sensor_map = SensorMap::init(input).context("failed to instantiate parser")?;

    info!(
        "instantiated sensor map - num sensors: {}",
        sensor_map.sensors.len()
    );

    match _task {
        1 => {
            let y = 2000000;
            let covered_ranges = sensor_map.get_row_coverage(y);
            let no_beacon_count = covered_ranges.iter().fold(0u32, |a, r| {
                let mut res = a + r.0.abs_diff(r.1) + 1;
                for (b_x, b_y) in sensor_map.beacons.iter() {
                    if *b_y == y && r.0 <= *b_x && r.1 >= *b_x {
                        res -= 1;
                    }
                }
                res
            });

            info!(
                "number of positions that cannot contain a beacon: {}",
                no_beacon_count
            );
        }
        2 => {
            let range_of_interest = (0, 4000000);
            // let range_of_interest = (0, 20);
            for y in range_of_interest.0..range_of_interest.1 + 1 {
                debug!("checking line {} for positions that aren't covered", y);
                let covered_ranges = sensor_map.get_row_coverage(y);
                let mut not_covered = Vec::new();
                not_covered.push(range_of_interest);
                difference_with_list(&mut not_covered, &covered_ranges);
                if let Some(r) = not_covered.pop() {
                    if not_covered.is_empty() && r.0 == r.1 {
                        let tuning_frequency = r.0 as i64 * 4000000 as i64 + y as i64;
                        info!(
                            "distress beacon found at ({},{}) - tuning frequency is {}",
                            r.0, y, tuning_frequency
                        );
                        break;
                    } else {
                        bail!("found range that wasn't entirely covered, but contained multiple elements: {:?}", not_covered);
                    }
                }
            }
        }
        _ => bail!("task doesn't exist!"),
    }

    Ok(())
}

type Coord = (i32, i32);
type Sensor = (Coord, u32);

struct SensorMap {
    /// unsorted list of sensors - a sensor consists of (x,y) coordinates and a radius
    sensors: Vec<Sensor>,
    /// unsorted list of detected beacons
    beacons: HashSet<Coord>,
}

fn from_scan((x_sensor, y_sensor): Coord, (x_beacon, y_beacon): Coord) -> Sensor {
    (
        (x_sensor, y_sensor),
        x_sensor.abs_diff(x_beacon) + y_sensor.abs_diff(y_beacon),
    )
}

fn reaches_row(((_, y_sensor), radius): &Sensor, y: i32) -> bool {
    y_sensor.abs_diff(y) <= *radius
}

impl SensorMap {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let mut in_reader = BufReader::new(in_file);
        let mut line = String::new();

        let mut sensors = Vec::new();
        let mut beacons = HashSet::new();

        // parse input file line-by-line
        let re_sensor = Regex::new(r"Sensor at x=(?P<x_sensor>[-]?\d+), y=(?P<y_sensor>[-]?\d+): closest beacon is at x=(?P<x_beacon>[-]?\d+), y=(?P<y_beacon>[-]?\d+)").unwrap();
        while in_reader
            .read_line(&mut line)
            .expect("Failed to read line in input file")
            != 0
            && line != "\n"
        {
            line = line.trim().to_owned();
            debug!("parsing line {}", line);
            let x_sensor = util::capture_and_parse(&re_sensor, &line, "x_sensor", &|s| {
                s.parse::<i32>().unwrap()
            });
            let y_sensor = util::capture_and_parse(&re_sensor, &line, "y_sensor", &|s| {
                s.parse::<i32>().unwrap()
            });
            let x_beacon = util::capture_and_parse(&re_sensor, &line, "x_beacon", &|s| {
                s.parse::<i32>().unwrap()
            });
            let y_beacon = util::capture_and_parse(&re_sensor, &line, "y_beacon", &|s| {
                s.parse::<i32>().unwrap()
            });
            sensors.push(from_scan((x_sensor, y_sensor), (x_beacon, y_beacon)));
            beacons.insert((x_beacon, y_beacon));
            line.clear();
        }

        Ok(SensorMap { sensors, beacons })
    }

    fn get_row_coverage(&self, y: i32) -> Vec<(i32, i32)> {
        let mut cover_range_list = Vec::new();
        for sensor in self.sensors.iter() {
            if reaches_row(sensor, y) {
                let ((x_sensor, y_sensor), radius) = sensor;
                let y_diff = radius - y_sensor.abs_diff(y);
                let sensor_range = (x_sensor - y_diff as i32, x_sensor + y_diff as i32);
                merge_into_list(&mut cover_range_list, sensor_range);
            }
        }
        cover_range_list
    }
}

fn merge_into_list(range_list: &mut Vec<(i32, i32)>, range: (i32, i32)) {
    let mut curr_range = range;
    range_list.retain(|r| {
        if curr_range.0 <= r.1 && r.0 <= curr_range.1 {
            curr_range.0 = cmp::min(curr_range.0, r.0);
            curr_range.1 = cmp::max(curr_range.1, r.1);
            false
        } else {
            true
        }
    });
    range_list.push(curr_range);
}

fn difference_with_range(range_list: &mut Vec<(i32, i32)>, r: &(i32, i32)) {
    let mut to_push = Vec::new();
    range_list.retain_mut(|curr_r| {
        if curr_r.0 <= r.1 && r.0 <= curr_r.1 {
            let overlap = (cmp::max(curr_r.0, r.0), cmp::min(curr_r.1, r.1));
            if overlap.0 == curr_r.0 && overlap.1 == curr_r.1 {
                // curr_r fully contained in r --> remove
                false
            } else if overlap.0 == r.0 && overlap.1 == r.1 {
                // r fully contained in curr_r --> split
                to_push.push((r.1 + 1, curr_r.1));
                curr_r.1 = overlap.0 - 1;
                true
            } else if overlap.0 == curr_r.0 {
                // reduce
                curr_r.0 = overlap.1 + 1;
                true
            } else {
                curr_r.1 = overlap.0 - 1;
                true
            }
        } else {
            // no overlap
            true
        }
    });
    range_list.extend(to_push);
}

fn difference_with_list(range_list_lhs: &mut Vec<(i32, i32)>, range_list_rhs: &Vec<(i32, i32)>) {
    for r in range_list_rhs.iter() {
        difference_with_range(range_list_lhs, r);
    }
}
