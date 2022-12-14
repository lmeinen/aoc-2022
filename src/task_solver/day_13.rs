use anyhow::{bail, Context, Result};

use log::{debug, error, info};

use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
    str::FromStr,
};

pub fn solve(_task: u8, input: String) -> Result<()> {
    let parser = PacketParser::init(input).context("failed to instantiate parser")?;

    match _task {
        1 => solve_1(parser),
        2 => solve_2(parser),
        _ => bail!("task doesn't exist!"),
    }
}

fn solve_1(parser: PacketParser) -> Result<()> {
    let mut index_sum = 0u32;

    for (i, (packet_0, packet_1)) in parser.enumerate() {
        debug!("found packet pair:\n\t{:?}\n\t{:?}", packet_0, packet_1);
        if packet_0 <= packet_1 {
            debug!("packets are in the right order!");
            index_sum += i as u32 + 1;
        }
    }

    info!(
        "sum of indices of packet pairs that are in the right order: {}",
        index_sum
    );

    Ok(())
}

fn solve_2(parser: PacketParser) -> Result<()> {
    let mut packet_list = parser
        .map(|(p1, p2)| vec![p1, p2])
        .flatten()
        .collect::<Vec<Packet>>();

    let sep_0 = Packet::LIST(vec![Packet::LIST(vec![Packet::INT(2)])]);
    let sep_1 = Packet::LIST(vec![Packet::LIST(vec![Packet::INT(6)])]);

    packet_list.push(sep_0.clone());
    packet_list.push(sep_1.clone());

    packet_list.sort_unstable();

    let sep_0_i = packet_list
        .iter()
        .position(|p| p == &sep_0)
        .expect("packet list didn't contain [[2]]")
        + 1;
    let sep_1_i = packet_list
        .iter()
        .position(|p| p == &sep_1)
        .expect("packet list didn't contain [[6]]")
        + 1;

    info!(
        "decoder key for the distress signal is {} x {} = {}",
        sep_0_i,
        sep_1_i,
        sep_0_i * sep_1_i
    );

    Ok(())
}

#[derive(Debug, Eq, Clone)]
enum Packet {
    INT(u32),
    LIST(Vec<Packet>),
}

impl Ord for Packet {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Self::INT(x), Self::INT(y)) => x.cmp(y),
            (Self::LIST(x), Self::LIST(y)) => x.cmp(y),
            (Self::INT(x), Self::LIST(_)) => Packet::LIST(vec![Packet::INT(*x)]).cmp(other),
            (Self::LIST(_), Self::INT(y)) => self.cmp(&Packet::LIST(vec![Packet::INT(*y)])),
        }
    }
}

impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Packet {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::INT(l0), Self::INT(r0)) => l0 == r0,
            (Self::LIST(l0), Self::LIST(r0)) => l0 == r0,
            (Self::INT(x), Self::LIST(_)) => Packet::LIST(vec![Packet::INT(*x)]).eq(other),
            (Self::LIST(_), Self::INT(y)) => self.eq(&Packet::LIST(vec![Packet::INT(*y)])),
        }
    }
}

impl FromStr for Packet {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        debug!("parsing packet: {}", s);

        let mut packet_stack = VecDeque::new();
        let mut current_list = None;
        let mut current_int = String::new();

        for c in s.trim().chars() {
            match c {
                '[' => {
                    if let Some(packet_list) = current_list {
                        packet_stack.push_back(packet_list);
                    }
                    current_list = Some(Vec::new());
                }
                ']' => {
                    let mut packet_list =
                        current_list.expect("found ']' char, but current_list doesn't exist.");
                    if !current_int.is_empty() {
                        packet_list.push(Packet::INT(
                            current_int.parse().expect(&format!(
                                "couldn't parse int from string: {}",
                                current_int
                            )),
                        ));
                        current_int.clear();
                    }
                    let new_packet = Packet::LIST(packet_list);
                    if let Some(outer_list) = packet_stack.pop_back() {
                        debug!(
                            "popped outer list: {:?} - adding new packet: {:?}",
                            outer_list, new_packet
                        );
                        packet_list = outer_list;
                        packet_list.push(new_packet);
                        current_list = Some(packet_list);
                    } else {
                        return Ok(new_packet);
                    }
                }
                ',' => {
                    if !current_int.is_empty() {
                        let mut packet_list = current_list
                            .expect("finished parsing int element, but current_list doesn't exist");
                        packet_list.push(Packet::INT(
                            current_int.parse().expect(&format!(
                                "couldn't parse int from string: {}",
                                current_int
                            )),
                        ));
                        current_list = Some(packet_list);
                        current_int.clear();
                    }
                }
                _ => current_int.push(c),
            }
        }

        error!("finished parsing string, but failed to create full packet");
        Err(())
    }
}

struct PacketParser {
    in_reader: BufReader<File>,
    line: String,
}

impl PacketParser {
    fn init(input: String) -> Result<Self> {
        // open input file
        let in_file = File::open(input).context(format!("Failed to read input"))?;

        // uses a reader buffer
        let in_reader = BufReader::new(in_file);
        let line = String::new();

        Ok(PacketParser { in_reader, line })
    }
}

impl Iterator for PacketParser {
    type Item = (Packet, Packet);

    fn next(&mut self) -> Option<Self::Item> {
        let mut packet_pair = Vec::new();

        while packet_pair.len() != 2 {
            let bytes_read = self
                .in_reader
                .read_line(&mut self.line)
                .expect("Failed to read line in input file");
            if bytes_read == 0 {
                return None; // EOF
            } else if self.line == "\n" {
                continue;
            }

            packet_pair.push(Packet::from_str(self.line.trim()).expect("failed to parse packet"));

            self.line.clear();
        }

        let packet_1 = packet_pair.pop().unwrap();
        let packet_0 = packet_pair.pop().unwrap();

        Some((packet_0, packet_1))
    }
}
