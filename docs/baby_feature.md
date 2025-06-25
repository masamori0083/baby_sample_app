# Bevy フレームワークの主要概念と型

## 1. App

Bevyアプリケーション本体を作成・実行するための型。
**使用例:**

```rust
App::new().run();
```

## 2. Commands

エンティティ（オブジェクト）の生成・変更・削除などを行うためのコマンド発行。
**使用例:**

```rust
commands.spawn((Component1, Component2));
```

## 3. Assets<T>

メッシュやテクスチャ、マテリアルなどのアセット）を管理するための型。
**使用例:**

```rust
meshes.add(mesh);
materials.add(material);
```

## 4. Handle<T>

アセット（資産）を参照するためのハンドル型。安全かつ簡単に資産へアクセス。
**使用例:**

```rust
let handle = meshes.add(mesh);
```

## 5. Resource

アプリケーション全体で共有されるデータ（設定や状態）を格納する型。
**使用例:**

```rust
commands.insert_resource(MyResource);
```

## 6. Res<T> / ResMut<T>

リソースへの参照型。Resは読み取り専用、ResMutは変更可能。
**使用例:**

```rust
fn system(res: Res<MyResource>, mut res_mut: ResMut<MyOtherResource>) {}
```

## 7. Component

エンティティに付属するデータ。位置や速度などの属性を保持。
**使用例:**

```rust
#[derive(Component)]
struct Velocity(Vec3);
```

## 8. System

コンポーネントを持つエンティティに対して処理を行う関数。
**使用例:**

```rust
fn move_system(query: Query<&mut Transform>) {}
```

## 9. Plugin

Bevyアプリケーションに特定の機能を追加する仕組み。
**使用例:**

```rust
app.add_plugins(MyPlugin);
```

## 10. DefaultPlugins

Bevyが提供する標準的なプラグイン群（レンダリング、入力処理など）を一括追加。
**使用例:**

```rust
app.add_plugins(DefaultPlugins);
```

## 11. Startup / Update (Schedule)

システムが実行されるタイミング。

* **Startup:** 起動時一回実行
* **Update:** 毎フレーム実行

**使用例:**

```rust
app
  .add_systems(Startup, setup)
  .add_systems(Update, update_fn);
```

## 12. Query

特定のコンポーネントを持つエンティティを取得する仕組み。
**使用例:**

```rust
fn system(query: Query<&Transform, With<Player>>) {}
```

## 13. Bundle

複数のコンポーネントをまとめてエンティティに追加する仕組み。
**使用例:**

```rust
commands.spawn(SpriteBundle { ..default() });
```

## 14. Transform

エンティティの位置、回転、スケールを管理するコンポーネント。
**使用例:**

```rust
Transform::from_xyz(0.0, 1.0, 0.0);
```

## 15. Mesh

3Dオブジェクトの形状（頂点情報）を管理する型。
**使用例:**

```rust
meshes.add(Sphere::default());
```

## 16. StandardMaterial

オブジェクトの色や質感などのマテリアル情報を管理する型。
**使用例:**

```rust
materials.add(StandardMaterial { ..default() });
```

## 17. Node / UI

UI要素を画面に表示するためのコンポーネント群。
**使用例:**

```rust
commands.spawn(NodeBundle { ..default() });
```

## 18. Camera3d

3D空間を描画するためのカメラを管理するコンポーネント。
**使用例:**

```rust
commands.spawn(Camera3dBundle::default());
```

## 19. Tonemapping

HDRレンダリングを表示可能な色域に変換する仕組み。
**使用例:**

```rust
Tonemapping::TonyMcMapface;
```

## 20. Bloom

光がにじむ視覚効果（ブルーム）を設定するための型。
**使用例:**

```rust
commands.spawn(Camera3dBundle { bloom: Bloom::default() });
```

## 21. AlphaMode

マテリアルの透明度を制御するための設定。
**使用例:**

```rust
alpha_mode: AlphaMode::Blend;
```

## 22. ClearColor

背景色（クリアカラー）を設定するリソース。
**使用例:**

```rust
commands.insert_resource(ClearColor(Color::BLACK));
```

## 23. Event

システム間でイベント（通知やメッセージ）を送信・受信する仕組み。
**使用例:**

```rust
app.add_event::<MyEvent>();
```
