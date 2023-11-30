use screeps::*;

pub enum CreepAction {
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
    Default,
    Fetch,
    Transfer,
}

impl CreepAction {
    pub fn new(obj: &ObjectWithPosition, act: ActionCommand) -> Self {
        match obj {
            ObjectWithPosition::StructureController(controller) => Self::Upgrade(controller.id()),
            ObjectWithPosition::Source(source) => Self::FetchFromSource(source.id()),
            ObjectWithPosition::ConstructionSite(cs) => Self::Build(cs.try_id().unwrap()),
            ObjectWithPosition::StructureSpawn(spawn) => Self::TransferToSpawn(spawn.id()),
            ObjectWithPosition::StructureExtension(extension) => {
                Self::TransferToExtension(extension.id())
            }
            ObjectWithPosition::StructureStorage(storage) => match act {
                ActionCommand::Fetch => Self::FetchFromStorage(storage.id()),
                ActionCommand::Transfer => Self::TransferToStorage(storage.id()),
                ActionCommand::Default => Self::Default,
            },
            ObjectWithPosition::StructureContainer(container) => match act {
                ActionCommand::Fetch => Self::FetchFromContainer(container.id()),
                ActionCommand::Transfer => Self::TransferToContainer(container.id()),
                ActionCommand::Default => Self::FetchFromContainer(container.id()),
            },
            _ => Self::Default,
        }
    }
}

pub struct CreepTarget {
    pos: Position,
    pub action: CreepAction,
}

impl CreepTarget {
    pub fn new(obj: &ObjectWithPosition) -> Self {
        Self {
            pos: obj.pos(),
            action: CreepAction::new(obj, ActionCommand::Default),
        }
    }

    pub fn get_pos(&self) -> Position {
        self.pos.clone()
    }
}

pub fn find_available_energy(room: &Room) -> Option<CreepTarget> {
    let structures = room.find(find::STRUCTURES, None);
    if let Some(structure) = structures.iter().find(|s| {
        if let StructureObject::StructureContainer(container) = s {
            container
                .store()
                .get_used_capacity(Some(ResourceType::Energy))
                > 0
        } else {
            false
        }
    }) {
        if let StructureObject::StructureContainer(container) = structure {
            let obj = ObjectWithPosition::from(container.clone());
            Some(CreepTarget::new(&obj))
        } else {
            None
        }
    } else {
        let sources = room.find(find::SOURCES_ACTIVE, None);
        if sources.len() > 0 {
            let obj = ObjectWithPosition::from(sources[0].clone());
            Some(CreepTarget::new(&obj))
        } else {
            None
        }
    }
}

pub fn find_construction_site(room: &Room) -> Option<CreepTarget> {
    let cs = room.find(find::CONSTRUCTION_SITES, None);
    if cs.len() > 0 {
        let obj;
        if let Some(cs_) = cs
            .iter()
            .find(|cs| cs.structure_type() == StructureType::Container)
        {
            obj = ObjectWithPosition::from(cs_.clone());
        } else {
            obj = ObjectWithPosition::from(cs[0].clone());
        }
        Some(CreepTarget::new(&obj))
    } else {
        None
    }
}

pub fn find_notfull_spawn_or_extension(room: &Room) -> Option<CreepTarget> {
    let spawns = room.find(find::MY_SPAWNS, None);
    if spawns.len() > 0 {
        if spawns[0]
            .store()
            .get_free_capacity(Some(ResourceType::Energy))
            > 50
        {
            let obj = ObjectWithPosition::from(spawns[0].clone());
            Some(CreepTarget::new(&obj))
        } else {
            let structures = room.find(find::STRUCTURES, None);
            if let Some(structure) = structures.iter().find(|s| {
                if let StructureObject::StructureExtension(ext) = s {
                    ext.store().get_free_capacity(Some(ResourceType::Energy)) > 0
                } else {
                    false
                }
            }) {
                if let StructureObject::StructureExtension(ext) = structure {
                    let obj = ObjectWithPosition::from(ext.clone());
                    Some(CreepTarget::new(&obj))
                } else {
                    None
                }
            } else {
                None
            }
        }
    } else {
        None
    }
}

pub fn find_controller(room: &Room) -> Option<CreepTarget> {
    let structures = room.find(find::STRUCTURES, None);
    for structure in structures.iter() {
        if let StructureObject::StructureController(constroller) = structure {
            let obj = ObjectWithPosition::from(constroller.clone());
            return Some(CreepTarget::new(&obj));
        }
    }
    None
}
