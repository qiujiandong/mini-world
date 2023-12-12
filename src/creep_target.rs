use screeps::*;

pub enum CreepTarget {
    // controller
    Upgrade(ObjectId<StructureController>), // upgrade

    // constructionsite
    Build(ObjectId<ConstructionSite>), // build

    // spawn
    TransferToSpawn(ObjectId<StructureSpawn>),

    // extension
    TransferToExtension(ObjectId<StructureExtension>),

    // storage
    TransferToStorage(ObjectId<StructureStorage>),
    FetchFromStorage(ObjectId<StructureStorage>),

    // container
    TransferToContainer(ObjectId<StructureContainer>),
    FetchFromContainer(ObjectId<StructureContainer>),

    // tower
    TransferToTower(ObjectId<StructureTower>),
    FetchFromTower(ObjectId<StructureTower>),

    // link
    TransferToLink(ObjectId<StructureLink>),
    FetchFromLink(ObjectId<StructureLink>),

    // source
    FetchFromSource(ObjectId<Source>),

    Default,
}

#[derive(Clone, Copy)]
pub enum ActionCommand {
    Fetch,
    Transfer,
}

impl CreepTarget {
    pub fn new(obj: ObjectWithPosition, act: Option<ActionCommand>) -> Self {
        match obj {
            // controller
            ObjectWithPosition::StructureController(controller) => Self::Upgrade(controller.id()),

            // construction_site
            ObjectWithPosition::ConstructionSite(construction_site) => {
                Self::Build(construction_site.try_id().unwrap())
            }

            // spawn
            ObjectWithPosition::StructureSpawn(spawn) => Self::TransferToSpawn(spawn.id()),

            // extension
            ObjectWithPosition::StructureExtension(extension) => {
                Self::TransferToExtension(extension.id())
            }

            // storage
            ObjectWithPosition::StructureStorage(storage) => match act {
                Some(ActionCommand::Fetch) => Self::FetchFromStorage(storage.id()),
                Some(ActionCommand::Transfer) => Self::TransferToStorage(storage.id()),
                None => Self::Default,
            },

            // container
            ObjectWithPosition::StructureContainer(container) => match act {
                Some(ActionCommand::Fetch) => Self::FetchFromContainer(container.id()),
                Some(ActionCommand::Transfer) => Self::TransferToContainer(container.id()),
                None => Self::Default,
            },

            // tower
            ObjectWithPosition::StructureTower(tower) => match act {
                Some(ActionCommand::Transfer) => Self::TransferToTower(tower.id()),
                Some(ActionCommand::Fetch) => Self::FetchFromTower(tower.id()),
                _ => Self::Default,
            },

            // link
            ObjectWithPosition::StructureLink(link) => match act {
                Some(ActionCommand::Transfer) => Self::TransferToLink(link.id()),
                Some(ActionCommand::Fetch) => Self::FetchFromLink(link.id()),
                _ => Self::Default,
            },

            // source
            ObjectWithPosition::Source(source) => match act {
                Some(ActionCommand::Fetch) => Self::FetchFromSource(source.id()),
                Some(ActionCommand::Transfer) => Self::Default,
                None => Self::Default,
            },

            _ => Self::Default,
        }
    }

    pub fn pos(&self) -> Option<Position> {
        match self {
            // controller
            Self::Upgrade(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            // construction_site
            Self::Build(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            // spawn
            Self::TransferToSpawn(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            // extension
            Self::TransferToExtension(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            // storage
            Self::TransferToStorage(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::FetchFromStorage(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            // container
            Self::TransferToContainer(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::FetchFromContainer(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            // tower
            Self::TransferToTower(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::FetchFromTower(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            // link
            Self::TransferToLink(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::FetchFromLink(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            // source
            Self::FetchFromSource(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),

            _ => None,
        }
    }
}

pub fn find_source(creep: &Creep, pos: Option<Position>) -> Option<CreepTarget> {
    let mut ret: Option<CreepTarget> = None;
    let src;
    let room = creep.clone().room().unwrap();
    let sources = room.find(find::SOURCES_ACTIVE, None);

    match pos {
        Some(pos_) => {
            src = sources.iter().find(|s| s.pos().is_equal_to(pos_));
        }
        None => {
            src = sources
                .iter()
                .min_by_key(|a| a.pos().get_range_to(creep.pos()))
        }
    }
    if let Some(src_) = src {
        ret = Some(CreepTarget::new(
            ObjectWithPosition::from(src_.clone()),
            Some(ActionCommand::Fetch),
        ));
    }
    ret
}

pub fn find_container(
    creep: &Creep,
    pos: Option<Position>,
    act: ActionCommand,
    amount: Option<u16>,
) -> Option<CreepTarget> {
    let container;
    let room = creep.clone().room().unwrap();
    let structures = room.find(find::STRUCTURES, None);

    // find all containers
    let containers: Vec<_> = structures
        .iter()
        .filter(|s| {
            if s.structure_type() == StructureType::Container {
                let c: StructureContainer = (*s).clone().try_into().unwrap();
                match act {
                    ActionCommand::Transfer => {
                        c.store().get_free_capacity(Some(ResourceType::Energy))
                            >= amount.unwrap_or(1) as i32
                    }
                    ActionCommand::Fetch => {
                        c.store().get_used_capacity(Some(ResourceType::Energy))
                            >= amount.unwrap_or(1) as u32
                    }
                }
            } else {
                false
            }
        })
        .collect();
    match pos {
        Some(pos_) => {
            container = containers.iter().find(|s| s.pos().is_equal_to(pos_));
        }
        None => {
            container = containers
                .iter()
                .min_by_key(|a| a.pos().get_range_to(creep.pos()))
        }
    }

    if let Some(container_) = container {
        let c: StructureContainer = (*container_).clone().try_into().unwrap();
        Some(CreepTarget::new(ObjectWithPosition::from(c), Some(act)))
    } else {
        None
    }
}

pub fn find_storage(
    creep: &Creep,
    pos: Option<Position>,
    act: Option<ActionCommand>,
    amount: Option<u16>,
) -> Option<CreepTarget> {
    let mut ret: Option<CreepTarget> = None;
    let room = creep.clone().room().unwrap();
    let structures = room.find(find::STRUCTURES, None);
    // find storage
    for storage in structures
        .iter()
        .filter(|s| s.structure_type() == StructureType::Storage)
    {
        let s: StructureStorage = storage.clone().try_into().unwrap();
        // check pos
        if let Some(pos_) = pos {
            if !s.pos().is_equal_to(pos_) {
                continue;
            }
        }
        // check energy
        if let Some(act_) = act {
            match act_ {
                ActionCommand::Fetch => {
                    if s.store().get_used_capacity(Some(ResourceType::Energy))
                        >= amount.unwrap_or(0) as u32
                    {
                        ret = Some(CreepTarget::new(
                            ObjectWithPosition::from(s),
                            Some(ActionCommand::Fetch),
                        ));
                        break;
                    }
                }
                ActionCommand::Transfer => {
                    if s.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        ret = Some(CreepTarget::new(
                            ObjectWithPosition::from(s),
                            Some(ActionCommand::Transfer),
                        ));
                        break;
                    }
                }
            }
        } else {
            ret = Some(CreepTarget::new(ObjectWithPosition::from(s), None));
        }
    }
    ret
}

pub fn find_construction_site(
    creep: &Creep,
    cs_type: Option<StructureType>,
    pos: Option<Position>,
) -> Option<CreepTarget> {
    let room = creep.clone().room().unwrap();
    let construction_sites = room.find(find::CONSTRUCTION_SITES, None);

    let targets: Vec<_> = construction_sites
        .iter()
        .filter(|cs| {
            if let None = cs_type {
                if let None = pos {
                    true
                } else {
                    pos.unwrap().is_equal_to(cs.pos())
                }
            } else {
                if let None = pos {
                    cs_type.unwrap() == cs.structure_type()
                } else {
                    cs_type.unwrap() == cs.structure_type() && pos.unwrap().is_equal_to(cs.pos())
                }
            }
        })
        .collect();
    let target = targets
        .iter()
        .min_by_key(|cs| cs.pos().get_range_to(creep.pos()));

    if let Some(construction_site) = target {
        let cs = (*construction_site).clone();
        Some(CreepTarget::new(ObjectWithPosition::from(cs), None))
    } else {
        None
    }
}

pub fn find_notfull_spawn_or_extension(creep: &Creep) -> Option<CreepTarget> {
    let room = creep.clone().room().unwrap();
    let structures = room.find(find::STRUCTURES, None);

    // find spawn or extension with not full store
    let targets: Vec<_> = structures
        .iter()
        .filter(|s| match s.structure_type() {
            StructureType::Spawn => {
                let s: StructureSpawn = (*s).clone().try_into().unwrap();
                s.store().get_free_capacity(Some(ResourceType::Energy)) > 0
            }
            StructureType::Extension => {
                let s: StructureExtension = (*s).clone().try_into().unwrap();
                s.store().get_free_capacity(Some(ResourceType::Energy)) > 0
            }
            _ => false,
        })
        .collect();

    let target = targets
        .iter()
        .min_by_key(|t| t.pos().get_range_to(creep.pos()));

    if let Some(target_) = target {
        match target_.structure_type() {
            StructureType::Spawn => {
                let s: StructureSpawn = (*target_).clone().try_into().unwrap();
                Some(CreepTarget::new(
                    ObjectWithPosition::from(s),
                    Some(ActionCommand::Transfer),
                ))
            }
            StructureType::Extension => {
                let s: StructureExtension = (*target_).clone().try_into().unwrap();
                Some(CreepTarget::new(
                    ObjectWithPosition::from(s),
                    Some(ActionCommand::Transfer),
                ))
            }
            _ => None,
        }
    } else {
        None
    }
}

pub fn find_controller(creep: &Creep) -> Option<CreepTarget> {
    let room = creep.clone().room().unwrap();
    let structures = room.find(find::STRUCTURES, None);
    let controller = structures
        .iter()
        .find(|s| s.structure_type() == StructureType::Controller);
    if let Some(controler_) = controller {
        let c: StructureController = controler_.clone().try_into().unwrap();
        Some(CreepTarget::new(ObjectWithPosition::from(c), None))
    } else {
        None
    }
}

pub fn find_tower(
    creep: &Creep,
    pos: Option<Position>,
    act: Option<ActionCommand>,
    amount: Option<u16>,
) -> Option<CreepTarget> {
    let mut ret: Option<CreepTarget> = None;
    let room = creep.clone().room().unwrap();
    let structures = room.find(find::STRUCTURES, None);
    for tower in structures
        .iter()
        .filter(|s| s.structure_type() == StructureType::Tower)
    {
        let t: StructureTower = tower.clone().try_into().unwrap();
        // check pos
        if let Some(pos_) = pos {
            if !t.pos().is_equal_to(pos_) {
                continue;
            }
        }
        // check energy
        if let Some(act_) = act {
            match act_ {
                ActionCommand::Fetch => {
                    if t.store().get_used_capacity(Some(ResourceType::Energy))
                        >= amount.unwrap_or(0) as u32
                    {
                        ret = Some(CreepTarget::new(
                            ObjectWithPosition::from(t),
                            Some(ActionCommand::Fetch),
                        ));
                        break;
                    }
                }
                ActionCommand::Transfer => {
                    if t.store().get_free_capacity(Some(ResourceType::Energy)) > 0 {
                        ret = Some(CreepTarget::new(
                            ObjectWithPosition::from(t),
                            Some(ActionCommand::Transfer),
                        ));
                        break;
                    }
                }
            }
        }
    }
    ret
}

pub fn find_link(
    creep: &Creep,
    pos: Option<Position>,
    act: Option<ActionCommand>,
    amount: Option<u16>,
) -> Option<CreepTarget> {
    let mut ret: Option<CreepTarget> = None;
    let room = creep.clone().room().unwrap();
    let structures = room.find(find::STRUCTURES, None);
    // find storage
    for link in structures
        .iter()
        .filter(|s| s.structure_type() == StructureType::Link)
    {
        let lnk: StructureLink = link.clone().try_into().unwrap();
        // check pos
        if let Some(pos_) = pos {
            if !lnk.pos().is_equal_to(pos_) {
                continue;
            }
        }
        // check energy
        if let Some(act_) = act {
            match act_ {
                ActionCommand::Fetch => {
                    if lnk.store().get_used_capacity(Some(ResourceType::Energy))
                        >= amount.unwrap_or(1) as u32
                    {
                        ret = Some(CreepTarget::new(
                            ObjectWithPosition::from(lnk),
                            Some(ActionCommand::Fetch),
                        ));
                        break;
                    }
                }
                ActionCommand::Transfer => {
                    if lnk.store().get_free_capacity(Some(ResourceType::Energy))
                        > amount.unwrap_or(1) as i32
                    {
                        ret = Some(CreepTarget::new(
                            ObjectWithPosition::from(lnk),
                            Some(ActionCommand::Transfer),
                        ));
                        break;
                    }
                }
            }
        } else {
            ret = Some(CreepTarget::new(ObjectWithPosition::from(lnk), None));
        }
    }
    ret
}
