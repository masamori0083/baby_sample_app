# 📘 Bevy Sample App - 学習プロジェクト

## 📌 プロジェクト概要

このプロジェクトは、[Bevy公式サンプル](https://bevy.org/examples/)をベースに、Rustの3Dゲームエンジン「Bevy」の学習を目的としたサンプルアプリケーション集です。各サンプルをリファクタリングし、日本語コメントと詳細な解説を追加して、Bevyフレームワークの理解を深めるための学習リソースとして構築しています。

## 📌 プロジェクト構造

```
bevy_sample_app/
├── README.md                    # このファイル
├── .gitignore                   # Git無視設定
├── docs/                        # 学習資料
│   ├── baby_feature.md          # Bevy基本概念解説
│   └── error/
│       └── map.md               # エラー対応マップ
├── primitives/                  # 基本図形サンプル
│   ├── Cargo.toml
│   ├── README.md
│   ├── src/main.rs
│   └── assets/sounds/
└── volumetric_fog/              # ボリューメトリック霧サンプル
    ├── Cargo.toml
    ├── README.md
    ├── src/main.rs
    └── assets/
        ├── environment_maps/
        └── models/
```

## 📌 収録サンプル

### 🔹 1. [Primitives](primitives/) - 基本図形とインタラクション
- **機能**: 3D基本図形の生成・表示・操作
- **学習要素**: ECS、マウス/キーボード入力、音声再生、アニメーション
- **技術**: PointLight、Transform、Query、Commands、Resource

### 🔹 2. [Volumetric Fog](volumetric_fog/) - ボリューメトリック霧効果
- **機能**: 立体的な霧効果とライトの可視化
- **学習要素**: 高度なレンダリング、HDR、Bloom効果、リアルタイム設定変更
- **技術**: VolumetricFog、VolumetricLight、Skybox、Tonemapping

## 📌 学習資料

### 🔹 [Bevy基本概念解説](docs/baby_feature.md)
Bevyフレームワークの主要な型と概念を体系的に整理:

- **ECS基本要素**: App, Commands, Component, System, Resource
- **アセット管理**: Assets<T>, Handle<T>, Mesh, StandardMaterial
- **レンダリング**: Camera3d, Transform, Tonemapping, Bloom
- **入出力**: Query, Bundle, Event, UI/Node
- **実行制御**: Plugin, DefaultPlugins, Startup/Update

### 🔹 [エラー対応マップ](docs/error/map.md)
よくあるエラーとその対処法をまとめた実践的なガイド

## 📌 技術スタック

### 🔹 主要クレート
```toml
[dependencies]
bevy = { version = "0.13", features = ["dynamic_linking"] }
rand = "0.8"              # 乱数生成
rand_chacha = "0.3"       # 高品質乱数
```

### 🔹 Bevyの主要機能
- **ECS (Entity Component System)**: 高効率なデータ管理
- **PBR (Physically Based Rendering)**: 物理ベースレンダリング
- **HDR Pipeline**: 高動的範囲レンダリング
- **Audio System**: 音声再生システム
- **Asset Management**: リソース管理システム
- **Input Handling**: 入力処理システム

## 📌 各サンプルの実行方法

### 🔹 基本図形サンプル
```bash
cd primitives
cargo run
```

**操作方法:**
- マウス: 図形の選択・操作
- キーボード: 各種設定変更
- 音声: インタラクション時の効果音

### 🔹 ボリューメトリック霧サンプル
```bash
cd volumetric_fog
cargo run
```

**操作方法:**
- `W/A/S/D` or `矢印キー`: 方向性ライト制御
- `P`: PointLightの霧効果切り替え
- `L`: SpotLightの霧効果切り替え

## 📌 学習のポイント

### 🔹 ECSアーキテクチャの理解
```rust
// Entity（エンティティ）: オブジェクトの識別子
// Component（コンポーネント）: データの塊
// System（システム）: 処理ロジック

#[derive(Component)]
struct Velocity(Vec3);

fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0;
    }
}
```

### 🔹 リソース管理
```rust
// 全体で共有するデータ
#[derive(Resource)]
struct GameSettings {
    volume: f32,
    difficulty: u32,
}

// システムでの使用
fn audio_system(settings: Res<GameSettings>) {
    // settings.volume を使用
}
```

### 🔹 クエリシステム
```rust
// 特定の条件でエンティティを取得
fn render_system(
    query: Query<(&Transform, &Mesh), With<Visible>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // 処理...
}
```

## 📌 開発環境セットアップ

### 🔹 前提条件
- Rust 1.70以上
- Cargo
- Git

### 🔹 クローンと実行
```bash
git clone [repository-url]
cd bevy_sample_app

# 各サンプルの実行
cd primitives && cargo run
cd ../volumetric_fog && cargo run
```

### 🔹 開発モード
```bash
# 高速コンパイル（デバッグ用）
cargo run

# 最適化ビルド（リリース用）
cargo run --release
```

## 📌 学習の進め方

### 🔹 Step 1: 基本概念の理解
1. [docs/baby_feature.md](docs/baby_feature.md)でBevy基本概念を学習
2. [primitives/](primitives/)で基本的なECSパターンを実践
3. コードを読み、コメントと照らし合わせて理解

### 🔹 Step 2: 実践的な開発
1. [volumetric_fog/](volumetric_fog/)で高度なレンダリング技術を学習
2. 各サンプルを改造・拡張してみる
3. 新しい機能を追加してみる

### 🔹 Step 3: 発展的な学習
1. [Bevy公式サンプル](https://bevy.org/examples/)の他のサンプルを試す
2. 独自のサンプルを作成
3. プラグインの作成に挑戦

## 📌 リファクタリングのポイント

### 🔹 コードの改善点
- **日本語コメント**: 理解しやすい詳細な説明
- **構造の整理**: 論理的な処理フローの整理
- **型安全性**: Rustの型システムを活用した安全性向上
- **パフォーマンス**: Bevyのベストプラクティスに従った最適化

### 🔹 学習効果を高める工夫
- **段階的な説明**: 初心者から上級者まで対応
- **実践的な例**: 実際のゲーム開発で使える技術
- **エラー対応**: よくある問題とその解決方法

## 📌 今後の拡張予定

- [ ] 物理演算サンプル
- [ ] UI/UXサンプル
- [ ] ネットワーク通信サンプル
- [ ] WebAssembly対応
- [ ] プラグイン開発ガイド

## 📌 参考資料

- [Bevy公式ドキュメント](https://bevy-cheatbook.github.io/)
- [Bevy公式サンプル](https://bevy.org/examples/)
- [Rust公式ドキュメント](https://doc.rust-lang.org/)

---

このプロジェクトを通じて、Bevyフレームワークの理解を深め、実践的なゲーム開発スキルを身につけることができます。各サンプルを順番に学習し、コードの改造・拡張を通じて理解を深めていってください。