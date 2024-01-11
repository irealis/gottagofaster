use bevy::{pbr::NotShadowCaster, prelude::*};

use crate::timing::MapDuration;
#[cfg(not(target_arch = "wasm32"))]
use bevy_hanabi::prelude::*;

use crate::{
    character_controller::{Grounded, JumpCount, Sliding},
    Player,
};

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, center_sky.run_if(in_state(crate::State::Playing)));

        #[cfg(not(target_arch = "wasm32"))]
        app.add_systems(
            Update,
            (emit_jump_effect, emit_ground_effect).run_if(in_state(crate::State::Playing)),
        );
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(clippy::type_complexity)]
pub fn emit_ground_effect(
    mut effect: Query<(&mut EffectSpawner, &mut EffectProperties, &mut Transform), Without<Player>>,
    query: Query<
        &Transform,
        (
            With<Player>,
            With<MapDuration>,
            Or<(Added<Sliding>, Added<Grounded>)>,
        ),
    >,
) {
    if let Ok(ptransform) = query.get_single() {
        if let Ok((mut spawner, mut properties, mut transform)) = effect.get_single_mut() {
            // encoded as `0xAABBGGRR`
            properties.set("particle_color", Vec4::new(1., 0., 0., 1.).into());
            transform.translation = ptransform.translation;
            spawner.reset();
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn emit_jump_effect(
    mut effect: Query<(&mut EffectSpawner, &mut EffectProperties, &mut Transform), Without<Player>>,
    player: Query<(&Transform, &JumpCount), (With<Player>, Changed<JumpCount>)>,
) {
    if let Ok((pt, jump_count)) = player.get_single() {
        if jump_count.0 > 0 {
            if let Ok((mut spawner, mut properties, mut transform)) = effect.get_single_mut() {
                //properties.set("particle_color", 0xFF02D2FC_u32.into());
                properties.set("particle_color", Vec4::new(1.5, 0.8, 0., 1.).into());
                transform.translation = pt.translation;
                spawner.reset();
            }
        }
    }
}

pub fn center_sky(
    player: Query<&Transform, With<Player>>,
    mut sky: Query<&mut Transform, (With<NotShadowCaster>, Without<Player>)>,
) {
    let player = player.get_single();

    if let Ok(player) = player {
        let mut sky = sky.single_mut();

        // Center sky on player so the box just moves with it
        sky.translation = player.translation;
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_portal() -> EffectAsset {
    let mut color_gradient1 = Gradient::new();
    color_gradient1.add_key(0.0, Vec4::new(2.0, 2.0, 4.0, 1.0));
    color_gradient1.add_key(0.1, Vec4::new(0.0, 0.0, 3.0, 1.0));
    color_gradient1.add_key(0.9, Vec4::new(4.0, 4.0, 4.0, 1.0));
    color_gradient1.add_key(1.0, Vec4::new(4.0, 4.0, 4.0, 0.0));

    let mut size_gradient1 = Gradient::new();
    size_gradient1.add_key(0.3, Vec2::new(0.2, 0.02));
    size_gradient1.add_key(1.0, Vec2::splat(0.0));

    let writer = ExprWriter::new();

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(1.).expr(),
        dimension: ShapeDimension::Surface,
    };

    let age = writer.lit(0.).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    // Give a bit of variation by randomizing the lifetime per particle
    let lifetime = writer.lit(0.6).uniform(writer.lit(1.3)).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    // Add drag to make particles slow down a bit after the initial acceleration
    let drag = writer.lit(2.).expr();
    let update_drag = LinearDragModifier::new(drag);

    let mut module = writer.finish();

    let tangent_accel = TangentAccelModifier::constant(&mut module, Vec3::ZERO, Vec3::Z, 30.);

    EffectAsset::new(32768, Spawner::rate(5000.0.into()), module)
        .with_name("portal")
        .with_simulation_space(SimulationSpace::Local)
        .init(init_pos)
        .init(init_age)
        .init(init_lifetime)
        .update(update_drag)
        .update(tangent_accel)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient1,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient1,
            screen_space_size: false,
        })
        .render(OrientModifier {
            mode: OrientMode::AlongVelocity,
            rotation: None,
        })
}

#[cfg(not(target_arch = "wasm32"))]
pub fn create_ground_effect() -> EffectAsset {
    let mut gradient = Gradient::new();
    gradient.add_key(0.0, Vec4::new(0.1, 0.1, 1., 1.));
    gradient.add_key(1.0, Vec4::splat(0.));

    let writer = ExprWriter::new();

    let color = writer.prop("particle_color").expr();
    let init_color = SetAttributeModifier::new(Attribute::HDR_COLOR, color);

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.05).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        axis: writer.lit(Vec3::Y).expr(),
        center: writer.lit(Vec3::ZERO).expr(),
        speed: writer.lit(16.).expr(),
    };

    let mut size_gradient = Gradient::new();
    size_gradient.add_key(0.3, Vec2::new(0.3, 0.3));
    size_gradient.add_key(1.0, Vec2::splat(0.0));

    let lifetime = writer.lit(2.);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr());

    let accel = writer.lit(Vec3::new(0., 16., 0.));
    let update_accel = AccelModifier::new(accel.expr());

    // Create the effect asset
    EffectAsset::new(
        // Maximum number of particles alive at a time
        32768,
        Spawner::rate(5.0.into()),
        // Move the expression module into the asset
        writer.finish(),
    )
    .with_name("ground effect")
    .with_property("particle_color", Vec4::new(1., 1., 1., 1.).into())
    .init(init_pos)
    .init(init_vel)
    .init(init_lifetime)
    .init(init_color)
    .update(update_accel)
    // Render the particles with a color gradient over their
    // lifetime. This maps the gradient key 0 to the particle spawn
    // time, and the gradient key 1 to the particle death (10s).
    // .render(ColorOverLifetimeModifier { gradient })
    .render(SizeOverLifetimeModifier {
        gradient: size_gradient,
        screen_space_size: false,
    })
    .render(OrientModifier {
        mode: OrientMode::FaceCameraPosition,
        rotation: None,
    })
}
