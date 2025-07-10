# 📘 Bevy Volumetric Fog サンプルアプリケーション

## 📌 アプリケーション概要

このアプリケーションは、Bevyエンジンを使用してボリューメトリック（立体的な）霧効果を実演するサンプルプログラムです。3Dシーン内でライトが霧の中を通る際の光の道筋を可視化し、リアルタイムで設定を変更できます。

## 📌 主な機能

### 🔹 ボリューメトリック効果
- **VolumetricFog**: 3D空間内に立体的な霧を生成
- **VolumetricLight**: ライトが霧の中を通る際の光の道筋を可視化
- **リアルタイム切り替え**: P/Lキーで各ライトの効果をオン/オフ

### 🔹 ライトシステム
- **PointLight**: 赤色の点光源（自動的に左右に移動）
- **SpotLight**: 白色のスポットライト（固定位置）
- **DirectionalLight**: 方向性ライト（WASD/矢印キーで制御）

### 🔹 視覚効果
- **HDRレンダリング**: 高動的範囲での色表現
- **Bloom効果**: 光のにじみ・拡散効果
- **Skybox**: 環境マップによる背景
- **リアルタイムシャドウ**: 全ライトでシャドウ生成

## 📌 操作方法

| キー | 機能 |
|-----|-----|
| **W/S** または **↑/↓** | 方向性ライトの縦方向移動 |
| **A/D** または **←/→** | 方向性ライトの横方向移動 |
| **P** | PointLightのボリューメトリック効果切り替え |
| **L** | SpotLightのボリューメトリック効果切り替え |

## 📌 システム構成

### 🔹 リソース (`Resource`)

```rust
#[derive(Resource)]
struct AppSettings {
    volumetric_fog_enabled: bool,     // 霧効果の有効/無効
    volumetric_light_enabled: bool,   // ライト効果の有効/無効
}
```

### 🔹 コンポーネント (`Component`)

```rust
#[derive(Component)]
struct MoveBackAndForthHorizontally {
    min_x: f32,    // 移動範囲の最小値
    max_x: f32,    // 移動範囲の最大値
    speed: f32,    // 移動速度
}
```

## 📌 システム実行フロー

```
【初期化フェーズ (Startup)】
└─ setup()
    ├─ 3Dモデル読み込み（glTF形式）
    ├─ カメラセットアップ（HDR、Bloom、VolumetricFog）
    ├─ ライト配置（PointLight、SpotLight）
    ├─ 霧効果設定（FogVolume）
    └─ UI表示（操作説明テキスト）

【毎フレーム処理フェーズ (Update)】
├─ tweak_scene()：DirectionalLightの自動設定
├─ move_directional_light()：方向性ライトの手動制御
├─ move_point_light()：PointLightの自動移動
└─ adjust_app_settings()：設定変更とUI更新
```

## 📌 技術的特徴

### 🔹 Bevy特有の機能
- **ECS（Entity Component System）**: 効率的なデータ管理
- **Query システム**: 特定条件のエンティティを高速検索
- **Commands**: エンティティの動的な追加・削除・変更
- **Changed フィルター**: 変更のあったコンポーネントのみ処理

### 🔹 レンダリング技術
- **HDR パイプライン**: 高動的範囲での色表現
- **Tonemapping**: HDRから表示用色域への変換
- **Volumetric Rendering**: 立体的な霧・光の表現
- **PBR（Physically Based Rendering）**: 物理ベースレンダリング

## 📌 ファイル構成

```
volumetric_fog/
├── Cargo.toml          # 依存関係定義
├── src/
│   └── main.rs         # メインプログラム
├── assets/
│   ├── environment_maps/
│   │   └── pisa_specular_rgb9e5_zstd.ktx2  # 環境マップ
│   └── models/
│       └── VolumetricFogExample/
│           └── VolumetricFogExample.glb    # 3Dモデル
└── README.md           # このファイル
```

## 📌 依存クレート

```toml
[dependencies]
bevy = { version = "0.13", features = ["dynamic_linking"] }
```

## 📌 実行方法

```bash
# プロジェクトのルートディレクトリで実行
cargo run

# リリースビルド（最適化有効）
cargo run --release
```

## 📌 学習ポイント

### 🔹 Bevy ECS の活用
- **Resource**: アプリケーション全体の設定管理
- **Component**: エンティティの属性定義
- **System**: 処理ロジックの実装
- **Query**: 効率的なデータアクセス

### 🔹 3Dレンダリング
- **Transform**: 位置・回転・スケール管理
- **Light**: 様々な光源の設定
- **Camera**: 視点・投影の制御
- **Material**: 材質・質感の定義

### 🔹 イベント処理
- **ButtonInput**: キーボード入力の検出
- **Time**: フレーム時間の管理
- **Changed**: 変更検出によるパフォーマンス最適化

---

このサンプルを通じて、Bevyエンジンの高度な3Dレンダリング機能と、効率的なECSアーキテクチャについて学習できます。