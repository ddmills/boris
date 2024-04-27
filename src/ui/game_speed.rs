use bevy::ecs::system::Resource;

#[derive(Resource)]
pub struct GameSpeed {
    pub speed: f32,
    pub is_paused: bool,
}

impl GameSpeed {
    pub fn speed(&self) -> f32 {
        if self.is_paused {
            0.
        } else {
            self.speed
        }
    }
}

impl Default for GameSpeed {
    fn default() -> Self {
        Self {
            speed: 1.,
            is_paused: false,
        }
    }
}
