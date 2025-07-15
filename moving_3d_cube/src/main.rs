//! # シンプルな3D世界探索ゲーム
//!
//! このゲームはRust製ゲームエンジン「Bevy」を使ったシンプルな3Dゲームです。
//! プレイヤーはキューブとして生成されるキャラクターを操作して、
//! 自由に地面の上を探索できます。
//!
//! ## 主な機能
//! - 3Dの世界（土台）の生成
//! - 昼と夜の時間帯を切り替える機能（キーボードの`T`キーで切り替え）
//! - キーボード入力によるプレイヤーの操作
//!   - `W` `A` `S` `D`キーで前後左右に移動可能
//!
//! ## 今後の拡張予定
//! - 3時間の周期での昼夜の自動切り替え
//! - インタラクティブな要素やNPCの追加
//! - より詳細な地形生成や探索可能なオブジェクトの導入

use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Component)] // キューブを識別するためのマーカーコンポーネント
struct MovingCube;

#[derive(Resource, PartialEq, Eq, Debug, Clone, Copy)]
enum Daytime {
    Day,
    Night,
}

/// 無限世界を表現するためのチャンク
/// チャンクは、地形の一部を表現するための単位
/// ここでは、チャンクのX座標とZ座標を保持する
#[derive(Component)]
struct GroundChunk {
    chunk_x: i32, // チャンクのX座標
    chunk_z: i32, // チャンクのZ座標
}

#[derive(Resource)]
struct InfiniteWorld {
    chunk_size: f32,      // チャンクのサイズ
    render_distance: i32, // レンダリング距離
}

/// チャンクオブジェクトを識別するためのマーカーコンポーネント
/// これにより、チャンクの位置を特定し、管理することができるようになる
/// チャンクは、地形の一部を表現するための単位であり、
/// ここでは、チャンクのX座標とZ座標を保持する
#[derive(Component)]
struct ChunkObject {
    chunk_x: i32, // チャンクのX座標
    chunk_z: i32, // チャンクのZ座標
}

/// 昼夜の状態を管理するリソース
#[derive(Resource)]
struct DayNightSettings {
    day: EnvironmentSettings,
    night: EnvironmentSettings,
}

/// 環境設定を定義する構造体
/// これにより、昼と夜の光源や環境光の設定を
/// 一元管理できるようにする
#[derive(Clone, Copy)]
struct EnvironmentSettings {
    directional_light_intensity: f32,
    directional_light_color: Color,
    ambient_light_brightness: f32,
    ambient_light_color: Color,
    sky_color: Color, // 空の色を追加
}

/// プレイヤーを識別するためのマーカーコンポーネント
#[derive(Component)]
struct Player;

/// カメラコントローラー
#[derive(Component)]
struct CameraController {
    offset: Vec3,      // カメラのオフセット位置
    follow_speed: f32, // プレイヤーに追従する速度
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Daytime::Day) // 初期状態は昼
        .insert_resource(DayNightSettings {
            day: EnvironmentSettings {
                directional_light_intensity: 10000.0,
                directional_light_color: Color::WHITE,
                ambient_light_brightness: 100.0,
                ambient_light_color: Color::WHITE,
                sky_color: Color::srgb(0.6, 0.8, 0.95), // 昼の空の色
            },
            night: EnvironmentSettings {
                directional_light_intensity: 500.0,
                directional_light_color: Color::linear_rgb(0.2, 0.3, 0.7),
                ambient_light_brightness: 30.0,
                ambient_light_color: Color::linear_rgb(0.2, 0.3, 0.6),
                sky_color: Color::srgb(0.1, 0.1, 0.3), // 夜の空の色
            },
        })
        .insert_resource(InfiniteWorld {
            chunk_size: 20.0,   // チャンクのサイズ
            render_distance: 2, // レンダリング距離（2x2のグリッド）
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                toggle_day_night,
                player_movement,
                camera_follow_player,
                manage_infinite_world,
            ),
        )
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 環境光を設定
    commands.insert_resource(ClearColor(Color::srgb(0.6, 0.8, 0.95)));

    // カメラを生成する
    commands.spawn((
        Camera::default(),
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        CameraController {
            offset: Vec3::new(0.0, 5.0, 10.0), // カメラのオフセット位置
            follow_speed: 2.0,                 // プレイヤーに追従する速度
        },
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    // 世界の土台を生成する
    // let plane_mesh = meshes.add(Mesh::from(Plane3d::default().mesh().size(20.0, 20.0)));
    // let plane_material = materials.add(StandardMaterial::from(Color::srgb(0.3, 0.5, 0.3)));
    // commands.spawn((
    //     Mesh3d(plane_mesh),
    //     MeshMaterial3d(plane_material),
    //     Transform::default(),
    //     Visibility::default(),
    //     InheritedVisibility::default(),
    //     ViewVisibility::default(),
    // ));

    // 無限に広がる地形を生成する
    let chunk_size = 20.0; // チャンクのサイズ
    for x in -1..=1 {
        for z in -1..=1 {
            spawn_ground_chunk(&mut commands, &mut meshes, &mut materials, x, z, chunk_size);
        }
    }

    // プレイヤーのキューブを生成
    let cube_handle = meshes.add(Cuboid::from_length(1.0));
    commands.spawn((
        Mesh3d(cube_handle),
        MeshMaterial3d(materials.add(Color::srgb(0.0, 0.0, 0.0))),
        Transform::from_xyz(0.0, 0.5, 0.0),
        Player,
    ));

    // 光源を生成
    commands.spawn((
        PointLight {
            intensity: 10000.0,
            shadows_enabled: true,
            range: 100.0,
            ..default()
        },
        Transform::from_xyz(0.0, 10.0, 0.0),
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));
}

/// カメラ追従システム
fn camera_follow_player(
    mut camera_query: Query<(&mut Transform, &CameraController), (With<Camera3d>, Without<Player>)>,
    player_query: Query<&Transform, With<Player>>,
    time: Res<Time>,
) {
    if let Ok(player_transform) = player_query.single() {
        for (mut camera_transform, controller) in &mut camera_query {
            // プレイヤーの位置にオフセットを加えた位置にカメラを配置
            let target_position = player_transform.translation + controller.offset;

            // カメラの位置をスムーズにプレイヤーに追従させる(線形補完)
            camera_transform.translation = camera_transform.translation.lerp(
                target_position,
                controller.follow_speed * time.delta_secs(), // 追従速度を調整
            );

            // カメラは常にプレイヤーを向く
            camera_transform.look_at(player_transform.translation, Vec3::Y);
        }
    }
}

/// 昼夜を切り替えるシステム
fn toggle_day_night(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut daytime: ResMut<Daytime>,
    settings: Res<DayNightSettings>,
    mut lights: Query<&mut DirectionalLight>,
    mut ambient: ResMut<AmbientLight>,
    mut clear_color: ResMut<ClearColor>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyT) {
        println!("Tキーが押されました。昼夜を切り替えます。");

        *daytime = match *daytime {
            Daytime::Day => Daytime::Night,
            Daytime::Night => Daytime::Day,
        };

        // 設定を選択
        let current_settings = match *daytime {
            Daytime::Day => settings.day,
            Daytime::Night => settings.night,
        };

        // DirectionalLightを変更
        for mut light in &mut lights {
            light.illuminance = current_settings.directional_light_intensity;
            light.color = current_settings.directional_light_color;
            println!(
                "Set DirectionalLight: intensity={}, color={:?}",
                light.illuminance, light.color
            );
        }

        // AmbientLightを変更
        ambient.color = current_settings.ambient_light_color;
        ambient.brightness = current_settings.ambient_light_brightness;
        println!(
            "Set AmbientLight: brightness={}, color={:?}",
            ambient.brightness, ambient.color
        );

        // 空の色を変更
        clear_color.0 = current_settings.sky_color;
        println!("Set ClearColor: {:?}", clear_color.0);
    }
}

/// プレイヤーの移動を制御するシステム
/// キューブを作成し、ユーザーの入力に応じて移動させる

fn player_movement(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut Transform, With<Player>>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;

    // プレイヤーの移動速度
    let speed = 5.0;

    // 入力に応じてプレイヤーを移動
    if keyboard.pressed(KeyCode::ArrowUp) {
        direction.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowDown) {
        direction.z += 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    for mut transform in &mut query {
        transform.translation += direction.normalize_or_zero() * speed * time.delta_secs();
    }
}

/// 無限に広がる地形を生成するシステム
/// チャンクを生成し、プレイヤーの位置に応じてチャンクを配置する
fn spawn_ground_chunk(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    chunk_x: i32,
    chunk_z: i32,
    chunk_size: f32,
) {
    // チャンクのメッシュを生成
    let world_x = chunk_x as f32 * chunk_size;
    let world_z = chunk_z as f32 * chunk_size;

    // 地面のチャンクを生成
    let plane_mesh = meshes.add(Mesh::from(
        Plane3d::default().mesh().size(chunk_size, chunk_size),
    ));
    let plane_material = materials.add(StandardMaterial::from(Color::srgb(0.4, 0.7, 0.4)));

    commands.spawn((
        Mesh3d(plane_mesh),
        MeshMaterial3d(plane_material),
        Transform::from_xyz(world_x, 0.0, world_z),
        GroundChunk { chunk_x, chunk_z },
        Visibility::default(),
        InheritedVisibility::default(),
        ViewVisibility::default(),
    ));

    // チャンクオブジェクトを識別するためのマーカーコンポーネントを追加
    // 中央チャンクにはオブジェクトを配置しない
    if chunk_x != 0 || chunk_z != 0 {
        let cube_mesh = meshes.add(Cuboid::from_length(2.0));
        // チャンクの座標に応じて色を変える
        let idx = (chunk_x.abs() + chunk_z.abs()) % 4; // チャンクの座標に基づいて色を決定
        let color = match idx {
            0 => Color::srgb(0.8, 0.2, 0.2), // 赤系
            1 => Color::srgb(0.2, 0.8, 0.2), // 緑系
            2 => Color::srgb(0.2, 0.2, 0.8), // 青系
            _ => Color::srgb(0.8, 0.8, 0.2), // 黄系
        };
        // キューブのマテリアルを生成
        let cube_material = materials.add(StandardMaterial::from(color));

        // チャンクの位置にキューブを配置
        commands.spawn((
            Mesh3d(cube_mesh),
            MeshMaterial3d(cube_material),
            Transform::from_xyz(world_x, 0.5, world_z), // 少し上に配置
            ChunkObject { chunk_x, chunk_z },
            Visibility::default(),
            InheritedVisibility::default(),
            ViewVisibility::default(),
        ));

        // チャンクの情報をログに出力
        println!(
            "チャンク生成: ({}, {}) at world position ({}, {})",
            chunk_x, chunk_z, world_x, world_z
        );
    } else {
        println!(
            "中央チャンクはオブジェクトを生成しません: ({}, {})",
            chunk_x, chunk_z
        );
    }
}

/// 無限世界のチャンクを管理するシステム
/// プレイヤーの位置に応じてチャンクを生成・削除する
fn manage_infinite_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<(Entity, &GroundChunk, &Transform)>,
    object_query: Query<(Entity, &ChunkObject)>,
    world_settings: Res<InfiniteWorld>,
) {
    if let Ok(player_transform) = player_query.single() {
        // プレイヤーのチャンク座標を計算
        let player_chunk_x =
            (player_transform.translation.x / world_settings.chunk_size).floor() as i32;
        let player_chunk_z =
            (player_transform.translation.z / world_settings.chunk_size).floor() as i32;

        println!(
            "プレイヤーのチャンク座標: ({}, {})",
            player_chunk_x, player_chunk_z
        );

        // 現在存在するチャンクを収集
        let mut existing_chunks: HashSet<(i32, i32)> = HashSet::new();
        let mut chunks_to_remove = Vec::new();
        let mut objects_to_remove = Vec::new();

        for (entity, chunk, _) in &chunk_query {
            existing_chunks.insert((chunk.chunk_x, chunk.chunk_z));

            // 描画はにがいのチャンクを削除対象に追加
            let distance_x = (chunk.chunk_x - player_chunk_x).abs();
            let distance_z = (chunk.chunk_z - player_chunk_z).abs();
            let max_distance = distance_x.max(distance_z);

            // 描画距離をworld_settings.render_distanceに設定
            if max_distance > world_settings.render_distance {
                chunks_to_remove.push(entity);
                println!(
                    "チャンク削除予定: ({}, {}) at distance {}",
                    chunk.chunk_x, chunk.chunk_z, max_distance
                );
            }
        }

        // オブジェクトの削除対象を収集
        for (entity, object) in &object_query {
            // チャンクオブジェクトの座標を取得
            let distance_x = (object.chunk_x - player_chunk_x).abs();
            let distance_z = (object.chunk_z - player_chunk_z).abs();
            let max_distance = distance_x.max(distance_z);

            // 描画距離をworld_settings.render_distanceに設定
            if max_distance > world_settings.render_distance {
                objects_to_remove.push(entity);
                println!(
                    "チャンクオブジェクト削除予定: ({}, {}) at distance {}",
                    object.chunk_x, object.chunk_z, max_distance
                );
            }
        }
        //不要なチャンクを削除
        for entity in chunks_to_remove {
            commands.entity(entity).despawn();
            println!("チャンク削除: {:?}", entity);
        }
        for entity in objects_to_remove {
            commands.entity(entity).despawn();
            println!("チャンクオブジェクト削除: {:?}", entity);
        }

        // 新しいチャンクを生成
        for x in (player_chunk_x - world_settings.render_distance)
            ..=(player_chunk_x + world_settings.render_distance)
        {
            for z in (player_chunk_z - world_settings.render_distance)
                ..=(player_chunk_z + world_settings.render_distance)
            {
                if !existing_chunks.contains(&(x, z)) {
                    spawn_ground_chunk(
                        &mut commands,
                        &mut meshes,
                        &mut materials,
                        x,
                        z,
                        world_settings.chunk_size,
                    );
                }
            }
        }
    }
}
