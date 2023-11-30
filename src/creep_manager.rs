use crate::creep_target::*;
use log::*;
use screeps::*;

const BUILDER_PART: [Part; 8] = [
    Part::Work,
    Part::Work,
    Part::Carry,
    Part::Carry,
    Part::Move,
    Part::Move,
    Part::Move,
    Part::Move,
];
const MINER_PART: [Part; 8] = [
    Part::Work,
    Part::Work,
    Part::Work,
    Part::Work,
    Part::Move,
    Part::Move,
    Part::Move,
    Part::Move,
];
const CARRIER_PART: [Part; 10] = [
    Part::Carry,
    Part::Carry,
    Part::Carry,
    Part::Carry,
    Part::Carry,
    Part::Move,
    Part::Move,
    Part::Move,
    Part::Move,
    Part::Move,
];
const NORMAL_PART: [Part; 4] = [Part::Work, Part::Carry, Part::Move, Part::Move];

#[derive(Debug)]
enum CreepState {
    NotExist,
    Spawning,
    Idle,
    OnWay,
    Working,
}

#[derive(Debug)]
enum CreepType {
    Normal,
    Upgrader,
    Builder,
    Carrier,
    Miner,
}

impl From<String> for CreepType {
    fn from(value: String) -> CreepType {
        let name = value
            .trim_end_matches(char::is_numeric)
            .trim_end_matches('-');
        match name {
            "upgrader" => CreepType::Upgrader,
            "builder" => CreepType::Builder,
            "carrier" => CreepType::Carrier,
            "miner" => CreepType::Miner,
            _ => CreepType::Normal,
        }
    }
}

pub struct CreepMgr {
    name: String,
    id: Option<ObjectId<Creep>>,
    state: CreepState,
    target: Option<CreepTarget>,
    career: CreepType,
}

impl CreepMgr {
    pub fn new(name: &str) -> Self {
        match is_creep_exist(name) {
            Some(id) => {
                debug!("find existing creep {:?}", name);
                Self {
                    name: String::from(name),
                    id: Some(id),
                    state: CreepState::Idle,
                    target: None,
                    career: CreepType::from(String::from(name)),
                }
            }
            None => Self {
                name: String::from(name),
                id: None,
                state: CreepState::NotExist,
                target: None,
                career: CreepType::from(String::from(name)),
            },
        }
    }

    // pub fn report_status(&self) {
    //     info!(
    //         "creep {} state: {:?}, target: {:?}, path: {:?}, career: {:?}",
    //         self.name, self.state, self.target, self.path, self.career
    //     );
    // }

    pub fn run(&mut self) {
        if let None = is_creep_exist(self.name.as_str()) {
            self.state = CreepState::NotExist;
        }
        match self.state {
            CreepState::NotExist => {
                if let Err(error_code) = self.try_spawn() {
                    debug!("creep {:?} try spawn error: {:?}", self.name, error_code)
                }

                if let Ok(()) = self.is_spawning() {
                    self.state = CreepState::Spawning;
                    debug!("creep {:?} is spawning", self.name);
                }
            }
            CreepState::Spawning => {
                if let Ok(()) = self.is_spawn_done() {
                    debug!("creep {:?} spawn done", self.name);
                    self.set_id();
                    if let Ok(()) = self.seek_target() {
                        if let Ok(()) = self.do_work() {
                            // try do work succeed
                            debug!("creep {:?} is working", self.name);
                            self.state = CreepState::Working;
                        } else {
                            // try do work failed
                            if let Err(_) = self.go_ahead() {
                                debug!("creep {:?} can't move", self.name);
                            }
                            self.state = CreepState::OnWay;
                        }
                    } else {
                        debug!("creep {:?} can't find target", self.name);
                        self.state = CreepState::Idle;
                    }
                } else {
                    debug!("creep {:?} is spawning", self.name);
                }
            }
            CreepState::Idle => {
                if let Ok(()) = self.seek_target() {
                    if let Ok(()) = self.do_work() {
                        // try do work succeed
                        debug!("creep {:?} is working", self.name);
                        self.state = CreepState::Working;
                    } else {
                        // try do work failed
                        if let Err(_) = self.go_ahead() {
                            debug!("creep {:?} can't move", self.name);
                        }
                        self.state = CreepState::OnWay;
                    }
                } else {
                    debug!("creep {:?} can't find target", self.name);
                }
            }
            CreepState::OnWay => {
                if let Ok(()) = self.is_work_done() {
                    if let Ok(()) = self.seek_target() {
                        if let Ok(()) = self.do_work() {
                            // try do work succeed
                            debug!("creep {:?} is working", self.name);
                            self.state = CreepState::Working;
                        } else {
                            // try do work failed
                            if let Err(_) = self.go_ahead() {
                                debug!("creep {:?} can't move", self.name);
                            }
                            self.state = CreepState::OnWay;
                        }
                    } else {
                        debug!("creep {:?} can't find target", self.name);
                        self.state = CreepState::Idle;
                    }
                } else {
                    if let Ok(()) = self.do_work() {
                        // try do work succeed
                        debug!("creep {:?} is working", self.name);
                        self.state = CreepState::Working;
                    } else {
                        // try do work failed
                        if let Err(_) = self.go_ahead() {
                            debug!("creep {:?} can't move", self.name);
                        }
                        self.state = CreepState::OnWay;
                    }
                }
            }
            CreepState::Working => {
                if let Ok(()) = self.is_work_done() {
                    if let Ok(()) = self.seek_target() {
                        if let Ok(()) = self.do_work() {
                            // try do work succeed
                            debug!("creep {:?} is working", self.name);
                            self.state = CreepState::Working;
                        } else {
                            // try do work failed
                            if let Err(_) = self.go_ahead() {
                                debug!("creep {:?} can't move", self.name);
                            }
                            self.state = CreepState::OnWay;
                        }
                    } else {
                        debug!("creep {:?} can't find target", self.name);
                        self.state = CreepState::Idle;
                    }
                } else {
                    if let Ok(()) = self.do_work() {
                        debug!("creep {:?} is working", self.name);
                    } else {
                        debug!("creep {:?} can't do work", self.name);
                        if let Ok(()) = self.seek_target() {
                            if let Ok(()) = self.do_work() {
                                // try do work succeed
                                debug!("creep {:?} is working", self.name);
                                self.state = CreepState::Working;
                            } else {
                                // try do work failed
                                if let Err(_) = self.go_ahead() {
                                    debug!("creep {:?} can't move", self.name);
                                }
                                self.state = CreepState::OnWay;
                            }
                        } else {
                            debug!("creep {:?} can't find target", self.name);
                            self.state = CreepState::Idle;
                        }
                    }
                }
            }
        }
    }

    fn try_spawn(&self) -> Result<(), ErrorCode> {
        // TODO(qiujiandong): to support multiple spawn
        match game::spawns().get(String::from("Spawn1")) {
            Some(spawn) => match spawn.spawning() {
                Some(spawning) => {
                    if spawning.name() == self.name {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                None => {
                    let part_array: &[Part];
                    match self.career {
                        CreepType::Builder => part_array = &BUILDER_PART,
                        CreepType::Miner => part_array = &MINER_PART,
                        CreepType::Carrier => part_array = &CARRIER_PART,
                        CreepType::Upgrader => part_array = &NORMAL_PART,
                        _ => part_array = &NORMAL_PART,
                    }

                    let cost: u32 = part_array.iter().map(|p| p.cost()).sum();
                    let energy_amount = spawn.room().unwrap().energy_available();

                    // NOTE(qiujiandong): stop spawn miner
                    if let CreepType::Miner = self.career {
                        Err(ErrorCode::InvalidArgs)
                    } else {
                        if energy_amount < cost {
                            Err(ErrorCode::NotEnough)
                        } else {
                            spawn.spawn_creep(part_array, self.name.as_str())
                        }
                    }
                }
            },
            None => Err(ErrorCode::NotFound),
        }
    }

    fn is_spawn_done(&mut self) -> Result<(), ErrorCode> {
        // TODO(qiujiandong): to support multiple spawn
        let spawning = game::spawns()
            .get(String::from("Spawn1"))
            .unwrap()
            .spawning();

        match spawning {
            None => Ok(()),
            Some(s) => {
                if s.name() == self.name {
                    Err(ErrorCode::Busy)
                } else {
                    Ok(())
                }
            }
        }
    }

    fn is_spawning(&self) -> Result<(), ErrorCode> {
        let spawn = game::spawns().get(String::from("Spawn1"));
        let spawning: Option<_>;

        match spawn {
            None => {
                return Err(ErrorCode::NotFound);
            }
            Some(spawn_) => {
                spawning = spawn_.spawning();
            }
        }

        match spawning {
            None => Err(ErrorCode::NotFound),
            Some(s) => {
                if s.name() == self.name {
                    Ok(())
                } else {
                    Err(ErrorCode::Busy)
                }
            }
        }
    }

    fn seek_target(&mut self) -> Result<(), ErrorCode> {
        self.target = None;
        let creep = self.get_creep().unwrap();

        debug!("creep {:?} is seeking target", self.name);

        match self.career {
            CreepType::Builder => {
                let no: i32 = self.name.strip_prefix("builder-").unwrap().parse().unwrap();
                // if energy full
                // 1. find construction site and build
                // 2. find spawn, energy in which is not full
                // 3. find controller to upgrade
                // else
                // 1. fetch energy
                if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                    match find_notfull_spawn_or_extension(creep.room().as_ref().unwrap()) {
                        Some(spawn) => {
                            self.target = Some(spawn);
                        }
                        None => {
                            let cs = creep
                                .room()
                                .as_ref()
                                .unwrap()
                                .find(find::CONSTRUCTION_SITES, None);

                            if let Some(cs_) = cs
                                .iter()
                                .find(|cs| cs.structure_type() == StructureType::Storage)
                            {
                                self.target =
                                    Some(CreepTarget::new(&ObjectWithPosition::from(cs_.clone())));
                            } else {
                                self.target = Some(CreepTarget::new(&ObjectWithPosition::from(
                                    cs[0].clone(),
                                )));
                            }
                        }
                    }
                } else {
                    let structures = creep.room().as_ref().unwrap().find(find::STRUCTURES, None);
                    if let Some(structure) = structures.iter().find(|s| {
                        if let StructureObject::StructureContainer(container) = s {
                            container
                                .store()
                                .get_used_capacity(Some(ResourceType::Energy))
                                > 50
                        } else {
                            false
                        }
                    }) {
                        if let StructureObject::StructureContainer(container) = structure {
                            self.target = Some(CreepTarget::new(&ObjectWithPosition::from(
                                container.clone(),
                            )));
                        }
                    } else {
                        let sources = creep
                            .room()
                            .as_ref()
                            .unwrap()
                            .find(find::SOURCES_ACTIVE, None);

                        if no < 3 {
                            if let Some(source_) =
                                sources.iter().find(|s| s.js_pos().is_equal_to_xy(42, 5))
                            {
                                self.target = Some(CreepTarget::new(&ObjectWithPosition::from(
                                    source_.clone(),
                                )));
                            }
                        } else {
                            if let Some(source_) =
                                sources.iter().find(|s| s.js_pos().is_equal_to_xy(4, 46))
                            {
                                self.target = Some(CreepTarget::new(&ObjectWithPosition::from(
                                    source_.clone(),
                                )));
                            }
                        }
                    }
                }
            }
            CreepType::Miner => {
                let no: i32 = self.name.strip_prefix("miner-").unwrap().parse().unwrap();
                let sources = creep
                    .room()
                    .as_ref()
                    .unwrap()
                    .find(find::SOURCES_ACTIVE, None);
                if no == 0 {
                    if let Some(source_) = sources.iter().find(|s| s.js_pos().is_equal_to_xy(42, 5))
                    {
                        self.target =
                            Some(CreepTarget::new(&ObjectWithPosition::from(source_.clone())));
                    }
                } else if no == 1 {
                    if let Some(source_) = sources.iter().find(|s| s.js_pos().is_equal_to_xy(4, 46))
                    {
                        self.target =
                            Some(CreepTarget::new(&ObjectWithPosition::from(source_.clone())));
                    }
                }
            }
            CreepType::Carrier => {}
            CreepType::Upgrader => {
                // if energy full
                // 1. find controller to upgrade
                // else
                // 1. fetch energy
                if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                    if let Some(controller) = find_controller(creep.room().as_ref().unwrap()) {
                        self.target = Some(controller);
                    }
                } else {
                    if let Some(energy) = find_available_energy(creep.room().as_ref().unwrap()) {
                        self.target = Some(energy);
                    }
                }
            }
            _ => {}
        }
        if let Some(_) = self.target {
            Ok(())
        } else {
            Err(ErrorCode::NotFound)
        }
    }

    fn go_ahead(&mut self) -> Result<(), ErrorCode> {
        // TODO(qiujiandong): memory path
        if let CreepType::Miner = self.career {
            let no: i32 = self.name.strip_prefix("miner-").unwrap().parse().unwrap();
            let creep = self.get_creep().unwrap();
            if no == 0 {
                let mut pos = self.target.as_ref().unwrap().get_pos();
                pos.offset(-1, 0);
                creep.move_to(pos)
            } else if no == 1 {
                let mut pos = self.target.as_ref().unwrap().get_pos();
                pos.offset(0, -1);
                creep.move_to(pos)
            } else {
                Err(ErrorCode::InvalidTarget)
            }
        } else {
            let creep = self.get_creep().unwrap();
            creep.move_to(self.target.as_ref().unwrap().get_pos())
        }
    }

    fn do_work(&mut self) -> Result<(), ErrorCode> {
        let creep = self.get_creep().unwrap();
        match self.target.as_ref() {
            Some(target_) => match target_.action {
                CreepAction::Build(id) => {
                    creep.build(game::get_object_by_id_typed(&id).as_ref().unwrap())
                }
                CreepAction::FetchFromSource(id) => {
                    let source = game::get_object_by_id_typed(&id).unwrap();
                    if let CreepType::Miner = self.career {
                        let mut target_pos = self.target.as_ref().unwrap().get_pos();
                        let no: i32 = self.name.strip_prefix("miner-").unwrap().parse().unwrap();
                        if no == 0 {
                            target_pos.offset(-1, 0);
                        } else if no == 1 {
                            target_pos.offset(0, -1);
                        }
                        if creep.pos().is_equal_to(target_pos) {
                            creep.harvest(&source)
                        } else {
                            Err(ErrorCode::NoPath)
                        }
                    } else {
                        creep.harvest(&source)
                    }
                }
                CreepAction::FetchFromContainer(id) => {
                    let container = game::get_object_by_id_typed(&id).unwrap();
                    creep.withdraw(&container, ResourceType::Energy, None)
                }
                CreepAction::TransferToSpawn(id) => {
                    let spawn = game::get_object_by_id_typed(&id).unwrap();
                    creep.transfer(&spawn, ResourceType::Energy, None)
                }
                CreepAction::TransferToExtension(id) => {
                    let extension = game::get_object_by_id_typed(&id).unwrap();
                    creep.transfer(&extension, ResourceType::Energy, None)
                }
                CreepAction::Upgrade(id) => {
                    let controller = game::get_object_by_id_typed(&id).unwrap();
                    creep.upgrade_controller(&controller)
                }
                _ => Err(ErrorCode::InvalidArgs),
            },
            None => Err(ErrorCode::InvalidArgs),
        }
    }

    fn is_work_done(&self) -> Result<(), ErrorCode> {
        let creep = self.get_creep().unwrap();
        match self.target.as_ref() {
            Some(target_) => match target_.action {
                CreepAction::Build(id) => {
                    if let None = game::get_object_by_id_typed(&id) {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepAction::FetchFromSource(id) => {
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                        Ok(())
                    } else if game::get_object_by_id_typed(&id).unwrap().energy() == 0 {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepAction::TransferToSpawn(id) => {
                    if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        == 0
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepAction::Upgrade(_) => Err(ErrorCode::Busy),
                _ => Ok(()),
            },
            None => Ok(()),
        }
    }

    fn set_id(&mut self) {
        let name_ = self.name.clone();
        let raw_id = game::creeps().get(name_).unwrap().try_raw_id().unwrap();
        self.id = Some(ObjectId::from(raw_id));
    }

    fn get_creep(&self) -> Option<Creep> {
        game::get_object_by_id_typed(self.id.as_ref().unwrap())
    }
}

fn is_creep_exist(name: &str) -> Option<ObjectId<Creep>> {
    let mut it = game::creeps().keys();
    if let Some(_) = it.find(|key| *key == name) {
        let raw_id = game::creeps()
            .get(name.to_string())
            .unwrap()
            .try_raw_id()
            .unwrap();
        Some(ObjectId::from(raw_id))
    } else {
        None
    }
}
