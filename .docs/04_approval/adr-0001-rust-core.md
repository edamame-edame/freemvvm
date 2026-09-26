# ADR-0001: Rust Core と C ABI を採用する

- 状態: Accepted
- 決定日: 2026-09-25
- 対象: C++ / C# / Rust / Python 3 対応 MVVM UI ライブラリ

## 決定

UI ライブラリの共通ランタイムを Rust で実装する。各言語へ公開する境界は versioned C ABI とし、各言語には薄い wrapper を提供する。

```text
Rust UI Core
  ├─ Object / Property / Binding / Command
  ├─ Collection / Selection
  ├─ Visual Tree / Layout
  ├─ Input / Focus / IME
  └─ Error / Lifetime / UI thread

Versioned C ABI
  ├─ C++ wrapper
  ├─ C# P/Invoke wrapper
  ├─ Rust safe API
  └─ Python ctypes or cffi wrapper
```

## 背景

UI Core 全体を後から別言語へ置き換える計画は採用しない。Property の更新規則、layout、rendering、input、accessibility、thread、lifetime は相互依存するため、後置換は実質的な再実装になる。C ABI は wrapper とアプリケーション側の境界を安定させるが、Core の置換作業を小さくする仕組みではない。

## Rustを選ぶ理由

- object lifetime と所有権を Core の型で表現できる
- Property / Binding / Collection の共有状態を安全に扱いやすい
- UI thread と worker 間のデータ競合を減らせる
- C++、C#、Python へは共通の C ABI で公開できる
- C++ ABI や STL 型を公開せず、言語間の契約を明確にできる

## ABI 方針

公開境界には次の要素だけを置く。

- 固定幅整数、`float`、`double`
- UTF-8 の `(pointer, length)` 形式
- opaque handle
- 明示的な `retain/release`
- `ui_status` と error object
- `callback + userdata + destroy`
- ABI version と feature query

次の要素は ABI を越えない。

- Rust 型、trait、generic、借用参照
- C++ class、STL、exception、RTTI
- Python object、C# object、delegate の直接保持
- Rust panic、C++ exception、Python exception

各 target triple に対して native artifact をビルドする。同じ target の C ABI artifact は C++、C#、Rust、Python で共有する。

## 最初に固定する契約

1. UI thread と dispatch 規則
2. handle の所有権と破棄順序
3. 文字列の encoding、借用期間、コピー規則
4. callback の保持、解除、再入可能性
5. Property の OneWay / TwoWay / OneTime と更新順序
6. error、panic、キャンセルの表現
7. ABI versioning と compatibility policy

## 最初の実装範囲

最初の縦断シナリオを次の構成に限定する。

```text
Window
  └─ StackPanel
       ├─ TextBox  ← TwoWay  → ViewModel.Name
       ├─ Text    ← OneWay   ← ViewModel.Message
       └─ Button  ─ Command  → ViewModel.Save
```

Rust Core、C ABI、C++ / C# / Rust / Python wrapper の全てで同じシナリオを動かす。次に Text、Button、TextBox、StackPanel、Window、focus、keyboard、最小の accessibility name を追加する。

## ビルドと配布

初期の CI target は対象 OS の決定後に確定する。候補は以下。

- Windows x64 / ARM64
- Linux x64 / ARM64
- macOS x64 / ARM64

配布物は native library と言語 wrapper を分ける。Python は ctypes / cffi を優先し、C extension が必要になった場合は Python ABI の扱いを別途決める。C# は RID ごとの native asset をパッケージに含める。

## 置換可能性の範囲

Core 全体の置換は計画しない。renderer、platform backend、text engine、画像 codec のように明確な契約を持つ subsystem は差し替え可能にする。

## 組み込み Linux

組み込み Linux を正式な対象に含める。対象はまず ARMv7 / AArch64 Linux とし、bare metal や MCU は別の platform profile として扱う。Linux が提供する `std`、thread、filesystem、socket を利用できる target では、Core 全体を `no_std` 前提にしない。

C++ は vendor SDK、graphics stack、既存の組み込み Linux 資産との接続で有利である。Rust は target triple と cross linker を設定して同じ target に出力でき、Yocto には Rust library を C / C++ から呼び出す `cargo_c` class がある。したがって、組み込み Linux を理由に C++ Coreへ変更しない。Rust Core のまま、C++ / C API が必要な platform backend を wrapper 層に閉じ込める。

描画 backend は Core から分離し、初期候補を次の順で検証する。

1. DRM/KMS + EGL/OpenGL ES
2. Wayland
3. X11
4. framebuffer または software renderer

Qt と同様に、組み込み Linux の cross build は toolchain と sysroot を必要とする。Rust でも Yocto SDK の linker、sysroot、C library と target triple を CI で固定する。実機での GPU、入力、IME、display rotation、power management は target board ごとに platform adapter で吸収する。

バイナリサイズ、起動時間、描画 FPS、メモリ使用量は C++ と Rust の一般論で決めない。代表的な ARM board を一つ選び、同じ機能を実装した benchmark を Core 初期段階で測定する。
