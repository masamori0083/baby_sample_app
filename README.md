# 📘 Bevy ECS サンプルアプリケーションの概要

## 📌 全体の処理フロー

```
【初期化フェーズ (Startup)】
└─ setup()
    ├─ 地面・図形・カメラ・ライト・UI を初期配置
    └─ リソースを初期化（メッシュ、マテリアル、乱数生成器など）

【毎フレーム処理フェーズ (Update)】
├─ 入力処理
│  ├─ handle_keypress()：キーボード入力
│  └─ handle_mouse()：マウス入力
│
├─ ポイント管理
│  ├─ spawn_points()：ポイント生成
│  │   └─ animate_spawning()：生成アニメーション
│  └─ despawn_points()：ポイント削除
│      └─ animate_despawning()：削除アニメーション
│
└─ 描画更新
   ├─ update_camera()：カメラ更新
   └─ update_lights()：ライト強度調整
```

## 📌 構造体と役割

### 🔹 リソース (`Resource`)

| リソース名         | 役割          |
| ------------- | ----------- |
| RandomSource  | 乱数生成器       |
| PointMesh     | ポイント表示用メッシュ |
| PointMaterial | ポイント用マテリアル  |
| SpawnQueue    | ポイント生成キュー   |
| PointCounter  | ポイント数管理     |
| SamplingMode  | サンプリングモード   |
| SpawningMode  | ポイント生成モード   |
| SampledShapes | サンプリング対象図形  |
| MousePressed  | マウス押下状態     |

### 🔹 コンポーネント (`Component`)

| コンポーネント名        | 役割               |
| --------------- | ---------------- |
| SamplePoint     | ポイントマーカー         |
| SpawningPoint   | ポイント生成時アニメーション管理 |
| DespawningPoint | ポイント消滅時アニメーション管理 |
| FireflyLights   | ライト強度調整マーカー      |
| CameraRig       | カメラ操作            |
| Transform       | 位置・回転・スケール       |
| PointLight      | ライト情報            |

## 📌 Bevy特有の用語・概念

```rust
// Bevy ECS 特有の概念
// -----------------------------------
// App: アプリケーション本体
// Commands: ECSコマンド
// Assets<T>: 資産管理
// Handle<T>: 資産参照ハンドル
// Resource: 共有データ
// Res<T>, ResMut<T>: リソース参照
// Component: エンティティ属性
// System: エンティティ処理関数
// Plugin: 機能拡張
// DefaultPlugins: 標準プラグイン
// Startup, Update: 実行タイミング
// Query: エンティティ取得
// Bundle: コンポーネント一括追加
// Transform: 位置・回転・スケール
// Mesh: 形状データ
// StandardMaterial: マテリアル情報
// Node/UI: UI表示コンポーネント
// Camera3d: カメラ
// Tonemapping: HDR色域変換
// Bloom: ブルーム効果
// AlphaMode: 透明度設定
// ClearColor: 背景色設定
// Event: イベント通信
```

## 📌 Rust構文の補足

```rust
// 補足構文
// -----------------------------------
// Res<T>: 読み取り専用リソース
// ResMut<T>: 変更可能リソース
// Query<&T, With<U>>: Uを持つエンティティからT取得
// Single<&T>: 単一存在保証コンポーネント取得
// commands.spawn(): エンティティ生成
// commands.despawn(): エンティティ削除
// commands.insert_resource(): リソース追加
// transform.scale = Vec3::splat(x): スケール一括設定
// if let, match: パターンマッチ
// lerp(target, fraction): 線形補間
```

## 📌 Cargo.toml 依存クレート

```toml
[dependencies]
bevy = { version = "0.13", features = ["dynamic_linking"] }
rand = "0.8"
rand_chacha = "0.3"
```

---

以上を参考にBevy ECSを活用したアプリ開発を進めてください。
