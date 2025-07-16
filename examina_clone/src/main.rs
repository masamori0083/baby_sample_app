use bevy::prelude::*;
use bevy::render::mesh::primitives::Capsule3dMeshBuilder;
use bevy_kira_audio::{Audio, AudioControl, AudioPlugin};
use bevy_rapier3d::prelude::*;

/// ゲームオーバーなどの状態を管理するリソース
#[derive(Resource, Debug, Clone, Eq, PartialEq, Hash, Default)]
enum GameState {
    #[default]
    Playing, // ゲームプレイ中
    GameOver, // ゲームオーバー状態
}

/// ゲームオーバーのUIを表示するシステム
#[derive(Component)]
struct GameOverUI;

/// プレイヤーキャラクターのコンポーネント
#[derive(Component)]
struct Player;

/// 敵キャラクターのコンポーネント
#[derive(Component)]
struct Enemy {
    vision_range: f32,           // 敵の視界範囲
    vision_angle: f32,           // 敵の視界角度
    patrol_points: Vec<Vec3>,    // パトロールポイント
    current_patrol_index: usize, // 現在のパトロールポイントのインデックス
    speed: f32,                  // 敵の移動速度
    initial_position: Vec3,      // 敵の初期位置
    initial_rotation: Quat,      // 敵の初期向き
}

/// カメラのオフセットを管理するコンポーネント
#[derive(Component)]
struct CameraController {
    height: f32,       // カメラの高さ
    distance: f32,     // プレイヤーからの距離
    min_distance: f32, // 最小距離
    max_distance: f32, // 最大距離
    zoom_speed: f32,   // ズーム速度
}

/// カメラコントローラーのデフォルト値
impl Default for CameraController {
    fn default() -> Self {
        Self {
            distance: 8.0,
            height: 3.0,
            min_distance: 3.0,
            max_distance: 20.0,
            zoom_speed: 5.0,
        }
    }
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            AudioPlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            RapierDebugRenderPlugin::default(),
        ))
        .init_resource::<GameState>() // ゲーム状態の初期化
        .add_systems(Startup, setup_scene)
        .add_systems(
            Update,
            (
                player_input,
                enemy_vision_system,
                enemy_patrol_system,
                camera_follow_player.after(player_input),
                camera_zoom,
                restart_game,
            ),
        )
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 地面(静的オブジェクト)
    commands.spawn((
        RigidBody::Fixed, // 静的リジットボディ
        Collider::cuboid(50.0, 0.1, 50.0),
        Mesh3d(meshes.add(Cuboid::new(100.0, 0.2, 100.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5), // 緑色の地面
            ..default()
        })),
        Transform::from_xyz(0.0, -0.1, 0.0),
    ));

    // 落下するキューブ(動的リジットボディ)
    commands.spawn((
        RigidBody::Dynamic,
        Collider::cuboid(0.5, 0.5, 0.5),
        Transform::from_xyz(0.0, 5.0, 0.0),
        GravityScale(1.0), // 重力の影響を受ける
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.8, 0.2, 0.2), // 赤色のキューブ
            ..default()
        })),
    ));

    // プレイヤーキャラクター(動的リジットボディ)
    commands.spawn((
        Player,
        RigidBody::KinematicPositionBased, // 動的リジットボディ
        Collider::capsule_y(0.9, 0.4),     // 高さ1.8m（半分の0.9）、半径0.4m
        KinematicCharacterController::default(), // キャラクターコントローラー
        Transform::from_xyz(0.0, 1.0, 0.0), // 初期位置
        Mesh3d(
            meshes.add(
                Capsule3dMeshBuilder::new(
                    /* radius */ 0.4, /* height between hemisphere centers */ 1.8,
                    /* longitudes */ 16, /* latitudes */ 8,
                )
                .build(),
            ),
        ), // プレイヤーのメッシュ
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.8, 0.2), // 緑色のプレイヤー
            ..default()
        })),
    ));

    // 敵キャラクター(視界を持つ動的リジットボディ)
    // 敵キャラクターの初期位置を設定
    let enemy_initial_position = Vec3::new(5.0, 1.0, 5.0); // 初期位置を設定
    let initial_target = Vec3::new(-5.0, 1.0, 5.0); // 初期ターゲット位置
    let enemyinitial_rotation = Quat::from_rotation_y(
				(initial_target - enemy_initial_position).angle_between(Vec3::Z),
		); // 初期向きを計算
    let enemy_entity = commands
        .spawn((
            Enemy {
                vision_range: 10.0,
                vision_angle: 45.0,
                patrol_points: vec![
                    Vec3::new(5.0, 1.0, 5.0),   // 1. 右前
                    Vec3::new(-5.0, 1.0, 5.0),  // 2. 左前
                    Vec3::new(-5.0, 1.0, -5.0), // 3. 左後
                    Vec3::new(5.0, 1.0, -5.0),  // 4. 右後
                ],
                current_patrol_index: 0,
                speed: 4.0,                               // 敵の移動速度
                initial_position: enemy_initial_position, // 敵の初期位置
                initial_rotation: enemyinitial_rotation,  // 敵の初期向き
            },
            RigidBody::KinematicPositionBased, // 動的リジットボディ
            Collider::capsule_y(0.9, 0.4),
            Transform {
                translation: enemy_initial_position,
                rotation: enemyinitial_rotation, // 初期向き
                ..default()
            },
            Mesh3d(
                meshes.add(
                    Capsule3dMeshBuilder::new(
                        /* radius */ 0.4, /* height between hemisphere centers */ 1.8,
                        /* longitudes */ 16, /* latitudes */ 8,
                    )
                    .build(),
                ),
            ),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.8, 0.2, 0.2), // 赤色の敵
                ..default()
            })),
        ))
        .id();

    // 敵の前面に「目」のようなマーカーを追加(子エンティティとして);
    commands.entity(enemy_entity).with_children(|parent| {
        parent.spawn((
            Mesh3d(meshes.add(Sphere::new(0.15))), // 小さな球を目として使用
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgb(0.2, 0.2, 0.8), // 青色の目
                ..default()
            })),
            Transform::from_xyz(0.0, 0.5, -0.4), // 敵の前面に配置
        ));
    });
    // カメラの設定
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0) // カメラの位置
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y), // カメラの向き設定
        CameraController::default(), // カメラコントローラーの初期化
    ));

    // ライトの設定
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true, // シャドウを有効にする
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0) // ライトの位置
            .looking_at(Vec3::ZERO, Vec3::Y), // ライトの向き設定
    ));
}

/// プレイヤー入力システム
fn player_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<&mut KinematicCharacterController, With<Player>>,
    camera_query: Query<&Transform, (With<Camera3d>, Without<Player>)>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    // ゲーム状態がGameOverの場合は何もしない
    if *game_state == GameState::GameOver {
        return; // ゲームオーバー状態ではプレイヤー入力を無視
    }

    // 動きを制御するための変数
    let Ok(camera_transform) = camera_query.single() else {
        return; // カメラが存在しない場合は何もしない
    };

    // カメラの前方向と右方向を取得（Vec3に変換）
    let forward = camera_transform.forward().as_vec3();
    let right = camera_transform.right().as_vec3();
    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::ArrowUp) {
        direction += forward;
    }

    if keys.pressed(KeyCode::ArrowDown) {
        direction -= forward;
    }

    if keys.pressed(KeyCode::ArrowLeft) {
        direction -= right;
    }

    if keys.pressed(KeyCode::ArrowRight) {
        direction += right;
    }

    direction.y = 0.0; // 垂直方向の動きを無効化

    // スニーキング判定
    let is_sneaking = keys.pressed(KeyCode::ShiftLeft);
    let base_speed = 5.0; // 基本速度を上げる
    let speed = if is_sneaking {
        base_speed * 0.3 // 30%の速度
    } else {
        base_speed
    };

    if direction.length_squared() > 0.0 {
        direction = direction.normalize() * speed * time.delta_secs();

        for mut controller in &mut query {
            controller.translation = Some(direction);
        }

        // デバッグ出力
        if is_sneaking {
            println!("🚶 Sneaking mode active! Speed: {}", speed);
        }
    } else {
        for mut controller in &mut query {
            controller.translation = Some(Vec3::ZERO);
        }
    }
}

/// 敵キャラクターの視界検知システム
fn enemy_vision_system(
    player_query: Query<&Transform, With<Player>>,
    enemy_query: Query<(&Transform, &Enemy)>,
    mut game_state: ResMut<GameState>,
    mut commands: Commands,
) {
    let Ok(player_transform) = player_query.single() else {
        return; // プレイヤーが存在しない場合は何もしない
    };

    // ゲーム状態がGameOverの場合は何もしない
    if *game_state == GameState::GameOver {
        return;
    }

    // 敵キャラクターの情報を取得
    for (enemy_transform, enemy) in enemy_query.iter() {
        // プレイヤーと敵の位置を取得
        let enemy_forward = enemy_transform.forward();
        let to_player = player_transform.translation - enemy_transform.translation;

        // プレイヤーとの距離を計算
        let distance_to_player = to_player.length();

        // プレイヤーが視界範囲外の場合は無視
        if distance_to_player > enemy.vision_range {
            continue;
        }

        // プレイヤーとの角度を計算
        let to_player_direction = to_player.normalize();
        let angle_to_player = enemy_forward
            .angle_between(to_player_direction)
            .to_degrees();

        // 敵に検知されたかどうかを判定
        if angle_to_player < enemy.vision_angle / 2.0 {
            // プレイヤーが視界内にいる場合の処理
            println!(
                "🔴 Enemy detected player at distance: {:.2} and angle: {:.2}",
                distance_to_player, angle_to_player
            );
            // ここに敵がプレイヤーを検知した際の処理を追加できる
            *game_state = GameState::GameOver; // ゲームオーバー状態に変更
            spawn_game_over_ui(&mut commands); // ゲームオーバーのUIを表示

            // 一度検知したらループを抜ける
            break;
        }
    }
}

/// 敵キャラクターのパトロールシステム
fn enemy_patrol_system(
    mut enemy_query: Query<(&mut Transform, &mut Enemy)>,
    time: Res<Time>,
    game_state: Res<GameState>,
) {
    // ゲーム状態がGameOverの場合は何もしない
    if *game_state == GameState::GameOver {
        return; // ゲームオーバー状態ではパトロールしない
    }

    // 敵キャラクターの情報を取得
    for (mut transform, mut enemy) in enemy_query.iter_mut() {
        // パトロールポイントが空の場合は何もしない
        if enemy.patrol_points.is_empty() {
            continue;
        }

        // 現在のパトロールポイントを取得
        let target_point = enemy.patrol_points[enemy.current_patrol_index];

        // 目標位置までのベクトルを計算
        let direction = (target_point - transform.translation).normalize();
        // 目標位置までの距離を計算
        let distance_to_target = transform.translation.distance(target_point);

        // 目標位置に近づいたら次のパトロールポイントへ移動
        if distance_to_target < 0.2 {
            enemy.current_patrol_index =
                (enemy.current_patrol_index + 1) % enemy.patrol_points.len();
        } else {
            // 目標位置に向かって移動
            transform.translation += direction * enemy.speed * time.delta_secs();
            // 敵の向きを目標位置に向ける
            transform.look_at(target_point, Vec3::Y);
            println!("🟢 Enemy patrolling to point: {:?}", target_point);
        }
    }
}

/// カメラ追従システム
fn camera_follow_player(
    player_query: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    mut camera_query: Query<(&mut Transform, &CameraController), (With<Camera3d>, Without<Player>)>,
) {
    let Ok(player_transform) = player_query.single() else {
        return; // プレイヤーが存在しない場合は何もしない
    };
    let Ok((mut camera_transform, camera_controller)) = camera_query.single_mut() else {
        return; // カメラが存在しない場合は何もしない
    };

    // カメラの目標位置を計算
    // プレイヤーの後ろに距離を取り、上に高さを加える
    let horizontal_offset = Vec3::new(0.0, 0.0, camera_controller.distance);
    let vertical_offset = Vec3::new(0.0, camera_controller.height, 0.0);

    // カメラの目標位置
    let desired_position = player_transform.translation + horizontal_offset + vertical_offset;

    // カメラの位置を滑らかに更新
    camera_transform.translation = camera_transform.translation.lerp(desired_position, 0.1);

    // カメラの向きをプレイヤーに向ける（プレイヤーの中心を見る）
    camera_transform.look_at(player_transform.translation, Vec3::Y);
}

/// カメラズームシステム
fn camera_zoom(
    keys: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut CameraController, With<Camera3d>>,
    time: Res<Time>,
) {
    let Ok(mut camera_controller) = camera_query.single_mut() else {
        return; // カメラコントローラーが存在しない場合は何もしない
    };

    let zoom_delta = camera_controller.zoom_speed * time.delta_secs();

    // ズームイン(Q)
    if keys.pressed(KeyCode::KeyQ) {
        camera_controller.distance = (camera_controller.distance - zoom_delta).clamp(
            camera_controller.min_distance,
            camera_controller.max_distance,
        );
        println!("Zooming in: {}", camera_controller.distance);
    }

    // ズームアウト(E)
    if keys.pressed(KeyCode::KeyE) {
        camera_controller.distance = (camera_controller.distance + zoom_delta).clamp(
            camera_controller.min_distance,
            camera_controller.max_distance,
        );
        println!("Zooming out: {}", camera_controller.distance);
    }
}

/// ゲームオーバー表示システム
fn spawn_game_over_ui(commands: &mut Commands) {
    // ゲームオーバーのUIを表示するためのコードをここに追加
    // 例えば、テキストやボタンを表示するなど
    commands.spawn((
        Text::new("Game Over! Press R to Restart"),
        TextFont {
            font_size: 50.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 0.0, 0.0)), // 赤色のテキスト
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(30.0),
            top: Val::Percent(40.0),
            ..default()
        },
        GameOverUI, // GameOverUIコンポーネントを追加
    ));
    println!("Game Over! Press R to Restart");
}

/// ゲーム再スタートシステム
fn restart_game(
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut game_state: ResMut<GameState>,
    game_over_query: Query<Entity, With<GameOverUI>>,
    mut player_query: Query<&mut Transform, (With<Player>, Without<Enemy>)>,
    mut enemy_query: Query<(&mut Transform, &mut Enemy), Without<Player>>,
) {
    // ゲームオーバー状態でない場合は何もしない
    if *game_state != GameState::GameOver {
        return;
    }
    // Rキーが押された場合の処理
    if keys.just_pressed(KeyCode::KeyR) {
        for entity in game_over_query.iter() {
            commands.entity(entity).despawn(); // ゲームオーバーUIを削除
        }

        // ゲーム状態をPlayingに戻す
        if let Ok(mut player_transform) = player_query.single_mut() {
            // プレイヤーの位置を初期位置にリセット
            player_transform.translation = Vec3::new(0.0, 1.0, 8.0);
        }

        // 敵キャラクターの位置を初期位置にリセット
        for (mut enemy_transform, mut enemy) in enemy_query.iter_mut() {
            enemy_transform.translation = enemy.initial_position; // 敵の初期位置に戻す
            enemy.current_patrol_index = 0; // パトロールポイントのインデックスをリセット

            // 敵の向きを初期位置に向ける
            if !enemy.patrol_points.is_empty() {
                enemy_transform.rotation = enemy.initial_rotation; // 敵の初期向きに戻す
            } else {
                // パトロールポイントがない場合は外側を向ける
                enemy_transform.rotation = Quat::from_rotation_y(std::f32::consts::PI);
            }
        }
        *game_state = GameState::Playing; // ゲーム状態をPlayingに戻す
        println!("Game restarted!");
    }
}
