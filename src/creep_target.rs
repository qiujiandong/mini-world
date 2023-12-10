use screeps::*;

pub enum CreepTarget {
    Upgrade(ObjectId<StructureController>),            // upgrade
    Build(ObjectId<ConstructionSite>),                 // build
    TransferToSpawn(ObjectId<StructureSpawn>),         // transfer
    TransferToExtension(ObjectId<StructureExtension>), // transfer
    TransferToStorage(ObjectId<StructureStorage>),     // transfer
    TransferToContainer(ObjectId<StructureContainer>), // transfer
    FetchFromStorage(ObjectId<StructureStorage>),      // withdraw
    FetchFromContainer(ObjectId<StructureContainer>),  // withdraw
    FetchFromSource(ObjectId<Source>),                 // harvest
    Default,
}

pub enum ActionCommand {
    Fetch,
    Transfer,
}

impl CreepTarget {
    pub fn new(obj: ObjectWithPosition, act: Option<ActionCommand>) -> Self {
        match obj {
            ObjectWithPosition::StructureController(controller) => Self::Upgrade(controller.id()),
            ObjectWithPosition::Source(source) => Self::FetchFromSource(source.id()),
            ObjectWithPosition::ConstructionSite(cs) => Self::Build(cs.try_id().unwrap()),
            ObjectWithPosition::StructureSpawn(spawn) => Self::TransferToSpawn(spawn.id()),
            ObjectWithPosition::StructureExtension(extension) => {
                Self::TransferToExtension(extension.id())
            }
            ObjectWithPosition::StructureStorage(storage) => match act {
                Some(ActionCommand::Fetch) => Self::FetchFromStorage(storage.id()),
                Some(ActionCommand::Transfer) => Self::TransferToStorage(storage.id()),
                None => Self::Default,
            },
            ObjectWithPosition::StructureContainer(container) => match act {
                Some(ActionCommand::Fetch) => Self::FetchFromContainer(container.id()),
                Some(ActionCommand::Transfer) => Self::TransferToContainer(container.id()),
                None => Self::Default,
            },
            _ => Self::Default,
        }
    }

    pub fn pos(&self) -> Option<Position> {
        match self {
            Self::Upgrade(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::Build(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::TransferToSpawn(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::TransferToExtension(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::TransferToStorage(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::TransferToContainer(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::FetchFromStorage(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::FetchFromContainer(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            Self::FetchFromSource(id) => Some(game::get_object_by_id_typed(&id).unwrap().pos()),
            _ => None,
        }
    }
}

pub fn find_source(creep: &Creep, pos: Option<Position>) -> Option<CreepTarget> {
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
        Some(CreepTarget::new(
            ObjectWithPosition::from(src_.clone()),
            None,
        ))
    } else {
        None
    }
}

pub fn find_container(
    creep: &Creep,
    pos: Option<Position>,
    act: ActionCommand,
    amount: Option<u32>,
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
                c.store().get_used_capacity(Some(ResourceType::Energy)) >= amount.unwrap_or(0)
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
    act: ActionCommand,
    amount: Option<u32>,
) -> Option<CreepTarget> {
    let room = creep.clone().room().unwrap();
    let structures = room.find(find::STRUCTURES, None);

    // find storage
    let storage = structures.iter().find(|s| {
        if s.structure_type() == StructureType::Storage {
            let c: StructureStorage = (*s).clone().try_into().unwrap();
            match act {
                ActionCommand::Fetch => {
                    c.store().get_used_capacity(Some(ResourceType::Energy)) >= amount.unwrap_or(0)
                }
                ActionCommand::Transfer => {
                    c.store().get_free_capacity(Some(ResourceType::Energy)) > 0
                }
            }
        } else {
            false
        }
    });
    if let Some(storage_) = storage {
        if let Some(pos_) = pos {
            if !storage_.pos().is_equal_to(pos_) {
                return None;
            } else {
            }
        } else {
        }
        let s: StructureStorage = storage_.clone().try_into().unwrap();
        Some(CreepTarget::new(ObjectWithPosition::from(s), Some(act)))
    } else {
        None
    }
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
