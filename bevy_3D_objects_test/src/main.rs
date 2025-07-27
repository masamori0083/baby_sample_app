use avian3d::dynamics::rigid_body::LinearVelocity;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_trenchbroom::class::builtin::*;
use bevy_trenchbroom::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins::default())
        .add_plugins(
            TrenchBroomPlugins(
                TrenchBroomConfig::new("bevy_3D_objects_test")
                    .default_solid_spawn_hooks(|| SpawnHooks::new().convex_collider()),
            )
            .build(),
        )
        .register_type::<InfoPlayerStart>()
        .override_class::<FuncGroup>()
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                player_movement,
                debug_loaded_entities,
                debug_scene_loading,
                debug_info_player_start,
                spawn_player_at_spawn_point,
                camera_follow_player,
            ),
        )
        .run();
}

// プレイヤーのスポーンポイントを定義
#[point_class]
#[derive(Component, Default, Debug, Clone, Reflect)]
#[reflect(Component, QuakeClass)]
struct InfoPlayerStart {
    angle: f32, // プレイヤーのスポーン角度
}

#[solid_class(base(Worldspawn),hooks(SpawnHooks::new().convex_collider()))]
pub struct FuncGroup;

fn debug_loaded_entities(query: Query<&Transform, Added<FuncGroup>>) {
    for transform in query.iter() {
        println!("FuncGroup loaded at: {:?}", transform.translation);
    }
}

fn debug_scene_loading(query: Query<&SceneRoot, Added<SceneRoot>>) {
    for scene_root in query.iter() {
        println!("SceneRoot loaded: {:?}", scene_root);
    }
}

fn debug_info_player_start(query: Query<(&InfoPlayerStart, &Transform), Added<InfoPlayerStart>>) {
    for (info, transform) in query.iter() {
        println!(
            "✅ InfoPlayerStart loaded at: {:?}, angle: {}",
            transform.translation, info.angle
        );
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct MainCamera;

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((AmbientLight {
        color: Color::WHITE,
        brightness: 1000.0,
        affects_lightmapped_meshes: false,
    },));

    // 後でマップをロードする
    commands.spawn(SceneRoot(asset_server.load("maps/complete_map.map#Scene")));

    // カメラを俯瞰位置に追加
    commands.spawn((
        MainCamera,
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // directional lightを追加
    commands.spawn((
        DirectionalLight {
            illuminance: 10000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, -0.5, 0.0, 0.0)),
    ));

    // point lightを追加
    commands.spawn((
        Transform::from_xyz(0.0, 4.0, 0.0),
        PointLight {
            shadows_enabled: true,
            range: 500.0,
            color: Color::srgb(1.0, 0.0, 0.0), // 赤色の光
            intensity: 100000.0,
            ..default()
        },
    ));
}

/// プレイヤーの動きとカメラの追従を制御
fn player_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut LinearVelocity, &Transform), With<Player>>,
    camera_query: Query<&Transform, (With<MainCamera>, Without<Player>)>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let Ok((mut linear_velocity, transform)) = query.single_mut() else {
        return;
    };

    let speed = 5.0;
    let mut direction = Vec3::ZERO;

    if keyboard_input.pressed(KeyCode::KeyW) {
        direction += *camera_transform.forward();
    }
    if keyboard_input.pressed(KeyCode::KeyS) {
        direction += *camera_transform.back();
    }
    if keyboard_input.pressed(KeyCode::KeyA) {
        direction += *camera_transform.left();
    }
    if keyboard_input.pressed(KeyCode::KeyD) {
        direction += *camera_transform.right();
    }

    if direction.length_squared() > 0.0 {
        direction = direction.normalize() * speed;

        linear_velocity.0.x = direction.x;
        linear_velocity.0.z = direction.z;
    } else {
        // 入力がなければ速度をゼロにする
        linear_velocity.0.x = 0.0;
        linear_velocity.0.z = 0.0;
    }
    println!("プレイヤーの位置: {:?}", transform.translation);
}

fn camera_follow_player(
    player_query: Query<(&Transform, &LinearVelocity), (With<Player>, Without<MainCamera>)>,
    mut camera_query: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    time: Res<Time>,
) {
    let Ok((player_transform, player_velocity)) = player_query.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera_query.single_mut() else {
        return;
    };

    // プレイヤーの背後、少し上にカメラを配置
    let desired_position = player_transform.translation + Vec3::new(0.0, 0.5, 2.0); // プレイヤーの背後0.5m、上に2m

    // カメラの位置を滑らかにプレイヤーに追従
    let lerp_factor = 8.0 * time.delta_secs();
    camera_transform.translation = camera_transform
        .translation
        .lerp(desired_position, lerp_factor);

    // カメラの回転をプレイヤーの向きに合わせる
    // プレイヤーの進行方向を取得(速度の向き)
    let horizontal_velocity = Vec3::new(player_velocity.0.x, 0.0, player_velocity.0.z);
    if horizontal_velocity.length_squared() > 0.0 {
        // プレイヤーの進行方向にカメラを向ける
        let look_target = player_transform.translation + horizontal_velocity.normalize();
        camera_transform.look_at(look_target + Vec3::Y * 1.0, Vec3::Y);
    } 
    println!(
        "カメラ位置: {:?}, プレイヤー位置: {:?}",
        camera_transform.translation, player_transform.translation
    );
}

fn spawn_player_at_spawn_point(
    mut commands: Commands,
    spawn_query: Query<(&InfoPlayerStart, &Transform), Added<InfoPlayerStart>>,
    player_query: Query<(), With<Player>>, // 追加：プレイヤーの存在チェック
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 既にプレイヤーが存在する場合はスポーンしない
    // すでにプレイヤーが存在するなら即リターン
    if !player_query.is_empty() {
        return;
    }
    // プレイヤーのスポーンポイントを取得
    for (spawn, transform) in spawn_query.iter() {
        println!("🚩 プレイヤースポーン位置: {:?}", transform.translation);
        // プレイヤーをスポーンポイントの位置にスポーン
        commands.spawn((
            Player,
            RigidBody::Dynamic,
            Collider::capsule(0.5, 1.0), // プレイヤーのサイズ
            LockedAxes::ROTATION_LOCKED, // 回転をロック
            LinearVelocity(Vec3::ZERO),  // 初期速度はゼロ
            TransformInterpolation,      // Avianでなめらかに補完
            Transform {
                translation: transform.translation + Vec3::Y * 1.0, // 少し上に配置
                rotation: Quat::from_rotation_y(spawn.angle.to_radians()),
                ..default()
            },
            Mesh3d(meshes.add(Capsule3d::new(0.5, 1.0))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.3, 0.6, 1.0), // 明るい青色のプレイヤー
                ..default()
            })),
        ));
        println!(
            "✅ プレイヤーをスポーンしました: {:?}",
            transform.translation
        );
    }
}
