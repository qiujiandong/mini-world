use crate::creep_target::*;
use log::*;
use screeps::*;
use wasm_bindgen::JsValue;

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
const MINER_PART: [Part; 9] = [
    Part::Work,
    Part::Work,
    Part::Work,
    Part::Work,
    Part::Carry,
    Part::Move,
    Part::Move,
    Part::Move,
    Part::Move,
];
const CARRIER_PART: [Part; 12] = [
    Part::Work,
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
    no: u8,
    id: Option<ObjectId<Creep>>,
    state: CreepState,
    target: Option<CreepTarget>,
    career: CreepType,
}

impl CreepMgr {
    pub fn new(name: &str) -> Self {
        let seg: Vec<_> = name.split('-').collect();
        let no: u8 = seg[1].parse().unwrap_or(0xFF);
        match is_creep_exist(name) {
            Some(id) => {
                debug!("find existing creep {:?}", name);
                Self {
                    name: String::from(name),
                    id: Some(id),
                    no,
                    state: CreepState::Idle,
                    target: None,
                    career: CreepType::from(String::from(name)),
                }
            }
            None => Self {
                name: String::from(name),
                id: None,
                no,
                state: CreepState::NotExist,
                target: None,
                career: CreepType::from(String::from(name)),
            },
        }
    }

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
                            debug!("creep {:?} is on way", self.name);
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
                        debug!("creep {:?} is on way", self.name);
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

                    if energy_amount < cost {
                        Err(ErrorCode::NotEnough)
                    } else {
                        spawn.spawn_creep(part_array, self.name.as_str())
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
                // if energy exist
                // 1. find spawn or extension, energy in which is not full
                // 2. find construction site and build
                // 3. find controller to upgrade
                // else
                // 1. fetch energy
                if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0
                    && self.is_creep_working().unwrap_or(true)
                {
                    // 1. find spawn, energy in which is not full
                    if let Some(spawn) = find_notfull_spawn_or_extension(&creep) {
                        self.target = Some(spawn);
                    } else {
                        // 2. find construction site and build
                        if let Some(construction_site) = find_construction_site(&creep, None, None)
                        {
                            self.target = Some(construction_site);
                        } else {
                            // 3. find controller to upgrade
                            if let Some(controller) = find_controller(&creep) {
                                self.target = Some(controller);
                            }
                        }
                    }
                } else {
                    self.setup_working_status(false);
                    self.find_energy_for_work();
                }
            }
            CreepType::Miner => {
                if let Some(source) = find_source(&creep, None) {
                    self.target = Some(source);
                }
            }
            CreepType::Carrier => {
                // if has energy
                // 1. find spawn or extension, energy in which is not full
                // 2. find storage to store
                // 3. find controller to upgrade
                // else
                // 1. fetch energy from container, source
                if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0
                    && self.is_creep_working().unwrap_or(true)
                {
                    // 1. find spawn, energy in which is not full
                    if let Some(spawn) = find_notfull_spawn_or_extension(&creep) {
                        self.target = Some(spawn);
                    } else {
                        // 2. find storage to store
                        if let Some(storage) = find_storage(&creep, None, ActionCommand::Transfer) {
                            self.target = Some(storage);
                        } else {
                            // 3. find controller to upgrade
                            if let Some(controller) = find_controller(&creep) {
                                self.target = Some(controller);
                            }
                        }
                    }
                } else {
                    self.setup_working_status(false);
                    if let Some(container) = find_container(&creep, None, ActionCommand::Fetch) {
                        self.target = Some(container);
                    } else {
                        if let Some(source) = find_source(&creep, None) {
                            self.target = Some(source);
                        }
                    }
                }
            }
            CreepType::Upgrader => {
                // if energy full
                // 1. find spawn or extension, energy in which is not full
                // 2. find controller to upgrade
                // else
                // 1. fetch energy
                if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0
                    && self.is_creep_working().unwrap_or(true)
                {
                    // 1. find spawn, energy in which is not full
                    if let Some(spawn) = find_notfull_spawn_or_extension(&creep) {
                        self.target = Some(spawn);
                    } else {
                        // 2. find controller to upgrade
                        if let Some(controller) = find_controller(&creep) {
                            self.target = Some(controller);
                        }
                    }
                } else {
                    self.setup_working_status(false);
                    self.find_energy_for_work();
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
        let creep = self.get_creep().unwrap();
        if let CreepType::Miner = self.career {
            match self.no {
                0 => {
                    let pos = self
                        .target
                        .as_ref()
                        .unwrap()
                        .pos()
                        .unwrap()
                        .checked_add((-1, 0))
                        .unwrap();
                    // creep.move_to(pos)
                    let options = MoveToOptions::new().reuse_path(0);
                    creep.move_to_with_options(pos, Some(options))
                }
                1 => {
                    let pos = self
                        .target
                        .as_ref()
                        .unwrap()
                        .pos()
                        .unwrap()
                        .checked_add((0, -1))
                        .unwrap();
                    let options = MoveToOptions::new().reuse_path(0);
                    creep.move_to_with_options(pos, Some(options))
                }
                _ => Err(ErrorCode::InvalidTarget),
            }
        } else {
            // creep.move_to(self.target.as_ref().unwrap().pos().unwrap())
            let options = MoveToOptions::new().reuse_path(0);
            creep.move_to_with_options(self.target.as_ref().unwrap().pos().unwrap(), Some(options))
        }
    }

    fn do_work(&mut self) -> Result<(), ErrorCode> {
        let creep = self.get_creep().unwrap();
        match self.target.as_ref() {
            Some(target_) => match target_ {
                CreepTarget::Build(id) => {
                    creep.build(game::get_object_by_id_typed(&id).as_ref().unwrap())
                }
                CreepTarget::FetchFromSource(id) => {
                    let source = game::get_object_by_id_typed(&id).unwrap();
                    if let CreepType::Miner = self.career {
                        let mut target_pos = self.target.as_ref().unwrap().pos().unwrap();
                        match self.no {
                            0 => {
                                target_pos.offset(-1, 0);
                            }
                            1 => {
                                target_pos.offset(0, -1);
                            }
                            _ => {}
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
                CreepTarget::FetchFromContainer(id) => {
                    let container = game::get_object_by_id_typed(&id).unwrap();
                    creep.withdraw(&container, ResourceType::Energy, None)
                }
                CreepTarget::FetchFromStorage(id) => {
                    let storage = game::get_object_by_id_typed(&id).unwrap();
                    creep.withdraw(&storage, ResourceType::Energy, None)
                }
                CreepTarget::TransferToSpawn(id) => {
                    let spawn = game::get_object_by_id_typed(&id).unwrap();
                    creep.transfer(&spawn, ResourceType::Energy, None)
                }
                CreepTarget::TransferToExtension(id) => {
                    let extension = game::get_object_by_id_typed(&id).unwrap();
                    creep.transfer(&extension, ResourceType::Energy, None)
                }
                CreepTarget::Upgrade(id) => {
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
            Some(target_) => match target_ {
                CreepTarget::Build(id) => {
                    if let None = game::get_object_by_id_typed(&id) {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::FetchFromSource(id) => {
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0
                        || game::get_object_by_id_typed(&id).unwrap().energy() == 0
                    {
                        self.setup_working_status(true);
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::FetchFromContainer(id) => {
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0
                        || game::get_object_by_id_typed(&id)
                            .unwrap()
                            .store()
                            .get_used_capacity(Some(ResourceType::Energy))
                            == 0
                    {
                        self.setup_working_status(true);
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::FetchFromStorage(id) => {
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0
                        || game::get_object_by_id_typed(&id)
                            .unwrap()
                            .store()
                            .get_used_capacity(Some(ResourceType::Energy))
                            == 0
                    {
                        self.setup_working_status(true);
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::TransferToSpawn(id) => {
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
                CreepTarget::TransferToExtension(id) => {
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
                CreepTarget::Upgrade(_) => Err(ErrorCode::Busy),
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

    fn find_energy_for_work(&mut self) {
        let creep: Creep = self.get_creep().unwrap();
        if let Some(container) = find_container(&creep, None, ActionCommand::Fetch) {
            self.target = Some(container);
        } else {
            if let Some(storage) = find_storage(&creep, None, ActionCommand::Fetch) {
                self.target = Some(storage);
            } else {
                if let Some(source) = find_source(&creep, None) {
                    self.target = Some(source);
                }
            }
        }
    }

    fn setup_working_status(&self, is_working: bool) {
        let creep = self.get_creep().unwrap();
        creep.set_memory(&JsValue::from_bool(is_working));
    }

    fn is_creep_working(&self) -> Option<bool> {
        if let Some(creep) = self.get_creep() {
            creep.memory().as_bool()
        } else {
            None
        }
    }
}

fn is_creep_exist(name: &str) -> Option<ObjectId<Creep>> {
    if let Some(creep) = game::creeps().get(name.to_string()) {
        let raw_id = creep.try_raw_id().unwrap();
        Some(ObjectId::from(raw_id))
    } else {
        None
    }
}
