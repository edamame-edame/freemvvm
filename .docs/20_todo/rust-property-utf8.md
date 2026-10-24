# Rust wrapper: UTF-8 Property

- 更新日: 2026-10-24
- 状態: 計画承認済み（利用者、2026-10-24）。実装ローカル検証済み、CI確認中
- 対象ブランチ: `impl-rust/wrappers-runtime-api`
- 親: [Rust BOOL/I64 Property](rust-property-bool-i64.md)

## 目的と範囲

既存のsafe `Property<T>`へ`String`を加え、共有C ABIのUTF-8 Propertyを生成・取得・設定・closeできるようにする。byte length方式に従い、埋め込みNULや多byte文字を正確に扱う。F64/DECIMAL、notification、Binding、package配布経路は対象外とする。

添付の`handle.rs`とworkspace側の[`handle.rs`](../../wrappers/rust/freemvvm/src/handle.rs)は読み取り専用とし、編集しない。既存の`Owned<T>`を利用し、追加のownership機構は作らない。

## 根拠

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-03-003](../11_requirements/RQ-03_language-support.md#rq-03-003)、[RQ-03-007](../11_requirements/RQ-03_language-support.md#rq-03-007)、[RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003)、[RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- 承認済み設計: [Rust wrapper DD 1.2](../13_detail_design/wrappers/rust.md)、[共通C ABI DD 1.2](../13_detail_design/runtime-objects/abi.md)「UTF-8 Property create/get/set」「buffer query and retry」
- 試験方針: [unit test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)

## 実装・確認項目

1. `freemvvm-sys`へUTF-8 create/get/set raw declarationを追加する。
2. `PropertyValue`を非Copy値へ対応させ、safe `Property<String>`の生成/get/set/closeを実装する。
3. getterは必要byte数を照会し、allocation failureをstatus化する。`UI_BUFFER_TOO_SMALL`なら必要長を取り直して再試行し、buffer値を部分返却しない。
4. UTF-8 bytesを`String`へ検証して返し、埋め込みNULを保持する。setter/createはUTF-8 byte lengthを渡し、Rust StringのUTF-8 validityを利用する。
5. `handle.rs`と添付snapshotには変更を加えない。
6. 独立Rust consumerでcreate/get/set/closeを検証し、日本語・補助文字・埋め込みNUL・空文字列を含める。
7. unit-test設計、implementation inventory、dev commandを追加し、docs/format/lint/build/test、Rust 1.85、Windows/Linux CIを確認する。

## 完了条件

- `Property<String>`がsafe APIから共有C ABIを使って利用できる。
- UTF-8文字列のbyte長、埋め込みNUL、空文字列を保持し、不正なnative出力を正常値として返さない。
- `UI_BUFFER_TOO_SMALL`後の再試行とメモリ確保失敗を安全に処理する。
- 添付とworkspaceの`handle.rs`に差分がなく、既存BOOL/I64 consumerも引き続き成功する。
- manifest、verification record、実装unitが同じCI revisionを示す。

## 履歴

| 日付 | 対象 | 内容 |
| --- | --- | --- |
| 2026-10-24 | 計画・実装案 | Rust Propertyの次unitとしてUTF-8 Stringを選択。`handle.rs`は読み取り専用とし、consumer/focused testsを追加。`tools/dev.py check`、`minimum`、UTF-8 consumerとgetter retry/error testsは成功。commit/push前に計画・実装差分を利用者レビューする。 |
| 2026-10-24 | 計画承認 | 利用者の「OK」により本計画・consumer test案・実装差分のcommit/pushを承認。添付consumer例を確認し、UTF-8/NUL/empty coverageを含むことを確認。 |
