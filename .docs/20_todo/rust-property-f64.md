# Rust wrapper: F64 Property

- 更新日: 2026-10-24
- 状態: 完了（local gates・Rust 1.85・Windows/Linux CI確認済み）
- 対象ブランチ: `impl-rust/wrappers-runtime-api`
- 親: [Rust UTF-8 Property](rust-property-utf8.md)

## 目的と範囲

承認済みRust wrapper DD 1.2の `Property<T>` に `f64` を追加し、共有C ABIのF64 Propertyを生成・取得・設定・closeできるようにする。値を別精度へ変換せず、Rust `f64` とABI `double` を直接対応させる。DECIMAL、notification、Binding、package配布は対象外。

添付 `property.rs` snapshotは編集しない。実装方式を決める前に、workspace側の [`property.rs`](../../wrappers/rust/freemvvm/src/property.rs) を変更できるか確認する。現在のgeneric `Property<T>` は同ファイル内のsealed `PropertyValue` に依存するため、そこを変更禁止とする場合、同じ公開APIを別moduleから追加することはできない。別型を作る場合はAPI/設計変更として改めて提案する。

## 根拠

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018)、[RQ-03-003](../11_requirements/RQ-03_language-support.md#rq-03-003)、[RQ-04-002](../11_requirements/RQ-04_abi-boundary.md#rq-04-002)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- 承認済み設計: [Rust wrapper DD 1.2](../13_detail_design/wrappers/rust.md)、[共通C ABI DD 1.2](../13_detail_design/runtime-objects/abi.md) F64 Property追加型のABI形
- 試験方針: [unit test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)

## 実装・確認項目

1. 添付snapshotは編集せず、workspace側 `property.rs` にsealed `PropertyValue` の `f64` mappingを追加する。
2. `freemvvm-sys`へF64 create/get/set declarationを追加する。
3. `Property<f64>`でfinite value、positive/negative infinity、NaN、positive/negative zeroを共有C ABI経由で生成・get/setする。
4. Binary64値を暗黙の精度変換・正規化なしに渡し、owner/Runtime anchor/cleanup契約を既存Property実装から維持する。
5. unit-test設計・manifest・独立consumer commandを追加し、Rust focused tests、consumer、full bootstrap、Rust 1.85、Windows/Linux CIで確認する。

## 完了条件

- 承認済みgeneric `Property<T>` APIのまま `f64` が追加される。公開API方式を独自に変更しない。
- F64 ABIを介した値の丸め・切り詰めがなく、特殊値と符号付きゼロを扱える。
- 添付 `property.rs` snapshotは変更しない。workspace側の編集が許可されない場合は設計変更なしに別型で代替しない。
- plan/test design approval、実装unit、Windows/Linux CI evidenceを記録する。

## 履歴

| 日付 | 対象 | 内容 |
| --- | --- | --- |
| 2026-10-24 | 計画承認 | 利用者の「OK」によりF64 Property計画とworkspace側 `property.rs` の編集を承認。添付snapshotと既存UTF-8 consumer snapshotは変更せず、新規F64 consumerを追加する。 |
| 2026-10-24 | 実装・検証完了 | implementation commit `2b5ae0c`。`tools/dev.py check`、`minimum`、F64 consumerは成功。Windows/Linux CI run `40081760421` success。plan/test design commit `1311576`。 |
