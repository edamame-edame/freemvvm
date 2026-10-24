# Rust wrapper: DECIMAL Property

- 更新日: 2026-10-24
- 状態: 完了（local gates・Rust 1.85・Windows/Linux CI確認済み）
- 対象ブランチ: `impl-rust/wrappers-runtime-api`
- 親: [Rust F64 Property](rust-property-f64.md)

## 目的と範囲

承認済みgeneric `Property<T>`へDECIMALを追加し、Rustからcanonical `uint64_t` representationを損失なく共有C ABIへ渡す。Rust側の型名・構築/accessor APIは既存DDで未定義のため、[Rust wrapper DD 1.3改訂案](../13_detail_design/wrappers/rust.md)として`Decimal64`型を提案し、文書全体の承認後に実装する。

添付の`property.rs` snapshotは変更しない。workspace側 `property.rs` の編集は、DD 1.3と本計画の承認後に行う。十進演算、parse/format、Validator、notification、Binding、package配布は対象外。

## 提案するRust API

- `Decimal64`はcanonical bitsをprivateに保持する`Copy` scalar。
- `Decimal64::try_from_parts(negative, scale, coefficient)`で構築し、scale 0～31、coefficient 0～2^58−1のみ受理する。範囲外は`UiError(INVALID_ARGUMENT)`相当を返す。
- `is_negative()`、`scale()`、`coefficient()`で各partを参照する。十進演算・文字列変換・raw bits構築は提供しない。
- 標準`PartialEq`/`Eq`は実装しない。raw bits比較を数値同値と誤認させず、等価setterのstored representation維持はProperty/Core contractとして検証する。
- `Property<Decimal64>`は初期値と取得値をcanonical bitsのままC ABIへ渡す。

## 根拠

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)、[RQ-03-003](../11_requirements/RQ-03_language-support.md#rq-03-003)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- 設計: [Rust wrapper DD 1.3改訂案](../13_detail_design/wrappers/rust.md)、[共通ABIのcanonical DECIMAL](../13_detail_design/events/notifications-binding.md)
- 試験方針: [unit test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)

## 完了条件

- Rust `Decimal64` APIと`Property<Decimal64>`がDD 1.3全文承認後に実装される。
- scale/coefficientの範囲、canonical bits、数値上同値な異representationの保持が試験される。
- 既存のBOOL/I64/UTF-8/F64 Property、`handle.rs`、添付`property.rs` snapshotに影響しない。
- implementation unitとverification recordが同じCI revisionを示す。

## 履歴

| 日付 | 対象 | 内容 |
| --- | --- | --- |
| 2026-10-24 | 計画・DD案 | DECIMAL用Rust公開型が未定義のため`Decimal64` API案と試験範囲を作成。コード変更・commit/push前にDD 1.3と計画を利用者レビューへ出す。 |
| 2026-10-24 | 計画・DD承認 | 利用者の「OK」によりDD 1.3全文、Rust DECIMAL計画、test designを承認。workspace側`property.rs`を編集して実装を進める。 |
| 2026-10-24 | 実装・検証完了 | 実装commit `815a8dd`。`tools/dev.py check`、`minimum`、Decimal64 unit tests、DECIMAL consumer成功。Windows/Linux CI run `40082654144` success。計画・DD commit `2e35b6b`。 |
