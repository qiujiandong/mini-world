use screeps::*;

pub fn link_tx_from_mine() {
    let structures = game::rooms()
        .get(RoomName::new("E36N7").unwrap())
        .unwrap()
        .find(find::STRUCTURES, None);
    let src = structures.iter().find(|s| {
        s.pos().is_equal_to(Position::new(
            RoomCoordinate::new(40).unwrap(),
            RoomCoordinate::new(4).unwrap(),
            RoomName::new("E36N7").unwrap(),
        )) && s.structure_type() == StructureType::Link
    });
    let dst = structures.iter().find(|s| {
        s.pos().is_equal_to(Position::new(
            RoomCoordinate::new(8).unwrap(),
            RoomCoordinate::new(31).unwrap(),
            RoomName::new("E36N7").unwrap(),
        )) && s.structure_type() == StructureType::Link
    });
    if let (Some(src), Some(dst)) = (src, dst) {
        let src_: StructureLink = src.clone().try_into().unwrap();
        let dst_: StructureLink = dst.clone().try_into().unwrap();
        if src_.store().get_free_capacity(Some(ResourceType::Energy)) == 0
            && dst_.store().get_used_capacity(Some(ResourceType::Energy)) == 0
        {
            src_.transfer_energy(&dst_, None).unwrap_or(());
        }
    }
}
