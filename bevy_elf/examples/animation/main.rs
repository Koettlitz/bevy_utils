// Example sprites: "DuskBorne Enemy #1 [Elf]" by KiyoZ, https://kiyoz.itch.io/duskborne-enemy-1

use bevy::prelude::*;
use bevy_elf::{AppExt, FromDef, ResolveStrategy, RonAssetLoader};
use std::time::Duration;

#[derive(FromDef, Asset, TypePath)]
struct AnimationAsset {
    frames: Vec<usize>,
    frame_duration: Duration,

    #[elf(from_default)]
    spritesheet: Spritesheet,
}

#[derive(FromDef)]
#[elf(def_type(()))]
struct Spritesheet {
    #[elf(implicit, with_spec(
        base_path = "images",
        resolve_strategy = ResolveStrategy::FileStemIsId { extension: "png" }
    ))]
    image: Handle<Image>,

    #[elf(implicit, with_spec(
        base_path = "layouts",
        resolve_strategy = ResolveStrategy::FileStemIsId { extension: "tl.ron" }
    ))]
    layout: Handle<TextureAtlasLayout>,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            file_path: "examples/animation/assets".into(),
            ..Default::default()
        }))
        .init_ron_asset::<AnimationAsset>()
        .init_asset_loader::<RonAssetLoader<TextureAtlasLayout>>()
        .add_systems(Startup, (trigger_loading, spawn_camera))
        .add_systems(
            Update,
            (
                spawn_sprite.run_if(resource_exists::<LoadingAnimation>),
                update_animation,
            ),
        )
        .run();
}

#[derive(Resource)]
struct LoadingAnimation(Handle<AnimationAsset>);

#[derive(Component)]
struct Animation {
    handle: Handle<AnimationAsset>,
    timer: Timer,
}

fn trigger_loading(asset_loader: Res<AssetServer>, mut commands: Commands) {
    let handle = asset_loader.load("animations/elf.ron");
    commands.insert_resource(LoadingAnimation(handle));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn spawn_sprite(
    asset_loader: Res<AssetServer>,
    loading_animation: Res<LoadingAnimation>,
    animations: Res<Assets<AnimationAsset>>,
    mut commands: Commands,
) {
    if !asset_loader
        .recursive_dependency_load_state(loading_animation.0.id())
        .is_loaded()
    {
        return;
    }

    let animation = animations.get(&loading_animation.0).unwrap();
    let atlas = TextureAtlas {
        layout: animation.spritesheet.layout.clone(),
        index: 0,
    };

    commands.spawn((
        Sprite::from_atlas_image(animation.spritesheet.image.clone(), atlas),
        Transform::from_scale(Vec3::new(2.0, 2.0, 1.0)),
        Animation {
            handle: loading_animation.0.clone(),
            timer: Timer::new(animation.frame_duration, TimerMode::Repeating),
        },
    ));

    commands.remove_resource::<LoadingAnimation>();
}

fn update_animation(
    sprites: Query<(&mut Sprite, &mut Animation)>,
    assets: Res<Assets<AnimationAsset>>,
    time: Res<Time>,
) {
    for (mut sprite, mut animation) in sprites {
        if animation.timer.tick(time.delta()).just_finished() {
            let asset = assets.get(animation.handle.id()).unwrap();
            if sprite.texture_atlas.as_ref().unwrap().index == asset.frames.len() - 1 {
                sprite.texture_atlas.as_mut().unwrap().index = 0;
            } else {
                sprite.texture_atlas.as_mut().unwrap().index += 1;
            }
        }
    }
}
