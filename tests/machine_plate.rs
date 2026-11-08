//! The machine plate: every system this crate adds is in [`LightsSystems`], every public component
//! is reflected as a component, and the authoring components carry what an editor needs.
//!
//! `bevy_lights` has no public `Resource` named `*Settings`, so there is no settings list here: the
//! tuning resources (`FlameLook`, `FlameWind`, `FlameGust`, `FlameMotion`, `LightPool`) keep their own
//! reflection and are not saved by name.
//!
//! System names are the full path only with Bevy's `debug` feature, which the dev-dependency turns on;
//! the test asserts it found crate systems so a build without the feature cannot pass vacuously.

use std::any::TypeId;

use bevy::ecs::schedule::Schedules;
use bevy::prelude::*;
use bevy_lights::*;

const CRATE: &str = "bevy_lights::";

/// The crate's group on an app that can host `FlamePlugin`: it adds materials, so the app needs the
/// asset and mesh machinery of a render app, but no window and no GPU.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), TransformPlugin))
        .init_asset::<Mesh>()
        .init_asset::<Image>()
        .init_asset::<Shader>()
        .add_plugins(LightsPlugins);
    app.update();
    app
}

#[test]
fn machine_plate_every_system_is_in_the_root_set() {
    let app = app();
    let sets = [
        LightsSystems::Fire,
        LightsSystems::Motion,
        LightsSystems::Pool,
        LightsSystems::Flicker,
        LightsSystems::Flame,
        LightsSystems::Report,
    ]
    .map(|set| set.intern());
    let schedules = app.world().resource::<Schedules>();
    let mut seen = 0;
    let mut stray = Vec::new();
    for (_label, schedule) in schedules.iter() {
        let Ok(systems) = schedule.systems() else { continue };
        let members: Vec<_> =
            sets.iter().filter_map(|set| schedule.graph().systems_in_set(*set).ok()).collect();
        for (key, system) in systems {
            let name = system.name().to_string();
            if !name.starts_with(CRATE) {
                continue;
            }
            seen += 1;
            if !members.iter().any(|m| m.contains(&key)) {
                stray.push(name);
            }
        }
    }
    assert!(seen > 0, "no system named `{CRATE}*` found; is Bevy's `debug` feature on?");
    assert!(stray.is_empty(), "systems outside LightsSystems: {stray:?}");
}

#[test]
fn machine_plate_components_are_reflected() {
    let app = app();
    let registry = app.world().resource::<AppTypeRegistry>().read();
    let components = [
        TypeId::of::<Flicker>(),
        TypeId::of::<Kindle>(),
        TypeId::of::<Snuff>(),
        TypeId::of::<Fire>(),
        TypeId::of::<FlameSite>(),
        TypeId::of::<Flame>(),
        TypeId::of::<FlameVelocity>(),
        TypeId::of::<FlameEmbers>(),
        TypeId::of::<FlameSmoke>(),
        TypeId::of::<FlameTongues>(),
        TypeId::of::<FlameTongue>(),
        TypeId::of::<FuelBed>(),
        TypeId::of::<PoolCandidate>(),
        TypeId::of::<PoolLight>(),
        TypeId::of::<PoolFocus>(),
    ];
    for id in components {
        let name = registry.get(id).map_or("<unregistered>", |r| r.type_info().type_path());
        assert!(registry.get_type_data::<ReflectComponent>(id).is_some(), "{name} has no ReflectComponent");
    }
}

#[test]
fn machine_plate_authoring_components_carry_default_and_serde() {
    let app = app();
    let registry = app.world().resource::<AppTypeRegistry>().read();
    for id in [TypeId::of::<Flicker>(), TypeId::of::<Fire>(), TypeId::of::<PoolFocus>()] {
        let name = registry.get(id).map_or("<unregistered>", |r| r.type_info().type_path());
        assert!(registry.get_type_data::<ReflectDefault>(id).is_some(), "{name} has no ReflectDefault");
        #[cfg(feature = "serde")]
        {
            assert!(
                registry.get_type_data::<ReflectSerialize>(id).is_some(),
                "{name} has no ReflectSerialize"
            );
            assert!(
                registry.get_type_data::<ReflectDeserialize>(id).is_some(),
                "{name} has no ReflectDeserialize"
            );
        }
    }
}
