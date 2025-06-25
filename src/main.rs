use std::f32::consts::PI;

use bevy::{
    core_pipeline::bloom::Bloom, // ブルーム(光の拡散)とトーンマッピング(HDRからディスプレイ表示に変換)
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseButtonInput}, // 入力イベント
    math::prelude::*,
    prelude::*, // Bevyの基本的なプリリュード(基本的機能とか要素とか)
};
use rand::{Rng, SeedableRng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;

fn main() {
    App::new() // 新しいBevyアプリケーションを作成(初期化)
        .add_plugins(DefaultPlugins) // Bevyのデフォルトプラグインを追加
        .insert_resource(SampledShapes::new()) // SampledShapesリソース(Resource)を追加
        .add_systems(Startup, setup) // 起動時にsetupシステムを実行(System)
        .add_systems(
            Update,
            (
                handle_mouse,       // マウス入力を処理するシステム
                handle_keypress,    // キーボード入力を処理するシステム
                spawn_points,       // ポイントを生成するシステム(エンティティをランダムに生成)
                despawn_points,     // ポイントを削除するシステム
                animate_spawning, // ポイントの生成アニメーションを処理するシステム(出現アニメーション)
                animate_despawning, // ポイントの削除アニメーションを処理するシステム(消失アニメーション)
                update_camera,      // カメラの更新を処理するシステム(カメラの位置や角度の変更)
                update_lights, // ライトの更新を処理するシステム(シーン内の光源の位置や強度の変更)
            ),
        )
        .run();
}

/////////// 定数定義 ///////////

/// カメラとターゲット(注視点)との最大距離(m)
/// すべてのオブジェクトがこの距離内に収まるようにする
const MAX_CAMERA_DISTANCE: f32 = 12.0;

/// カメラとターゲットの最小距離(m)
/// オブジェクトにカメラが被らないようにする
const MIN_CAMERA_DISTANCE: f32 = 1.0;

/// オブジェクト間の距離(間隔)
const DISTANCE_BETWEEN_SHAPES: Vec3 = Vec3::new(2.0, 0.0, 0.0);

/// 存在できるポイント（点）の最大数
/// 動作が重くならないように調整する必要がある
const MAX_POINTS: usize = 3000; // wasm環境で動作を検証し、wasm専用の最大値を設定する必要あり

/// 1フレームあたりに生成されるポイント数
const POINTS_PER_FRAME: usize = 3;

/// 内部に表示するポイントの色
const INSIDE_POINT_COLOR: LinearRgba = LinearRgba::rgb(0.855, 1.1, 0.01);

/// 境界（表面）に表示するポイントの色
const BOUNDARY_POINT_COLOR: LinearRgba = LinearRgba::rgb(0.08, 0.2, 0.90);

/// ポイントの生成・削除アニメーションの所要時間(秒)
const ANIMATION_TIME: f32 = 1.0;

/// 空と環境光に使用される色
const SKY_COLOR: Color = Color::srgb(0.02, 0.06, 0.15);

/// 小サイズの3Dオブジェクトの寸法
const SMALL_3D: f32 = 0.5;

/// 大サイズの3Dオブジェクトの寸法
const BIG_3D: f32 = 1.0;

// 図形の設定

/// 直方体
use once_cell::sync::Lazy;
static CUBOID: Lazy<Cuboid> = Lazy::new(|| Cuboid::new(SMALL_3D, BIG_3D, SMALL_3D));

/// 球体
static SPHERE: Lazy<Sphere> = Lazy::new(|| Sphere {
    radius: 1.5 * SMALL_3D,
});

/// 3Dの三角形
static TRIANGLE_3D: Lazy<Triangle3d> = Lazy::new(|| Triangle3d {
    vertices: [
        // 頂点座標
        Vec3::new(BIG_3D, -BIG_3D * 0.5, 0.0),  // 頂点1
        Vec3::new(0.0, BIG_3D, 0.0),            // 頂点2（頂上）
        Vec3::new(-BIG_3D, -BIG_3D * 0.5, 0.0), // 頂点3
    ],
});

/// カプセル型(円柱の両端に半球がついた形)
static CAPSULE_3D: Lazy<Capsule3d> = Lazy::new(|| Capsule3d {
    radius: SMALL_3D,
    half_length: BIG_3D,
});

/// 円柱
static CYLINDER: Lazy<Cylinder> = Lazy::new(|| Cylinder {
    radius: SMALL_3D,
    half_height: SMALL_3D,
});

// 四面体（ピラミッド型）
static TETRAHEDRON: Lazy<Tetrahedron> = Lazy::new(|| Tetrahedron {
    vertices: [
        // 頂点座標
        Vec3::new(-BIG_3D, -BIG_3D * 0.67, BIG_3D * 0.5), // 頂点1
        Vec3::new(BIG_3D, -BIG_3D * 0.67, BIG_3D * 0.5),  // 頂点2
        Vec3::new(0.0, -BIG_3D * 0.67, -BIG_3D * 1.17),   // 頂点3（背面）
        Vec3::new(0.0, BIG_3D, 0.0),                      // 頂点4（頂上）
    ],
});

// コンポーネントとリソース定義→リソースはアプリケーション全体で共有されるデータ
/// ランダムにポイントを生成するときのモードを示すリソース
/// 内部をサンプリングするか、境界をサンプリングするかを決める

#[derive(Resource)]
enum SamplingMode {
    Interior, // 内部をサンプリング
    Boundary, // 境界をサンプリング
}

/// ポイントが自動的に生成されるかどうかを指定するリソース
#[derive(Resource)]
enum SpawningMode {
    Manual,    // 手動（自動生成しない）
    Automatic, // 自動（継続的に自動生成）
}

/// 生成するポイントの数を管理するリソース
#[derive(Resource)]
struct SpawnQueue(usize);

/// 現在シーン内に存在するポイントの数を追跡するリソース
#[derive(Resource)]
struct PointCounter(usize);

/// サンプリング(ランダムポイントを生成)される図形と、それぞれ位置(オフセット)を保持するリソース
/// 図形のリストを管理する
#[derive(Resource)]
struct SampledShapes(Vec<(Shape, Vec3)>); // Vec<(図形, 位置情報)>

impl SampledShapes {
    /// SampledShapesを新しく作成し、すべての図形を横並びにする
    fn new() -> Self {
        // サンプリング対象となるすべての図形を取得する
        let shapes = Shape::list_all_shapes();

        // 図形の数を取得
        let n_shapes = shapes.len();

        // 各図形を、中央を基準にして左右均等な間隔で並べる
        // 中央からの位置を計算(x方向のみ)
        let translations =
            (0..n_shapes).map(|i| (i as f32 - n_shapes as f32 / 2.0) * DISTANCE_BETWEEN_SHAPES);

        // 図形とそれぞれの位置情報をセットで保存して返す
        SampledShapes(shapes.into_iter().zip(translations).collect())
    }
}

/// サンプリング（ランダムに点を配置）可能な図形を示す列挙型
#[derive(Clone, Copy)]
enum Shape {
    Cuboid,      // 直方体
    Sphere,      // 球体
    Capsule,     // カプセル型
    Cylinder,    // 円柱
    Tetrahedron, // 四面体
    Triangle,    // 三角形
}

/// Meshを生成するためのビルダー構造体（どのShapeかを保持）
struct ShapeMeshBuilder {
    shape: Shape,
}

impl Shape {
    /// 実装済みのすべてのShapeをVecで返す
    fn list_all_shapes() -> Vec<Shape> {
        vec![
            Shape::Cuboid,
            Shape::Sphere,
            Shape::Capsule,
            Shape::Cylinder,
            Shape::Tetrahedron,
            Shape::Triangle,
        ]
    }
}

/// ランダムサンプリングの処理を定義するトレイト（ShapeSample）をShapeに実装
impl ShapeSample for Shape {
    type Output = Vec3;

    /// 図形の「内部」をランダムにサンプリングして1つの点を返す
    fn sample_interior<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec3 {
        match self {
            Shape::Cuboid => CUBOID.sample_interior(rng),
            Shape::Sphere => SPHERE.sample_interior(rng),
            Shape::Capsule => CAPSULE_3D.sample_interior(rng),
            Shape::Cylinder => CYLINDER.sample_interior(rng),
            Shape::Tetrahedron => TETRAHEDRON.sample_interior(rng),
            Shape::Triangle => TRIANGLE_3D.sample_interior(rng),
        }
    }

    /// 図形の「境界（表面）」をランダムにサンプリングして1つの点を返す
    fn sample_boundary<R: Rng + ?Sized>(&self, rng: &mut R) -> Self::Output {
        match self {
            Shape::Cuboid => CUBOID.sample_boundary(rng),
            Shape::Sphere => SPHERE.sample_boundary(rng),
            Shape::Capsule => CAPSULE_3D.sample_boundary(rng),
            Shape::Cylinder => CYLINDER.sample_boundary(rng),
            Shape::Tetrahedron => TETRAHEDRON.sample_boundary(rng),
            Shape::Triangle => TRIANGLE_3D.sample_boundary(rng),
        }
    }
}

/// Mesh化（3D描画可能な形式への変換）を行うトレイト（Meshable）をShapeに実装
impl Meshable for Shape {
    type Output = ShapeMeshBuilder;

    /// このShapeからMeshビルダーを生成する
    fn mesh(&self) -> Self::Output {
        ShapeMeshBuilder { shape: *self }
    }
}

/// Mesh生成処理をShapeMeshBuilderに実装する
impl MeshBuilder for ShapeMeshBuilder {
    /// 実際にMesh（描画用オブジェクト）を構築する関数
    fn build(&self) -> Mesh {
        match self.shape {
            Shape::Cuboid => CUBOID.mesh().into(),
            Shape::Sphere => SPHERE.mesh().into(),
            Shape::Capsule => CAPSULE_3D.mesh().into(),
            Shape::Cylinder => CYLINDER.mesh().into(),
            Shape::Tetrahedron => TETRAHEDRON.mesh().into(),
            Shape::Triangle => TRIANGLE_3D.mesh().into(),
        }
    }
}

/// このサンプルで使用する乱数生成器を保持するリソース
#[derive(Resource)]
struct RandomSource(ChaCha8Rng);

/// managementいを球体として表示するためのMeshハンドルを保持するリソース
#[derive(Resource)]
struct PointMesh(Handle<Mesh>);

/// ポイント表示に使用するマテリアル(材料)のハンドルを保持するリソース
#[derive(Resource)]
struct PointMaterial {
    interior: Handle<StandardMaterial>,
    boundary: Handle<StandardMaterial>,
}

/// サンプリングされたポイントを示すマーカーコンポーネント
/// マーカーコンポーネントは、特定の機能や役割を持つエンティティを示すために使用される
/// これらがついているエンティティだけに特定の処理を適用することができる
#[derive(Component)]
struct SamplePoint;

/// ポイントが生成される時のアニメーションを管理するコンポーネント
/// マイフレームこの値を更新する
#[derive(Component)]
struct SpawningPoint {
    progress: f32, // アニメーションの進行度（0.0から1.0）
}

/// ポイントが削除される時のアニメーションを管理するコンポーネント
#[derive(Component)]
struct DespawningPoint {
    progress: f32, // アニメーションの進行度（0.0から1.0）
}

/// ポイントライト(光源)の強度を変更するためのマーカーコンポーネント
#[derive(Component)]
struct FireflyLights;

/// マウスが押されているかどうかを示すリソース(カメラ操作用)
#[derive(Resource)]
struct MousePressed(bool);

/// カメラの動きを管理するためのコンポーネント
#[derive(Component)]
struct CameraRig {
    /// カメラの水平方向（左右）の回転角度（ラジアン）
    /// 正の値が増えると右方向から見る形になる
    pub yaw: f32,

    /// カメラの垂直方向（上下）の回転角度（ラジアン、-π/2〜π/2）
    /// 正の値は上から下を見下ろす視点
    pub pitch: f32,

    /// 注視点からの距離（ズームレベル）。値が小さいほど拡大される。
    pub distance: f32,

    /// カメラが注視している、または周囲を回転する対象点の位置（3D空間座標）
    /// これがカメラの中心点となる
    pub target: Vec3,
}

/////////// 関数定義 ///////////

/// アプリのセットアップ処理を行う関数
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>, // メッメッシュ(3D)を管理するためのAssetsリソース
    mut materials: ResMut<Assets<StandardMaterial>>, // マテリアル(材料)を管理するためのAssetsリソース
    shapes: Res<SampledShapes>, // サンプリング対象の図形を保持するSampledShapesリソース
) {
    // シード値を指定して乱数生成器を初期化
    let seeded_rng = ChaCha8Rng::seed_from_u64(4); // 乱数生成器のシード値を設定
    commands.insert_resource(RandomSource(seeded_rng)); // 乱数生成器をリソースとして登録

    // 地面となる平面作成して配置する
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20.0, 20.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.3), // 地面の色
            perceptual_roughness: 0.95,             // 光沢感
            metallic: 0.0,                          // 金属感
            ..default()
        })),
        Transform::from_xyz(0.0, -2.5, 0.0), // 地面の位置
    ));

    // 図形表示用の半透明なマテリアルを作成
    let shape_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 0.1, 0.6, 0.3), // 半透明の青紫色
        metallic: 0.0,                                // 金属感なし
        perceptual_roughness: 1.0,                    // 反射率の逆数相当
        alpha_mode: AlphaMode::Blend,                 // 透明モード
        cull_mode: None,                              // 裏面も描画する
        ..default()
    });

    // 各図形を並べて配置する
    for (shape, transform) in shapes.0.iter() {
        // 図形を透明で表示
        commands.spawn((
            Mesh3d(meshes.add(shape.mesh())),
            MeshMaterial3d(shape_material.clone()), // 半透明マテリアルを適用
            Transform::from_translation(*transform), // 位置を設定
        ));

        // ポイントライトを各図形の位置に配置(蛍の光のように)
        commands.spawn((
            PointLight {
                range: 4.0,
                radius: 0.6,
                intensity: 1.0,
                shadows_enabled: false,
                color: Color::LinearRgba(INSIDE_POINT_COLOR),
                ..default()
            },
            Transform::from_translation(*transform), // 各図形の位置に配置
            FireflyLights,                           // ライト調整用のマーカー
        ));
    }

    // 全体を照らすためのグローバルなライトを配置
    commands.spawn((
        PointLight {
            color: SKY_COLOR,       // 環境光の色
            intensity: 2_000.0,     // 光の強さ
            shadows_enabled: false, // 影はなし
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // カメラを設定して初期位置に配置する
    commands.spawn((
        Camera3d::default(), // デフォルトの3Dカメラを使用
        Transform::from_xyz(-2.0, 3.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y), // カメラの初期位置
        Bloom::NATURAL,      // Bloom(光の滲み)エフェクトを有効化
        CameraRig {
            yaw: 0.56,          // 水平方向の角度
            pitch: 0.45,        // 垂直方向の角度
            distance: 8.0,      // ズーム距離
            target: Vec3::ZERO, // 注視点
        },
    ));

    // ポイントを表示する球体のMeshとマテリアルをリソースとして登録
    commands.insert_resource(PointMesh(
        meshes.add(Sphere::new(0.03).mesh().ico(1).unwrap()),
    ));
    commands.insert_resource(PointMaterial {
        interior: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            metallic: 0.0,                      // 金属感なし
            perceptual_roughness: 1.0 - 0.05,   // 反射率の逆数相当
            emissive: 2.5 * INSIDE_POINT_COLOR, // 内部ポイントの発光色
            ..default()
        }),
        boundary: materials.add(StandardMaterial {
            base_color: Color::BLACK,
            metallic: 0.0,                        // 金属感なし
            perceptual_roughness: 1.0 - 0.05,     // 反射率の逆数相当
            emissive: 1.5 * BOUNDARY_POINT_COLOR, // 境界ポイントの発光色
            ..default()
        }),
    });

    // ユーザー向けの操作説明テキストを画面に表示
    commands.spawn((
        Text::new(
            "Controls:\n\
            M: Toggle between sampling boundary and interior.\n\
            A: Toggle automatic spawning & despawning of points.\n\
            R: Restart (erase all samples).\n\
            S: Add one random sample.\n\
            D: Add 100 random samples.\n\
            Rotate camera by holding left mouse and panning.\n\
            Zoom camera by scrolling via mouse or +/-.\n\
            Move camera by L/R arrow keys.\n\
            Tab: Toggle this text",
        ),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    commands.insert_resource(SpawnQueue(0)); // ポイント生成キューを初期化

    commands.insert_resource(PointCounter(0)); // 現在のポイント数を初期化

    commands.insert_resource(SamplingMode::Interior); // 初期は内部サンプリング

    commands.insert_resource(SpawningMode::Automatic); // 初期は自動生成

    commands.insert_resource(MousePressed(false)); // マウスの押下状態を初期化
}

// キーボード入力を処理するシステム
fn handle_keypress(
    mut commands: Commands, // エンティティの生成・削除を行うためのコマンド
    keyboard: Res<ButtonInput<KeyCode>>, // キーボード入力状態
    mut mode: ResMut<SamplingMode>, // サンプリングモード（内部 or 境界）
    mut spawn_mode: ResMut<SpawningMode>, // ポイント生成モード（自動 or 手動）
    samples: Query<Entity, With<SamplePoint>>, // 現在存在する全てのポイント
    shapes: Res<SampledShapes>, // 配置されている図形のデータ
    mut spawn_queue: ResMut<SpawnQueue>, // ポイント生成予約のキュー
    mut counter: ResMut<PointCounter>, // 現在のポイント数を管理
    mut text_menus: Query<&mut Visibility, With<Text>>, // UIテキストの表示・非表示を管理
    mut camera_rig: Query<&mut CameraRig>, // カメラ操作用のコンポーネント
) {
    // Queryから一意のカメラリグを取得
    let mut camera_rig = camera_rig.single_mut().unwrap();

    // 「R」キー：すべてのポイントを削除してリセット
    if keyboard.just_pressed(KeyCode::KeyR) {
        counter.0 = 0; // ポイント数をゼロにリセット
        for entity in &samples {
            commands.entity(entity).despawn(); // 各ポイントを削除
        }
    }

    // 「S」キー：ポイントを1個生成予約
    if keyboard.just_pressed(KeyCode::KeyS) {
        spawn_queue.0 += 1;
    }

    // 「D」キー：ポイントを100個生成予約
    if keyboard.just_pressed(KeyCode::KeyD) {
        spawn_queue.0 += 100;
    }

    // 「M」キー：サンプリングモード（内部 or 境界）を切り替え
    if keyboard.just_pressed(KeyCode::KeyM) {
        *mode = match *mode {
            SamplingMode::Interior => SamplingMode::Boundary,
            SamplingMode::Boundary => SamplingMode::Interior,
        };
    }

    // 「A」キー：ポイント生成モード（自動 or 手動）を切り替え
    if keyboard.just_pressed(KeyCode::KeyA) {
        *spawn_mode = match *spawn_mode {
            SpawningMode::Manual => SpawningMode::Automatic,
            SpawningMode::Automatic => SpawningMode::Manual,
        };
    }

    // 「Tab」キー：画面上のヘルプメニューの表示・非表示を切り替え
    if keyboard.just_pressed(KeyCode::Tab) {
        for mut visibility in text_menus.iter_mut() {
            *visibility = match *visibility {
                Visibility::Hidden => Visibility::Visible,
                _ => Visibility::Hidden,
            };
        }
    }

    // 「-」キー：カメラをズームアウト（距離を遠ざける）
    if keyboard.just_pressed(KeyCode::NumpadSubtract) || keyboard.just_pressed(KeyCode::Minus) {
        camera_rig.distance += MAX_CAMERA_DISTANCE / 15.0;
        camera_rig.distance = camera_rig
            .distance
            .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE); // 距離の範囲制限
    }

    // 「+」キー：カメラをズームイン（距離を近づける）
    if keyboard.just_pressed(KeyCode::NumpadAdd) {
        camera_rig.distance -= MAX_CAMERA_DISTANCE / 15.0;
        camera_rig.distance = camera_rig
            .distance
            .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE); // 距離の範囲制限
    }

    // 「←」および「→」キー：カメラの注視する対象を左右の図形に切り替える
    let left = keyboard.just_pressed(KeyCode::ArrowLeft);
    let right = keyboard.just_pressed(KeyCode::ArrowRight);

    if left || right {
        let mut closest = 0;
        let mut closest_distance = f32::MAX;

        // 現在のターゲットに最も近い図形を検索
        for (i, (_, position)) in shapes.0.iter().enumerate() {
            let distance = camera_rig.target.distance(*position);
            if distance < closest_distance {
                closest = i;
                closest_distance = distance;
            }
        }

        // 左キーなら1つ左の図形へ移動（可能な場合）
        if closest > 0 && left {
            camera_rig.target = shapes.0[closest - 1].1;
        }

        // 右キーなら1つ右の図形へ移動（可能な場合）
        if closest < shapes.0.len() - 1 && right {
            camera_rig.target = shapes.0[closest + 1].1;
        }
    }
}

// マウス操作を処理し、カメラのズームや回転を行うシステム
fn handle_mouse(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>, // マウスの動きを蓄積したデータ
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>, // マウスのスクロールホイールの動きを蓄積したデータ
    mut button_events: EventReader<MouseButtonInput>,      // マウスボタンの入力イベントを取得
    mut camera_query: Query<&mut CameraRig>, // カメラの位置や回転、ズームを管理するコンポーネント
    mut mouse_pressed: ResMut<MousePressed>, // マウスが押されているかどうかの状態
) {
    // Queryから一意のカメラリグを取得
    let mut camera_rig = camera_query.single_mut().unwrap();

    // マウス左ボタンの押下・解放イベントを処理し、状態を更新
    for button_event in button_events.read() {
        if button_event.button != MouseButton::Left {
            continue; // 左ボタン以外は無視
        }
        // 左ボタンの押下状態を更新（true: 押下中, false: 離された状態）
        *mouse_pressed = MousePressed(button_event.state.is_pressed());
    }

    // マウスホイールのスクロールによるズーム操作
    if accumulated_mouse_scroll.delta != Vec2::ZERO {
        // ホイールの動きを使ってズーム距離を調整
        let mouse_scroll = accumulated_mouse_scroll.delta.y;
        camera_rig.distance -= mouse_scroll / 15.0 * MAX_CAMERA_DISTANCE;

        // カメラの距離が指定範囲内に収まるよう調整
        camera_rig.distance = camera_rig
            .distance
            .clamp(MIN_CAMERA_DISTANCE, MAX_CAMERA_DISTANCE);
    }

    // マウス左ボタンが押されていない場合、回転操作を行わない
    if !mouse_pressed.0 {
        return;
    }

    // マウスのドラッグ（動き）によるカメラ回転操作
    if accumulated_mouse_motion.delta != Vec2::ZERO {
        let displacement = accumulated_mouse_motion.delta;

        // 水平方向の動きに応じてカメラを左右に回転（yaw）
        camera_rig.yaw += displacement.x / 90.0;

        // 垂直方向の動きに応じてカメラを上下に回転（pitch）
        camera_rig.pitch += displacement.y / 90.0;

        // 上下の回転が行き過ぎてしまわないように、ピッチ角を制限
        camera_rig.pitch = camera_rig.pitch.clamp(-PI / 2.01, PI / 2.01);
    }
}

// ポイントを新しく生成するシステム
fn spawn_points(
    mut commands: Commands,                  // エンティティ生成用コマンド
    mode: ResMut<SamplingMode>,              // サンプリングモード（内部 or 境界）
    shapes: Res<SampledShapes>,              // サンプリング対象の図形データ
    mut random_source: ResMut<RandomSource>, // 乱数生成器のリソース
    sample_mesh: Res<PointMesh>,             // ポイント表示用のメッシュ
    sample_material: Res<PointMaterial>,     // ポイント表示用のマテリアル
    mut spawn_queue: ResMut<SpawnQueue>,     // ポイント生成キュー
    mut counter: ResMut<PointCounter>,       // 現在のポイント数カウンター
    spawn_mode: ResMut<SpawningMode>,        // ポイント生成のモード（自動 or 手動）
) {
    // 自動生成モードの場合、毎フレーム一定数のポイントを生成
    // マッチする場合のみ内部の処理を実行
    if let SpawningMode::Automatic = *spawn_mode {
        // 生成するポイント数をキューに追加
        spawn_queue.0 += POINTS_PER_FRAME;
    }

    // 生成キューが0なら何もしない
    if spawn_queue.0 == 0 {
        return; // 生成するポイントがない場合は終了
    }

    let rng = &mut random_source.0; // 乱数生成器を取得

    // 無限ループ防止のため、最大1000個までポイントを生成
    for _ in 0..1000 {
        if spawn_queue.0 == 0 {
            break; // 生成キューが空になったらループを抜ける
        }
        spawn_queue.0 -= 1; // キューから1つポイントを取り出す
        counter.0 += 1; // 現在のポイント数を更新

        // 図形と位置をランダムに1つ選ぶ
        let (shape, offset) = shapes.0.choose(rng).expect("図形は最低1つは必要です");

        // 図形の内部または境界からランダムな位置を取得
        // 列挙型のバリエーションをパターンマッチで処理
        let sample: Vec3 = *offset
            + match *mode {
                SamplingMode::Interior => shape.sample_interior(rng), // 内部の点
                SamplingMode::Boundary => shape.sample_boundary(rng), // 境界の点
            };

        // ランダム位置にポイントを生成(初期はスケール0で非表示状態)
        commands.spawn((
            Mesh3d(sample_mesh.0.clone()), // ポイントのメッシュを設定
            MeshMaterial3d(match *mode {
                SamplingMode::Interior => sample_material.interior.clone(), // 内部ポイントのマテリアル
                SamplingMode::Boundary => sample_material.boundary.clone(), // 境界ポイントのマテリアル
            }),
            Transform::from_translation(sample).with_scale(Vec3::ZERO), // 初期スケールは0(非表示)
            SamplePoint,                     // ポイントを示すマーカーコンポーネント
            SpawningPoint { progress: 0.0 }, // 生成アニメーション
        ));
    }
}

// ポイントを削除するシステム
// ポイント数が上限を超えた場合、古いポイントをランダムに削除する
fn despawn_points(
    mut commands: Commands,                    // エンティティ削除用コマンド
    samples: Query<Entity, With<SamplePoint>>, // 現在存在するポイントを取得
    spawn_mode: Res<SpawningMode>,             // ポイント生成モード（自動 or 手動）
    mut counter: ResMut<PointCounter>,         // 現在のポイント数カウンター
    mut random_source: ResMut<RandomSource>,   // 乱数生成器
) {
    // 手動モードでは自動削除しない
    if let SpawningMode::Manual = *spawn_mode {
        return;
    }

    // ポイント数が最大許容量未満の場合は削除しない
    if counter.0 < MAX_POINTS {
        return;
    }

    // 乱数生成器を取得
    let rng = &mut random_source.0;

    // ランダムにポイントを削除するためにスキップ数を決定
    let skip = rng.gen_range(0..counter.0);

    // 削除するポイント数を決定(最大100個まで一度に削除)
    let despawn_amount = (counter.0 - MAX_POINTS).min(100);

    // 実際にポイントを削除(アニメーション付き)
    // イテレータ（Iterator）の機能で、途中の要素をスキップして指定数だけ取得する処理。
    // スキップ数だけ飛ばして、削除するポイント数だけ取得
    let mut removed = 0;
    for entity in samples.iter().skip(skip).take(despawn_amount) {
        commands
            .entity(entity)
            .insert(DespawningPoint { progress: 0.0 })
            .remove::<SpawningPoint>()
            .remove::<SamplePoint>();
        removed += 1;
    }

    // 削除したポイント数をカウンターから引く
    counter.0 -= removed;
}

// ポイント生成アニメーションを処理するシステム
// 生成時のアニメーションで、スケールが0→1へ徐々に大きくなるようにする。
fn animate_spawning(
    mut commands: Commands, // エンティティ操作用コマンド
    time: Res<Time>,        // 時間リソース
    mut samples: Query<(Entity, &mut Transform, &mut SpawningPoint)>, // 生成中ポイントの取得
) {
    let dt = time.delta_secs(); // 前回のフレームからの経過時間を取得

    // 各生成中ポイントに対してアニメーションを更新
    for (entity, mut transform, mut spawning) in samples.iter_mut() {
        spawning.progress += dt / ANIMATION_TIME; // アニメーションの進行度を更新
        transform.scale = Vec3::splat(spawning.progress.min(1.0)); // スケールを徐々に拡大

        // アニメーション完了したら生成中マーカー削除
        if spawning.progress >= 1.0 {
            commands.entity(entity).remove::<SpawningPoint>(); // 生成中マーカーを削除
        }
    }
}

// ポイントの消滅アニメーションを処理するシステム
// 消滅時のアニメーションで、スケールが1→0へ徐々に小さくなるようにする。
fn animate_despawning(
    mut commands: Commands, // エンティティ操作用コマンド
    time: Res<Time>,        // 時間リソース
    mut samples: Query<(Entity, &mut Transform, &mut DespawningPoint)>, // 削除中ポイントの取得
) {
    // 前回のフレームからの経過時間を取得
    let dt = time.delta_secs(); // フレーム間の時間差を取得

    // 各消滅中ポイントに対してアニメーションを更新
    for (entity, mut transform, mut despawning) in samples.iter_mut() {
        despawning.progress += dt / ANIMATION_TIME; // アニメーションの進行度を更新

        // 急なサイズ変化を避けるため、進捗を調整
        despawning.progress = f32::max(despawning.progress, 1.0 - transform.scale.x); // スケールが0になるまで進行度を調整

        // スケールを徐々に縮小
        transform.scale = Vec3::splat((1.0 - despawning.progress).max(0.0));

        // アニメーションが完了したらエンティティを削除
        if despawning.progress >= 1.0 {
            commands.entity(entity).despawn(); // エンティティを削除
        }
    }
}

// カメラの位置や角度を更新するシステム
fn update_camera(mut camera: Query<(&mut Transform, &CameraRig), Changed<CameraRig>>) {
    // カメラ設定(CameraRig)が変更された場合にのみ更新
    for (mut transform, rig) in camera.iter_mut() {
        // 注視対象から見たカメラの方向を計算
        // Quat::from_rotation_x/y():
        // 特定軸の回転数を表す四元数を生成
        let looking_direction =
            Quat::from_rotation_y(-rig.yaw) * Quat::from_rotation_x(rig.pitch) * Vec3::Z; // Y軸とX軸の回転を適用

        // カメラの位置をターゲットから指定位置離れた位置に設定
        transform.translation = rig.target - looking_direction * rig.distance;

        // カメラがターゲットを見るように設定
        transform.look_at(rig.target, Vec3::Y);
    }
}

// ライトの明るさを現在のポイント数に応じて調整するシステム
fn update_lights(
    mut lights: Query<&mut PointLight, With<FireflyLights>>, // FireflyLightsを持つライトを取得
    counter: Res<PointCounter>,                              // ポイント数管理リソース
) {
    // ポイント数に応じてライトの強度を調整(最大2倍まで)
    let saturation = (counter.0 as f32 / MAX_POINTS as f32).min(2.0);
    let intensity = 4_000.0 * saturation; // 強度を計算

    // 各ライトの明るさをなめらかに調整
    for mut light in lights.iter_mut() {
        // 現在の明るさから徐々に目標の明るさに近づける
        // lerpは線形補間を行う関数
        light.intensity = light.intensity.lerp(intensity, 0.04);
    }
}
