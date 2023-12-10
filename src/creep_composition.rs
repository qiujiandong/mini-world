use screeps::*;

pub struct CreepComposition {
    carry_cnt: u8,
    work_cnt: u8,
    part_vec: Vec<Part>,
}

pub enum CreepType {
    Normal(CreepComposition),
    Upgrader(CreepComposition),
    Builder(CreepComposition),
    Carrier(CreepComposition),
    Miner(CreepComposition),
}

impl CreepComposition {
    pub fn new(compose: &Vec<(Part, u8)>) -> CreepComposition {
        let mut carry_cnt = 0;
        let mut work_cnt = 0;
        let mut part_vec: Vec<Part> = Vec::new();
        for item in compose.iter() {
            if let Part::Carry = item.0 {
                carry_cnt = item.1;
            }
            if let Part::Work = item.0 {
                work_cnt = item.1;
            }
            for _ in 0..item.1 {
                part_vec.push(item.0);
            }
        }
        CreepComposition {
            carry_cnt,
            work_cnt,
            part_vec,
        }
    }

    pub fn cost(&self) -> u32 {
        self.part_vec.iter().map(|p| p.cost()).sum()
    }
}

impl CreepType {
    pub fn new(name: &str) -> CreepType {
        match name {
            "upgrader" => {
                let mut compose: Vec<(Part, u8)> = Vec::new();
                compose.push((Part::Work, 1));
                compose.push((Part::Carry, 1));
                compose.push((Part::Move, 2));
                CreepType::Upgrader(CreepComposition::new(&compose))
            }
            "builder" => {
                let mut compose: Vec<(Part, u8)> = Vec::new();
                compose.push((Part::Work, 1));
                compose.push((Part::Carry, 1));
                compose.push((Part::Move, 2));
                CreepType::Builder(CreepComposition::new(&compose))
            }
            "carrier" => {
                let mut compose: Vec<(Part, u8)> = Vec::new();
                compose.push((Part::Work, 1));
                compose.push((Part::Carry, 1));
                compose.push((Part::Move, 2));
                CreepType::Carrier(CreepComposition::new(&compose))
            }
            "miner" => {
                let mut compose: Vec<(Part, u8)> = Vec::new();
                compose.push((Part::Work, 1));
                compose.push((Part::Carry, 1));
                compose.push((Part::Move, 2));
                CreepType::Miner(CreepComposition::new(&compose))
            }
            _ => {
                let mut compose: Vec<(Part, u8)> = Vec::new();
                compose.push((Part::Work, 1));
                compose.push((Part::Carry, 1));
                compose.push((Part::Move, 2));
                CreepType::Normal(CreepComposition::new(&compose))
            }
        }
    }

    pub fn cost(&self) -> u32 {
        match self {
            CreepType::Normal(c) => c.cost(),
            CreepType::Upgrader(c) => c.cost(),
            CreepType::Builder(c) => c.cost(),
            CreepType::Carrier(c) => c.cost(),
            CreepType::Miner(c) => c.cost(),
        }
    }

    pub fn part_vec(&self) -> &Vec<Part> {
        match self {
            CreepType::Normal(c) => &c.part_vec,
            CreepType::Upgrader(c) => &c.part_vec,
            CreepType::Builder(c) => &c.part_vec,
            CreepType::Carrier(c) => &c.part_vec,
            CreepType::Miner(c) => &c.part_vec,
        }
    }

    pub fn carry_cnt(&self) -> u8 {
        match self {
            CreepType::Normal(c) => c.carry_cnt,
            CreepType::Upgrader(c) => c.carry_cnt,
            CreepType::Builder(c) => c.carry_cnt,
            CreepType::Carrier(c) => c.carry_cnt,
            CreepType::Miner(c) => c.carry_cnt,
        }
    }

    pub fn work_cnt(&self) -> u8 {
        match self {
            CreepType::Normal(c) => c.work_cnt,
            CreepType::Upgrader(c) => c.work_cnt,
            CreepType::Builder(c) => c.work_cnt,
            CreepType::Carrier(c) => c.work_cnt,
            CreepType::Miner(c) => c.work_cnt,
        }
    }
}
