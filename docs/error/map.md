# `map()` 使用時のエラー原因

以下は、`map()` を使ったイテレータ変換で発生した2つの主な問題点です。

## 1. `map()` は「値を返す」ためのメソッド

```rust
samples.iter()
    .skip(skip)
    .take(despawn_amount)
    .map(|entity| {
        commands
            .entity(entity)
            .insert(DespawningPoint { progress: 0.0 })
            .remove::<SpawningPoint>()
            .remove::<SamplePoint>();
    })
    .count();
```

* `.map(...)` は「各要素をクロージャで処理し、その返り値を新しいイテレータとして返す」メソッドです。
* のみ副作用を行い、返り値として `()` を返しているので、意図と合っていません。
* 最後に `.count()` すると、空の返り値（`()`）を数えるためだけの処理になり、処理のタイミングや借用制御が不明瞭になります。

## 2. `commands.entity(...)` の借用がクロージャ外に「逃げる」

* `commands.entity(...)` の戻り値である `EntityCommands` は、内部で `&mut Commands` を借用し続けます。
* これを `.map()` の返り値としてイテレータに閉じ込めてしまうと、その借用がイテレータ（`.count()` が終わるまで）生き続けるため、
  Rust の借用規則が「借用が外へ逃げている」と判断し、コンパイルエラー（E0515）が発生します。

---

### 解決策：単純な `for` ループへ置き換え

副作用だけを行いたい場合は、イテレータ変換ではなくシンプルな `for` ループで記述します。

```rust
let mut removed = 0;
for entity in samples.iter().skip(skip).take(despawn_amount) {
    commands.entity(entity)
        .insert(DespawningPoint { progress: 0.0 })
        .remove::<SpawningPoint>()
        .remove::<SamplePoint>();
    removed += 1;
}
counter.0 -= removed;
```

* 各イテレーションごとに借用が完結するため、借用エラーが発生しません。
* 自前でカウントを行うことで `.count()` も不要になります。

---

上記をコピーして、該当箇所に貼り付けてご利用ください。
