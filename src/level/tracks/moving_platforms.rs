use crate::level::{
    common::{self, Param},
    utils::{BASE_Y, Geometry, TextureAssets, TrackOffsets},
};
use avian3d::prelude::{Collider, RigidBody};
use bevy::{
    animation::{
        AnimationClip, AnimationPlayer, AnimationTarget, AnimationTargetId, animated_field,
    },
    prelude::*,
};
use std::{collections::HashMap, f32::consts::PI};

// --- Plugin Definition ---
pub struct MovingPlatformsTrackPlugin;

impl Plugin for MovingPlatformsTrackPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup_moving_platforms_track.after(super::super::load_assets_and_setup),
        );
    }
}

// --- Constants ---
const TRACK_NAME: &str = "MovingPlatforms";
const TRACK_Z: f32 = 20.0;
const TEX_PLATFORM: usize = 4 * 13 + 3;

// --- Parameter Ranges ---
// Vertical
const V_PARAMS: &[(&str, Param)] = &[
    (
        "v_dist",
        Param::Float {
            start: 3.0,
            end: 7.0,
            step: 2.0,
        },
    ),
    (
        "v_dur",
        Param::Float {
            start: 4.0,
            end: 4.0,
            step: 2.0,
        },
    ),
];
const V_PLAT_SIZE: Vec3 = Vec3::new(3.0, 0.3, 3.0);

// Horizontal
const H_PARAMS: &[(&str, Param)] = &[
    (
        "h_dist",
        Param::Float {
            start: 4.0,
            end: 8.0,
            step: 4.0,
        },
    ),
    (
        "h_dur",
        Param::Float {
            start: 4.0,
            end: 4.0,
            step: 2.0,
        },
    ),
];
const H_PLAT_SIZE: Vec3 = Vec3::new(4.0, 0.3, 2.5);

// Rotating
const R_PARAMS: &[(&str, Param)] = &[
    (
        "r_angle",
        Param::Float {
            start: PI / 2.0,
            end: PI * 1.5,
            step: PI / 2.0,
        },
    ),
    (
        "r_dur",
        Param::Float {
            start: 8.0,
            end: 8.0,
            step: 4.0,
        },
    ),
];
const R_PLAT_SIZE: Vec3 = Vec3::new(3.5, 0.3, 3.5);

// --- Setup System ---
fn setup_moving_platforms_track(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut track_offsets: ResMut<TrackOffsets>,
    level_assets: Res<TextureAssets>,
    mut animation_clips: ResMut<Assets<AnimationClip>>,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
) {
    info!("Generating track: {}", TRACK_NAME);

    // --- Vertical Platforms ---
    let vertical_generator =
        |permutation: &HashMap<String, f64>,
         cmds: &mut Commands,
         mshs: &mut ResMut<Assets<Mesh>>,
         mats: &mut ResMut<Assets<StandardMaterial>>,
         offsets: &mut ResMut<TrackOffsets>,
         assets: &Res<TextureAssets>,
         clips: &mut ResMut<Assets<AnimationClip>>,
         graphs: &mut ResMut<Assets<AnimationGraph>>| {
            let v_dist = permutation["v_dist"] as f32;
            let v_dur = permutation["v_dur"] as f32;
            let name = format!("VPlatform_d{:.1}_t{:.1}", v_dist, v_dur);
            spawn_moving_platform_vertical_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                clips,
                graphs,
                &name,
                V_PLAT_SIZE,
                v_dist,
                v_dur,
                TEX_PLATFORM,
            );
        };
    common::generate_permutations(
        V_PARAMS,
        vertical_generator,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );

    // --- Horizontal Platforms ---
    let horizontal_generator =
        |permutation: &HashMap<String, f64>,
         cmds: &mut Commands,
         mshs: &mut ResMut<Assets<Mesh>>,
         mats: &mut ResMut<Assets<StandardMaterial>>,
         offsets: &mut ResMut<TrackOffsets>,
         assets: &Res<TextureAssets>,
         clips: &mut ResMut<Assets<AnimationClip>>,
         graphs: &mut ResMut<Assets<AnimationGraph>>| {
            let h_dist = permutation["h_dist"] as f32;
            let h_dur = permutation["h_dur"] as f32;
            let name = format!("HPlatform_d{:.1}_t{:.1}", h_dist, h_dur);
            spawn_moving_platform_horizontal_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                clips,
                graphs,
                &name,
                H_PLAT_SIZE,
                h_dist,
                h_dur,
                TEX_PLATFORM,
            );
        };
    common::generate_permutations(
        H_PARAMS,
        horizontal_generator,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );

    // --- Rotating Platforms ---
    let rotating_generator =
        |permutation: &HashMap<String, f64>,
         cmds: &mut Commands,
         mshs: &mut ResMut<Assets<Mesh>>,
         mats: &mut ResMut<Assets<StandardMaterial>>,
         offsets: &mut ResMut<TrackOffsets>,
         assets: &Res<TextureAssets>,
         clips: &mut ResMut<Assets<AnimationClip>>,
         graphs: &mut ResMut<Assets<AnimationGraph>>| {
            let r_angle = permutation["r_angle"] as f32;
            let r_dur = permutation["r_dur"] as f32;
            let name = format!("RPlatform_a{:.1}pi_t{:.1}", r_angle / PI, r_dur);
            spawn_moving_platform_rotating_instance(
                cmds,
                mshs,
                mats,
                offsets,
                assets,
                clips,
                graphs,
                &name,
                R_PLAT_SIZE,
                r_angle,
                r_dur,
                TEX_PLATFORM,
            );
        };
    common::generate_permutations(
        R_PARAMS,
        rotating_generator,
        &mut commands,
        &mut meshes,
        &mut materials,
        &mut track_offsets,
        &level_assets,
        &mut animation_clips,
        &mut animation_graphs,
    );
}

// --- Instance Spawners (Mostly unchanged, just use TRACK_Z) ---

fn spawn_moving_platform_vertical_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    name: &str,
    size: Vec3,
    vertical_distance: f32,
    animation_duration_one_way: f32,
    texture_index: usize,
) {
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, size.x);
    let platform_start_pos = Vec3::new(section_center_x, BASE_Y + size.y / 2.0, TRACK_Z);
    let platform_end_pos = platform_start_pos + Vec3::Y * vertical_distance;
    let platform_name = Name::new(name.to_string());
    let target_id = AnimationTargetId::from_name(&platform_name);

    // --- Animation Setup ---
    let mut clip = AnimationClip::default();
    if let Ok(interval) = Interval::new(0.0, animation_duration_one_way) {
        if let Ok(curve) = EasingCurve::new(
            platform_start_pos,
            platform_end_pos,
            EaseFunction::SineInOut,
        )
        .reparametrize_linear(interval)
        {
            if let Ok(ping_pong_curve) = curve.ping_pong() {
                let animatable_curve =
                    AnimatableCurve::new(animated_field!(Transform::translation), ping_pong_curve);
                clip.add_curve_to_target(target_id, animatable_curve);
            } else {
                warn!("Failed ping-pong for {}", name);
            }
        } else {
            warn!("Failed reparametrize for {}", name);
        }
    } else {
        warn!("Failed interval creation for {}", name);
    }

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();
    // --- End Animation Setup ---

    let platform_entity = common::spawn_kinematic_cuboid(
        // Use kinematic helper
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        size,
        Transform::from_translation(platform_start_pos),
        texture_index,
    );

    // Add animation components
    commands.entity(platform_entity).insert((
        platform_name, // Add Name back for target resolution
        AnimationGraphHandle(graph_handle),
        player,
        AnimationTarget {
            id: target_id,
            player: platform_entity,
        },
    ));
}

fn spawn_moving_platform_horizontal_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    name: &str,
    size: Vec3,
    horizontal_distance: f32,
    animation_duration_one_way: f32,
    texture_index: usize,
) {
    let footprint_x = size.x + horizontal_distance;
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, footprint_x);
    let platform_start_pos = Vec3::new(
        section_center_x - horizontal_distance / 2.0,
        BASE_Y + size.y / 2.0,
        TRACK_Z,
    );
    let platform_end_pos = platform_start_pos + Vec3::X * horizontal_distance;
    let platform_name = Name::new(name.to_string());
    let target_id = AnimationTargetId::from_name(&platform_name);

    // --- Animation Setup ---
    let mut clip = AnimationClip::default();
    if let Ok(interval) = Interval::new(0.0, animation_duration_one_way) {
        if let Ok(curve) = EasingCurve::new(
            platform_start_pos,
            platform_end_pos,
            EaseFunction::SineInOut,
        )
        .reparametrize_linear(interval)
        {
            if let Ok(ping_pong_curve) = curve.ping_pong() {
                let animatable_curve =
                    AnimatableCurve::new(animated_field!(Transform::translation), ping_pong_curve);
                clip.add_curve_to_target(target_id, animatable_curve);
            } else {
                warn!("Failed ping-pong for {}", name);
            }
        } else {
            warn!("Failed reparametrize for {}", name);
        }
    } else {
        warn!("Failed interval creation for {}", name);
    }

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();
    // --- End Animation Setup ---

    let platform_entity = common::spawn_kinematic_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        size,
        Transform::from_translation(platform_start_pos),
        texture_index,
    );

    commands.entity(platform_entity).insert((
        platform_name,
        AnimationGraphHandle(graph_handle),
        player,
        AnimationTarget {
            id: target_id,
            player: platform_entity,
        },
    ));
}

fn spawn_moving_platform_rotating_instance(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    track_offsets: &mut ResMut<TrackOffsets>,
    level_assets: &Res<TextureAssets>,
    animation_clips: &mut ResMut<Assets<AnimationClip>>,
    animation_graphs: &mut ResMut<Assets<AnimationGraph>>,
    name: &str,
    size: Vec3,
    rotation_angle_y: f32,
    animation_duration_cycle: f32,
    texture_index: usize,
) {
    let section_center_x = track_offsets.get_and_advance(TRACK_NAME, size.x);
    let platform_pos = Vec3::new(section_center_x, BASE_Y + size.y / 2.0, TRACK_Z);
    let start_rotation = Quat::IDENTITY;
    let end_rotation = Quat::from_rotation_y(rotation_angle_y);
    let platform_name = Name::new(name.to_string());
    let target_id = AnimationTargetId::from_name(&platform_name);

    // --- Animation Setup ---
    let mut clip = AnimationClip::default();
    if let Ok(interval) = Interval::new(0.0, animation_duration_cycle) {
        if let Ok(curve) = EasingCurve::new(start_rotation, end_rotation, EaseFunction::Linear)
            .reparametrize_linear(interval)
        {
            let animatable_curve =
                AnimatableCurve::new(animated_field!(Transform::rotation), curve);
            clip.add_curve_to_target(target_id, animatable_curve);
        } else {
            warn!("Failed reparametrize for {}", name);
        }
    } else {
        warn!("Failed interval creation for {}", name);
    }

    let clip_handle = animation_clips.add(clip);
    let (graph, node_index) = AnimationGraph::from_clip(clip_handle);
    let graph_handle = animation_graphs.add(graph);
    let mut player = AnimationPlayer::default();
    player.play(node_index).repeat();
    // --- End Animation Setup ---

    let platform_entity = common::spawn_kinematic_cuboid(
        commands,
        meshes,
        materials,
        level_assets,
        name.to_string(),
        size,
        Transform::from_translation(platform_pos),
        texture_index,
    );

    commands.entity(platform_entity).insert((
        platform_name,
        AnimationGraphHandle(graph_handle),
        player,
        AnimationTarget {
            id: target_id,
            player: platform_entity,
        },
    ));
}
