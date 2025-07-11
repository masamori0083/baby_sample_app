use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
				.add_systems(Startup, setup)
				.add_systems(Update, draw_cursor)
        .run();
}

fn draw_cursor(
	camera_query: Single<(&Camera, &GlobalTransform)>,
	ground: Single<&GlobalTransform, With<Ground>>,
	windows: Query<&Window>, // window情報
	mut gizmos: Gizmos,
) {
	// カメラの情報を取得
	let Ok(windows) = windows.single() else {
		return;
	};

	// カメラの情報を取得
	// 値にアクセスするためにデリファレンスを使用
	let (camera, camera_transform) = *camera_query;

	// カーソルがウィンドウにない場合は何もしない
	// Someはバリアントで、値が存在する場合にのみ処理を続ける
	let Some(cursor_position) = windows.cursor_position() else {
		return;
	};

	// 矢印のポイントを基準にカーソル位置をワールド座標に変換
	// 光線を生成
	let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
		return;
	};

	// 光線と地面が交差する距離を計算する
	let Some(distance) = ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up())) else {
		return;
	};

	let point = ray.get_point(distance);

	// Gizmosを使用してカーソル位置に円を描画
	// 求めた交点座標のわずか上に円を描く
	gizmos.circle(
		Isometry3d::new(point + ground.up() * 0.01,
		// デフォルトのZ軸方向に円を描くところを、地面の法線方向に円が向くよう回転
		Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
	),
	0.2,
	Color::WHITE,
	);
}

# [derive(Component)]
struct Ground;

fn setup(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: RetMut<Assets<StandardMaterial>>,
) {
	commands.spawn((
		// デフォルトの平面メッシュを使用して地面を作成
		// サイズを20x20に設定
		Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),

		// 地面のマテリアルを設定
		// 色を設定するためにStandardMaterialを使用
		StandardMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
		// Groundコンポーネントを追加して識別
		Ground,
	));

	// ライトを追加
	commands.spawn((
		// デフォルトの方向ライトを使用
		DirectionalLight::default(),
		// ライトの位置を設定
		// 原点を向くように配置
		Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
	));

	// カメラを追加
	commands.spawn((
		// デフォルトのカメラを使用
		Camera3d::default(),
		// カメラの位置を設定
		Transform::from_xyz(15.0, 5.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
	));
	
}