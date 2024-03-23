use bevy::ecs::system::Resource;

#[derive(Resource, Default)]
pub struct DebugSettings {
    pub path: bool,
}
