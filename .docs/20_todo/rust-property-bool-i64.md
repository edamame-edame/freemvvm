# Rust wrapper: BOOL/I64 Property

- 更新日: 2026-10-24
- 状態: 完了（実装・local gates・Rust 1.85・Windows/Linux CI確認済み）
- 対象ブランチ: `impl-rust/wrappers-runtime-api`
- 親: Rust wrapper Runtime基盤

## 目的と範囲

Rust safe wrapperから共有C ABIのBOOL/I64 Propertyを利用できるようにする。最初のProperty unitは生成・typed get/set・closeに限定し、UTF-8/F64/DECIMAL、notification、Binding、package配布経路は後続unitとする。

添付の `native_api.rs` は読み取り専用として扱い、変更しない。Property用raw declarationとsafe call層を別moduleへ置き、既存Runtime/handle owner規則を再利用する。

## 根拠・変更対象

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-03-003](../11_requirements/RQ-03_language-support.md#rq-03-003)、[RQ-03-007](../11_requirements/RQ-03_language-support.md#rq-03-007)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- 承認済み詳細設計: [Rust wrapper DD 1.2](../13_detail_design/wrappers/rust.md)「Propertyの生成・利用・終了API」。1.1全文承認済み範囲を維持し、Property constructorとownership mappingを具体化
- ABI: [共通C ABI](../13_detail_design/runtime-objects/abi.md) Property BOOL/I64 create/get/set、handle release
- 試験方針: [unit test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)

## 実装・確認項目

1. `freemvvm-sys` にBOOL/I64 Property ABI declarationを追加する。添付 `native_api.rs` は変更しない。
2. safe `Property<T>` とsealed `PropertyValue` を追加し、BOOL/I64の生成/get/setを型安全にする。
3. create前にProperty tokenとerror token用cleanup ticketを確保する。UI contextとRuntime identityを確認し、native status/error snapshotを既存`UiError`へ変換する。
4. explicit close、Drop、Runtime anchorは既存`Owned<T>`の契約を維持する。
5. 既存Runtime consumerを拡張し、共有native ABIを通じてBOOL/I64の初期値・更新値・closeを確認する。
6. wrapper test tableと`tools/units.yaml`へ実行対象を記録する。
7. docs links、manifest、format/lint/build/test、Rust 1.85、Windows/Linux CIを確認する。

## 完了条件

- 提案API `Property::try_new(&UiContext, initial)` を含む1.2設計が承認され、safe Rust APIだけでBOOL/I64 Propertyを生成・取得・設定・closeできる。
- 型不一致や不正BOOL表現を誤って成功値に変換しない。
- Runtime所属/UI threadの検証、token/errorの所有とcleanupを既存設計に従って行う。
- 添付指定の`native_api.rs`に差分がない。
- 試験設計・実装unit台帳・verification recordが同じrevisionを指し、静的検証とCIを通過する。

## 履歴

| 日付 | 対象 | 内容 |
| --- | --- | --- |
| 2026-10-24 | 計画承認 | 利用者の「OK」により計画、DD 1.2、BOOL/I64 consumer test設計を承認。`native_api.rs`は未編集。ローカルの`check`と`minimum`は成功。 |
| 2026-10-24 | 実装・検証完了 | implementation commit `02df263`、CI verification record `40078152572`（Windows/Linux success）。ローカル`tools/dev.py check`、`minimum`、consumer testも成功。計画/DD commitは`ec2f5ab`。 |
