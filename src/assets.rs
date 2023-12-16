use bevy::prelude::*;

#[derive(Resource)]
pub struct AssetHandles {
    pub tori: Handle<Scene>,
    pub fox: Handle<Scene>,
}

#[derive(Resource)]
pub struct Animations(pub Vec<Handle<AnimationClip>>);

impl AssetHandles {
    pub fn load(aserv: &AssetServer) -> Self {
        let tori = aserv.load("tori.glb#Scene0");
        let fox = aserv.load("Fox.gltf#Scene0");

        Self { tori, fox }
    }
}
