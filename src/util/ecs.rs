use log::debug;

pub fn despawn_all<T: hecs::Query>(world: &mut hecs::World) {
    let to_despawn: Vec<hecs::Entity> = world.query::<T>().iter().map(|(e, _)| e).collect();

    debug!("Despawning {:?} entities", to_despawn);
    for e in to_despawn {
        world.despawn(e).expect("Could not despawn entity");
    }
}
