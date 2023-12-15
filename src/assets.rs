use bevy::prelude::*;

#[derive(Resource)]
pub struct AssetHandles {
    pub tori: Handle<Scene>,
}

impl AssetHandles {
    pub fn load(aserv: &AssetServer) -> Self {
        let tori = aserv.load("tori.glb#Scene0");
        Self { tori }
    }
}
