use bevy_xpbd_3d::prelude::PhysicsLayer;

#[derive(PhysicsLayer)]
pub enum PhysicsLayers {
    Player,
    Sensor,
    Ground,
}
