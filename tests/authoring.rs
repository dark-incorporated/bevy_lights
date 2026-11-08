//! The authoring components, inserted alone: a `Flicker` brings its light, a `Fire` brings its whole fire.
//! This is what a level editor does when it adds one component to a piece.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::time::Duration;

use bevy::light::{PointLight, SpotLight};
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use bevy_lights::*;

fn app() -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        TransformPlugin,
        FlickerPlugin,
        LightPoolPlugin::<PoolFocus>::default(),
        FirePlugin,
    ))
    .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(1.0 / 60.0)))
    .insert_resource(LightPool { size: 4, reach: 100.0, gain: 1.0, ..default() });
    app.update();
    app
}

#[test]
fn a_flicker_inserted_alone_brings_a_point_light() {
    let mut app = app();
    let torch = app.world_mut().spawn(Flicker { base: 900.0, ..default() }).id();
    app.update();
    let light = app.world().entity(torch).get::<PointLight>().expect("the flicker's own light");
    assert!(light.intensity > 0.0, "the flicker drives it: {}", light.intensity);
    let mut bare = app.world_mut().spawn_empty();
    bare.insert(Flicker::default());
    let bare = bare.id();
    app.update();
    assert!(app.world().entity(bare).contains::<PointLight>(), "the default flicker brings one too");
}

#[test]
fn a_flicker_beside_a_light_of_its_own_adds_none() {
    let mut app = app();
    let sconce = app.world_mut().spawn((Flicker::default(), SpotLight::default())).id();
    let lamp = app
        .world_mut()
        .spawn((Flicker { base: 800.0, ..default() }, PointLight { intensity: 5.0, ..default() }))
        .id();
    app.update();
    let world = app.world();
    assert!(!world.entity(sconce).contains::<PointLight>(), "a spot light stays alone");
    assert_eq!(
        world.entity(lamp).get::<PointLight>().map(|l| l.intensity > 0.0),
        Some(true),
        "the lamp's own light is the one driven"
    );
}

#[test]
fn a_fire_inserted_alone_becomes_a_flame_and_a_light_in_it() {
    let mut app = app();
    app.world_mut().spawn((PoolFocus, Transform::default()));
    let mut entity = app.world_mut().spawn_empty();
    entity.insert(Fire::default());
    let fire = entity.id();
    for _ in 0..6 {
        app.update();
    }
    let world = app.world();
    assert!(world.entity(fire).contains::<Transform>(), "a fire brings its own place");
    assert!(world.entity(fire).contains::<FlameSite>(), "and its flame");
    assert!(world.entity(fire).contains::<PoolCandidate>(), "and the light the pool hands it");
    let lit: Vec<Vec3> = app
        .world_mut()
        .query::<(&PoolLight, &Transform, &Flicker, &PointLight)>()
        .iter(app.world())
        .filter(|(_, _, flicker, _)| flicker.base > 0.0)
        .map(|(_, at, _, _)| at.translation)
        .collect();
    assert_eq!(lit.len(), 1, "one fire, one lit slot: {lit:?}");
    let middle = Vec3::from(flame_light_offset(FireKind::Torch.height()));
    assert!((lit[0] - middle).length() < 1e-3, "the light stands in the flame: {:?} vs {middle:?}", lit[0]);
}
