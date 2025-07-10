use bevy:: {
	core_pipeline::{bloom::Bloom, tonemapping::Tonemapping, Skybox},
	math::Vec3,
	pbr::{FogVolume, VolumetricFog, VolumetricLight},
	prelude::*,
};

// 光の動きの速度を定義
const DIRECTIONAL_LIGHT_MOVEMENT_SPEED: f32 = 0.02;

/// ユーザーが選んだ設定
#[derive(Resource)]
struct AppSettings {
	// ボリューメトリックスポットライトが有効かどうか
	volumetric_spotlight: bool,
	// ボリューメトリックポイントライトが有効かどうか
	volumetric_pointlight: bool,
}

/// 構造体の初期化
impl Default for AppSettings {
	fn default() -> Self {
		// デフォルトではボリューメトリックスポットライトとポイントライトが有効
		Self {
			volumetric_spotlight: true,
			volumetric_pointlight: true,
		}
	}
}

/// point lightの動きの範囲を定義
#[derive(Component)]
struct MoveBackAndForthHorizontally {
	// 動く範囲の最小値
	min_x: f32,
	// 動く範囲の最大値
	max_x: f32,
	// 移動速度(正なら右、負なら左)
	speed: f32,
}


fn main() {
	App::new()
		.add_plugins(DefaultPlugins) // デフォルトのプラグインを追加
		.insert_resource(ClearColor(Color::Srgba(Srgba {
			red: 0.02,
			green: 0.02,
			blue: 0.02,
			alpha: 1.0,
		})))
		.insert_resource(AmbientLight {
    color: Color::BLACK,
    brightness: 0.0,
		affects_lightmapped_meshes: false,
	}) // 環境光を無効化
		.init_resource::<AppSettings>()
		.add_systems(Startup, setup)
		.add_systems(Update, tweak_scene) // Updateは毎フレーム呼ばれる
		.add_systems(Update, (
			move_point_light,
			move_directional_light,
		))
		.add_systems(Update, adjust_app_settings)
		.run();
}

/// シーンのセットアップ
fn setup(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	app_settings: Res<AppSettings>,
) {
	// glTF形式の3Dモデルを読み込む
	commands.spawn(
		SceneRoot(asset_server.load("models/VolumetricFogExample/VolumetricFogExample.glb#Scene0")),
	);

	// カメラを追加
	commands
	.spawn((
		Camera3d::default(),
		Camera {
			hdr: true, // HDRを有効化
			..default()
		},
		Transform::from_xyz(-1.7, 1.5, 4.5).looking_at(vec3(-1.5, 1.7, 3.5), Vec3::Y), // 注視点を設定
		Tonemapping::TonyMcMapface, // 明暗調整
    Bloom::default(), // 光のにじみ
	))
	.insert(Skybox { // 周囲の環境を示す背景
		image: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"), // 環境マップを設定
    brightness: 1000.0,
    ..default()
  })
	.insert(VolumetricFog { // 立体的な霧効果
		// 環境光は無効化
		ambient_intensity: 0.0, // 環境光の強度
		..default()
	});

	// point lightを追加
	commands.spawn((
		Transform::from_xyz(-0.4, 1.9, 1.0), // 初期位置
		PointLight {
			shadows_enabled: true, // シャドウを有効化
			range: 150.0, // 光の範囲
			color: Color::srgb(1.0, 0.0, 0.0), // 光の色を赤に設定
			intensity: 1000.0, // 光の強度
			..default()
		},
		VolumetricLight, // 光が当たった物体だけ明るくなる効果
		MoveBackAndForthHorizontally { // 左右に自動で動く設定
			min_x: -1.93,
			max_x: -0.4,
			speed: -0.2,
		},
	));

	// spot lightを追加
	commands.spawn((
		Transform::from_xyz(-1.8, 3.9, -2.7).looking_at(Vec3::ZERO, Vec3::Y), // 座標から原点を向く
		SpotLight {
			intensity: 5000.0, //ルーメンス
			color: Color::WHITE, // 光の色を白に設定
			inner_angle: 0.76, // 内側の角度
			outer_angle: 0.94, // 外側の角度
			shadows_enabled: true, // シャドウを有効化
			..default()
		},
		// 光が当たった物体だけ明るくなる効果
		// 光の通り道が見える立体的な表現
		VolumetricLight,
	));

	// FogVolumeを追加(霧の効果)
	commands.spawn((
		FogVolume::default(), // デフォルトの霧設定
		Transform::from_scale(Vec3::splat(35.0)), // 霧のスケールを均一に35倍
	));

	// 表示用のUIテキストを追加
	commands.spawn((
		create_text(&app_settings),
		Node {
			position_type: PositionType::Absolute, // 絶対位置
			top: Val::Px(12.0), // 上から12px
			left: Val::Px(12.0), // 左から12px
			..default()
		},
	));
}

/// UIテキストを作成する関数
fn create_text(app_settings: &AppSettings) -> Text {
    format!(
        "{}\n{}\n{}",
        "Press WASD or the arrow keys to change the direction of the directional light",
        if app_settings.volumetric_pointlight {
            "Press P to turn volumetric point light off"
        } else {
            "Press P to turn volumetric point light on"
        },
        if app_settings.volumetric_spotlight {
            "Press L to turn volumetric spot light off"
        } else {
            "Press L to turn volumetric spot light on"
        }
    )
    .into()
}

/// シーン内で変更があったDirectionLightに対して影の有効化と光源効果を付与
fn tweak_scene(
	mut commands: Commands,
	mut lights: Query<(Entity, &mut DirectionalLight), Changed<DirectionalLight>>, // シーン内で変更されたDirectionalLightを取得
) {
	// 直前のフレームでなんらかの変更があった全てのDirectionalLightに対して...
	for (light, mut directional_light) in lights.iter_mut() {
		directional_light.shadows_enabled = true; // シャドウを有効化
		commands.entity(light).insert(VolumetricLight); // 光の道筋が見える効果を付与
	}
}

/// ユーザーの入力に対して光の動きを調整するシステム
fn move_directional_light(
	input: Res<ButtonInput<KeyCode>>,
	mut directional_lights: Query<&mut Transform, With<DirectionalLight>>,
) {
	let mut delta_theta = Vec2::ZERO; // 光の動きの変化量を初期化

	if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        delta_theta.y += DIRECTIONAL_LIGHT_MOVEMENT_SPEED;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        delta_theta.y -= DIRECTIONAL_LIGHT_MOVEMENT_SPEED;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        delta_theta.x += DIRECTIONAL_LIGHT_MOVEMENT_SPEED;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        delta_theta.x -= DIRECTIONAL_LIGHT_MOVEMENT_SPEED;
    }

    if delta_theta == Vec2::ZERO { // 何も入力されていない場合は即時に終了
        return;
    }

		// オイラー角(XZY)で回転クォータニオンを生成
		let delta_quat = Quat::from_euler(EulerRot::XZY, delta_theta.y, 0.0, delta_theta.x);
		
		// 全てのDirectionalLightに対して...
		for mut transform in directional_lights.iter_mut() {
			// 回転を適用
			transform.rotate(delta_quat);
		}
}


/// シーン内のPointLightを左右に動かすシステム
/// 自動で動かす
fn move_point_light(
	timer: Res<Time>,
	mut objects: Query<(&mut Transform, &mut MoveBackAndForthHorizontally)>,
) {
	for (mut transform, mut move_data) in objects.iter_mut() {
		// 現在の位置を取得(translation)
		let mut translation = transform.translation;
		let mut need_toggle = false;

		// 移動量の計算
		translation.x += move_data.speed * timer.delta_secs();

		// 範囲を超えた場合の処理
		if translation.x > move_data.max_x {
			translation.x = move_data.max_x; // 最大値に設定
			need_toggle = true; // トグルが必要
		} else if translation.x < move_data.min_x {
			translation.x = move_data.min_x; // 最小値に設定
			need_toggle = true; // トグルが必要
		}

		// 折り返し処理
		if need_toggle {
			move_data.speed *= -1.0;
		}

		// 位置情報を更新
		transform.translation = translation;
	}
}

/// ユーザーの入力に応じてアプリケーションの設定を調整するシステム
fn adjust_app_settings(
	mut commands: Commands,
	keyboard_input: Res<ButtonInput<KeyCode>>,
	mut app_settings: ResMut<AppSettings>, // アプリケーションの設定を可変可能な形で取得
	mut point_lights: Query<Entity, With<PointLight>>,
	mut spot_lights: Query<Entity, With<SpotLight>>,
	mut text: Query<&mut Text>,
) {

	// 変更のフラグ
	let mut any_changed = false;

	if keyboard_input.just_pressed(KeyCode::KeyP) {
		// Pキーが押された場合、PointLightのボリューメトリック効果を切り替え(on/off)
		app_settings.volumetric_pointlight = !app_settings.volumetric_pointlight;
		any_changed = true;
	}
	if keyboard_input.just_pressed(KeyCode::KeyL) {
		// Lキーが押された場合、SpotLightのボリューメトリック効果を切り替え(on/off)
		app_settings.volumetric_spotlight = !app_settings.volumetric_spotlight;
		any_changed = true;
	}

	// 変更がない場合終了
	if !any_changed {
		return;
	}
	
	// PointLightのボリューメトリック効果を更新
	for point_light in point_lights.iter_mut() {
		if app_settings.volumetric_pointlight {
			commands.entity(point_light).insert(VolumetricLight); // 効果を追加
		} else {
			commands.entity(point_light).remove::<VolumetricLight>(); // 効果を削除
		}
	}

	// SpotLightのボリューメトリック効果を更新
	for spot_light in spot_lights.iter_mut() {
		if app_settings.volumetric_spotlight {
			commands.entity(spot_light).insert(VolumetricLight); // 効果を追加
		} else {
			commands.entity(spot_light).remove::<VolumetricLight>(); // 効果を削除
		}
	}

	// UIテキストを更新
	for mut text in text.iter_mut() {
		// テキストの内容を更新
		*text = create_text(&app_settings);
	}
}