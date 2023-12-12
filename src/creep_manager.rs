use crate::creep_composition::*;
use crate::creep_target::*;
use log::*;
use screeps::*;
use wasm_bindgen::JsValue;

enum CreepState {
    NotExist,
    Spawning,
    Idle,
    OnWay,
    Working,
    WaitingForSafe,
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
        let name_prefix = name
            .trim_end_matches(char::is_numeric)
            .trim_end_matches('-');

        match is_creep_exist(name) {
            Some(id) => {
                debug!("find existing creep {:?}", name);
                Self {
                    name: String::from(name),
                    id: Some(id),
                    no,
                    state: CreepState::Idle,
                    target: None,
                    career: CreepType::new(name_prefix),
                }
            }
            None => Self {
                name: String::from(name),
                id: None,
                no,
                state: CreepState::NotExist,
                target: None,
                career: CreepType::new(name_prefix),
            },
        }
    }

    pub fn run(&mut self) {
        if let Some(room) = game::rooms().get(RoomName::new("E36N7").unwrap()) {
            let hostiles = room.find(find::HOSTILE_CREEPS, None);
            if hostiles.len() > 0 {
                self.state = CreepState::WaitingForSafe;
                if let Some(creep) = self.get_creep() {
                    creep.suicide().unwrap_or(());
                }
            }
        } else if let None = self.get_creep() {
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
                    debug!("creep {:?} work is done, find another target", self.name);
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
                        }
                    } else {
                        debug!("creep {:?} can't find target", self.name);
                        self.state = CreepState::Idle;
                    }
                } else {
                    debug!("creep {:?} work not done", self.name);
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
                    }
                }
            }
            CreepState::Working => {
                if let Ok(()) = self.is_work_done() {
                    // work is done or can't do
                    if let Ok(()) = self.seek_target() {
                        if let Ok(()) = self.do_work() {
                            // try do work succeed
                            debug!("creep {:?} is working", self.name);
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
                    self.do_work().unwrap_or(());
                    debug!("creep {:?} is working", self.name);
                }
            }
            CreepState::WaitingForSafe => {
                if let Some(room) = game::rooms().get(RoomName::new("E36N7").unwrap()) {
                    let hostiles = room.find(find::HOSTILE_CREEPS, None);
                    if hostiles.len() == 0 {
                        self.state = CreepState::Idle;
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
                    if spawn.room().unwrap().energy_available() >= self.career.cost() {
                        spawn.spawn_creep(self.career.part_vec(), self.name.as_str())
                    } else {
                        Err(ErrorCode::NotEnough)
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
            CreepType::Builder(_) => {
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
                    if let Some(storage) = find_storage(
                        &creep,
                        None,
                        Some(ActionCommand::Fetch),
                        Some(self.career.carry_cnt() as u16 * 50),
                    ) {
                        self.target = Some(storage);
                    }
                }
            }
            CreepType::Miner(_) => match self.no {
                0 => {
                    if let Some(_) = find_container(
                        &creep,
                        Some(Position::new(
                            RoomCoordinate::new(4).unwrap(),
                            RoomCoordinate::new(45).unwrap(),
                            RoomName::new("E36N7").unwrap(),
                        )),
                        ActionCommand::Transfer,
                        None,
                    ) {
                        if let Some(source) = find_source(
                            &creep,
                            Some(Position::new(
                                RoomCoordinate::new(4).unwrap(),
                                RoomCoordinate::new(46).unwrap(),
                                RoomName::new("E36N7").unwrap(),
                            )),
                        ) {
                            self.target = Some(source);
                        }
                    }
                }
                1 => {
                    if let Some(_) = find_container(
                        &creep,
                        Some(Position::new(
                            RoomCoordinate::new(41).unwrap(),
                            RoomCoordinate::new(5).unwrap(),
                            RoomName::new("E36N7").unwrap(),
                        )),
                        ActionCommand::Transfer,
                        None,
                    ) {
                        // container is not full
                        if let Some(source) = find_source(
                            &creep,
                            Some(Position::new(
                                RoomCoordinate::new(42).unwrap(),
                                RoomCoordinate::new(5).unwrap(),
                                RoomName::new("E36N7").unwrap(),
                            )),
                        ) {
                            self.target = Some(source);
                        }
                    } else {
                        // container is full
                        if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0 {
                            if let Some(link) = find_link(
                                &creep,
                                Some(Position::new(
                                    RoomCoordinate::new(40).unwrap(),
                                    RoomCoordinate::new(4).unwrap(),
                                    RoomName::new("E36N7").unwrap(),
                                )),
                                Some(ActionCommand::Transfer),
                                None,
                            ) {
                                self.target = Some(link);
                            }
                        } else {
                            if let Some(source) = find_source(
                                &creep,
                                Some(Position::new(
                                    RoomCoordinate::new(42).unwrap(),
                                    RoomCoordinate::new(5).unwrap(),
                                    RoomName::new("E36N7").unwrap(),
                                )),
                            ) {
                                self.target = Some(source);
                            }
                        }
                    }
                }
                _ => {}
            },
            CreepType::Carrier(_) => {
                if creep.store().get_used_capacity(Some(ResourceType::Energy)) > 0
                    && self.is_creep_working().unwrap_or(true)
                {
                    // 1. find spawn, energy in which is not full
                    if let Some(spawn) = find_notfull_spawn_or_extension(&creep) {
                        self.target = Some(spawn);
                    } else {
                        // 2. find tower
                        if let Some(tower) = find_tower(
                            &creep,
                            None,
                            Some(ActionCommand::Transfer),
                            Some(self.career.carry_cnt() as u16 * 50),
                        ) {
                            self.target = Some(tower);
                        } else {
                            // 3. find storage to store
                            if let Some(storage) = find_storage(
                                &creep,
                                None,
                                Some(ActionCommand::Transfer),
                                Some(self.career.carry_cnt() as u16 * 50),
                            ) {
                                self.target = Some(storage);
                            } else {
                                // 4. find controller to upgrade
                                if let Some(controller) = find_controller(&creep) {
                                    self.target = Some(controller);
                                }
                            }
                        }
                    }
                } else {
                    self.setup_working_status(false);

                    if let Some(container) = find_container(
                        &creep,
                        None,
                        ActionCommand::Fetch,
                        Some(self.career.carry_cnt() as u16 * 50),
                    ) {
                        self.target = Some(container);
                    } else {
                        if let Some(storage) = find_storage(
                            &creep,
                            None,
                            Some(ActionCommand::Fetch),
                            Some(self.career.carry_cnt() as u16 * 50),
                        ) {
                            self.target = Some(storage);
                        }
                    }
                }
            }
            CreepType::Upgrader(_) => {
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
                    if let Some(storage) = find_storage(
                        &creep,
                        None,
                        Some(ActionCommand::Fetch),
                        Some(self.career.carry_cnt() as u16 * 50),
                    ) {
                        self.target = Some(storage);
                    } else {
                        if let Some(container) = find_container(
                            &creep,
                            None,
                            ActionCommand::Fetch,
                            Some(self.career.carry_cnt() as u16 * 50),
                        ) {
                            self.target = Some(container);
                        } else {
                            if let Some(source) = find_source(&creep, None) {
                                self.target = Some(source);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
        if let Some(_) = self.target {
            Ok(())
        } else {
            if let Some(storage) = find_storage(&creep, None, None, None) {
                if creep.pos().is_near_to(storage.pos().unwrap()) {
                    Err(ErrorCode::NotFound)
                } else {
                    self.target = Some(storage);
                    Ok(())
                }
            } else {
                Err(ErrorCode::NotFound)
            }
        }
    }

    fn go_ahead(&mut self) -> Result<(), ErrorCode> {
        // TODO(qiujiandong): memory path
        let creep = self.get_creep().unwrap();
        if let CreepType::Miner(_) = self.career {
            match self.no {
                0 => {
                    // creep.move_to(pos)
                    let options = MoveToOptions::new().reuse_path(0);
                    creep.move_to_with_options(
                        Position::new(
                            RoomCoordinate::new(4).unwrap(),
                            RoomCoordinate::new(45).unwrap(),
                            RoomName::new("E36N7").unwrap(),
                        ),
                        Some(options),
                    )
                }
                1 => {
                    // creep.move_to(pos)
                    let options = MoveToOptions::new().reuse_path(0);
                    creep.move_to_with_options(
                        Position::new(
                            RoomCoordinate::new(41).unwrap(),
                            RoomCoordinate::new(5).unwrap(),
                            RoomName::new("E36N7").unwrap(),
                        ),
                        Some(options),
                    )
                }
                _ => Err(ErrorCode::InvalidTarget),
            }
        } else {
            // creep.move_to(self.target.as_ref().unwrap().pos().unwrap())
            self.repair_road()?;
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
                    if let CreepType::Miner(_) = self.career {
                        let mut target_pos = self.target.as_ref().unwrap().pos().unwrap();
                        match self.no {
                            0 => {
                                target_pos.offset(0, -1);
                            }
                            1 => {
                                target_pos.offset(-1, 0);
                            }
                            _ => {}
                        }
                        if creep.pos().is_equal_to(target_pos) {
                            match self.repair_container() {
                                Ok(_) => creep.harvest(&source),
                                Err(ErrorCode::Full) => {
                                    if creep.store().get_free_capacity(Some(ResourceType::Energy))
                                        > 0
                                    {
                                        creep.harvest(&source)
                                    } else {
                                        Err(ErrorCode::Full)
                                    }
                                }
                                Err(ErrorCode::Busy) => Err(ErrorCode::Busy),
                                Err(_) => Err(ErrorCode::Busy),
                            }
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
                CreepTarget::TransferToStorage(id) => {
                    let storage = game::get_object_by_id_typed(&id).unwrap();
                    creep.transfer(&storage, ResourceType::Energy, None)
                }
                CreepTarget::Upgrade(id) => {
                    let controller = game::get_object_by_id_typed(&id).unwrap();
                    creep.upgrade_controller(&controller)
                }
                CreepTarget::TransferToContainer(id) => {
                    let container = game::get_object_by_id_typed(&id).unwrap();
                    creep.transfer(&container, ResourceType::Energy, None)
                }
                CreepTarget::TransferToTower(id) => {
                    let tower = game::get_object_by_id_typed(&id).unwrap();
                    creep.transfer(&tower, ResourceType::Energy, None)
                }
                CreepTarget::FetchFromTower(id) => {
                    let tower = game::get_object_by_id_typed(&id).unwrap();
                    creep.withdraw(&tower, ResourceType::Energy, None)
                }
                CreepTarget::TransferToLink(id) => {
                    let link = game::get_object_by_id_typed(&id).unwrap();
                    creep.transfer(&link, ResourceType::Energy, None)
                }
                CreepTarget::FetchFromLink(id) => {
                    let link = game::get_object_by_id_typed(&id).unwrap();
                    creep.withdraw(&link, ResourceType::Energy, None)
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
                        if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
                            Ok(())
                        } else {
                            Err(ErrorCode::Busy)
                        }
                    }
                }
                CreepTarget::FetchFromSource(id) => {
                    if let CreepType::Miner(_) = self.career {
                        match self.no {
                            0 => {
                                if let Some(_) = find_container(
                                    &creep,
                                    Some(Position::new(
                                        RoomCoordinate::new(4).unwrap(),
                                        RoomCoordinate::new(45).unwrap(),
                                        RoomName::new("E36N7").unwrap(),
                                    )),
                                    ActionCommand::Transfer,
                                    None,
                                ) {
                                    Err(ErrorCode::Busy)
                                } else {
                                    Ok(())
                                }
                            }
                            1 => {
                                if let Some(_) = find_container(
                                    &creep,
                                    Some(Position::new(
                                        RoomCoordinate::new(41).unwrap(),
                                        RoomCoordinate::new(5).unwrap(),
                                        RoomName::new("E36N7").unwrap(),
                                    )),
                                    ActionCommand::Transfer,
                                    None,
                                ) {
                                    Err(ErrorCode::Busy)
                                } else {
                                    if creep.store().get_free_capacity(Some(ResourceType::Energy))
                                        == 0
                                    {
                                        Ok(())
                                    } else {
                                        Err(ErrorCode::Busy)
                                    }
                                }
                            }
                            _ => Err(ErrorCode::Busy),
                        }
                    } else {
                        if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                            self.setup_working_status(true);
                            Ok(())
                        } else if game::get_object_by_id_typed(&id).unwrap().energy() == 0 {
                            Ok(())
                        } else {
                            Err(ErrorCode::Busy)
                        }
                    }
                }
                CreepTarget::FetchFromContainer(id) => {
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                        self.setup_working_status(true);
                        Ok(())
                    } else if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy))
                        < self.career.carry_cnt() as u32 * 50
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::FetchFromStorage(id) => {
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                        self.setup_working_status(true);
                        Ok(())
                    } else if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy))
                        < self.career.carry_cnt() as u32 * 50
                    {
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
                        || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0
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
                        || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::TransferToStorage(id) => {
                    if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        == 0
                        || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::TransferToContainer(id) => {
                    if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        == 0
                        || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::TransferToTower(id) => {
                    if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        == 0
                        || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::FetchFromTower(id) => {
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                        self.setup_working_status(true);
                        Ok(())
                    } else if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy))
                        < self.career.carry_cnt() as u32 * 50
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::TransferToLink(id) => {
                    if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_free_capacity(Some(ResourceType::Energy))
                        == 0
                        || creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::FetchFromLink(id) => {
                    if creep.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                        self.setup_working_status(true);
                        Ok(())
                    } else if game::get_object_by_id_typed(&id)
                        .unwrap()
                        .store()
                        .get_used_capacity(Some(ResourceType::Energy))
                        < self.career.carry_cnt() as u32 * 50
                    {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
                CreepTarget::Upgrade(_) => {
                    if creep.store().get_used_capacity(Some(ResourceType::Energy)) == 0 {
                        Ok(())
                    } else {
                        Err(ErrorCode::Busy)
                    }
                }
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
        if let Some(id) = self.id {
            game::get_object_by_id_typed(&id)
        } else {
            None
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

    fn repair_road(&self) -> Result<(), ErrorCode> {
        let mut ret: Result<(), ErrorCode> = Ok(());
        let creep = self.get_creep().unwrap();
        let structures = creep.pos().look_for(look::STRUCTURES);
        if let Ok(structures_) = structures {
            let road = structures_
                .iter()
                .find(|s| s.structure_type() == StructureType::Road);
            if let Some(road_) = road {
                let r: StructureRoad = road_.clone().try_into().unwrap();
                if r.hits_max() - r.hits() >= self.career.work_cnt() as u32 * 100
                    && creep.store().get_used_capacity(Some(ResourceType::Energy))
                        > self.career.work_cnt() as u32
                {
                    creep.repair(&r).unwrap_or(());
                    ret = Err(ErrorCode::Busy);
                }
            }
        }
        ret
    }

    fn repair_container(&self) -> Result<(), ErrorCode> {
        let mut ret: Result<(), ErrorCode> = Ok(());
        let creep = self.get_creep().unwrap();
        let structures = creep.pos().look_for(look::STRUCTURES);
        if let Ok(structures_) = structures {
            let container = structures_
                .iter()
                .find(|s| s.structure_type() == StructureType::Container);
            if let Some(container_) = container {
                let c: StructureContainer = container_.clone().try_into().unwrap();
                if c.hits_max() - c.hits() >= self.career.work_cnt() as u32 * 100
                    && creep.store().get_used_capacity(Some(ResourceType::Energy))
                        > self.career.work_cnt() as u32
                {
                    creep.repair(&c).unwrap_or(());
                    ret = Err(ErrorCode::Busy);
                }
                if c.store().get_free_capacity(Some(ResourceType::Energy)) == 0 {
                    ret = Err(ErrorCode::Full);
                }
            }
        }
        ret
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
