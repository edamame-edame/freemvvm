# C++ Property DECIMAL 対応計画

- 更新日: 2026-10-23
- 状態: 完了（実装・ローカル検証・Windows/Linux CI成功）
- 対象: `freemvvm::Decimal64` と `freemvvm::Property<Decimal64>` の値型mapping

## 背景と根拠

C++ Property の [BOOL/I64](cpp-property-wrapper.md)、[UTF-8](cpp-property-utf8.md)、[F64](cpp-property-f64.md) が完了した。RQ-02-017の初期値型のうち残るDECIMALをC++ wrapperへ接続する。

- 要求: [RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)。`RQ-02-020`の同値設定・格納表現保持はCoreのProperty責務として維持する。
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreを値の正本とし、C++はversioned C ABIを介する薄いwrapper。
- 詳細設計: 承認済み[C++ wrapper DD v1.2](../13_detail_design/wrappers/cpp.md)は`Decimal64`と`Decimal64::try_from_parts`を示し、Property値種別ごとの明示mappingを要求する。承認済み[共通C ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)および[通知・Binding DD v1.1](../13_detail_design/events/notifications-binding.md)はcanonical `uint64_t`表現とDECIMALの範囲・同値性を定める。

## 実装提案

- 専用`Decimal64`値型はcanonical `uint64_t`表現を保持し、公開factory `try_from_parts(bool negative, std::uint8_t scale, std::uint64_t coefficient)` とread-onlyなsign/scale/coefficient accessorsを提供する。
- scaleが0～31、coefficientが0～2^58−1の範囲外なら既存の`UI_INVALID_ARGUMENT`をraw statusとして返し、native error snapshotは付けない。値をmask・切捨て・丸めない。十進文字列parse/format、浮動小数変換、算術演算は追加しない。
- `Property<Decimal64>`は既存Property owner・move・close・cleanup supervisorを再利用し、C ABIの`property_create_decimal/get_decimal/set_decimal`へcanonical bitsを直接渡す。全64-bit patternは正規表現として有効なのでgetter値を変換・拒否しない。
- Coreが行う数値同値判定、同値set時の格納表現保持と通知抑止をC++で重複実装しない。wrapper testsは値のbits pass-throughとowner/error境界を検証し、Core testの代替にしない。

## 試験・実施手順

1. 本計画のレビュー・承認後、計画を日本語commitで先にcommit/pushする。
2. `UT-WRAP-CPP-PROP-DECIMAL-001` と `UT-PKG-CPP-005` を`.docs/32_unit_test/wrappers.md`および`tools/units.yaml`へsource変更前に登録する。実装unitは`UNIT-WRAP-CPP-005`とする。
3. `Decimal64`のparts境界（scale 0/31、coefficient 0/max、範囲外）、canonical bits往復と暗黙変換がないことを検証する。
4. fake ABIで`Property<Decimal64>` create/get/set、scale/sign/zeroの表現を含むcanonical bits pass-through、create/set error snapshot、get raw status、failed setの不変、move/close/release retryを試験する。Coreの数値同値判定は対象外とする。
5. relocated CMake package consumerから`Decimal64` factoryと`Property<Decimal64>`を使用し、create/get/set/closeを検証する。
6. focused tests、全C++ wrapper CTest、package consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書検査とWindows/Linux CIを同一revisionで確認する。
7. 実装境界とCore/wrapperの責務分担を`.docs/01_memory/`へ記録する。

2026-10-23のローカル検証ではDECIMAL focused CTest、relocated package consumer、全C++ CTest 9件、`tools/dev.py check`、Rust 1.85 minimum check、manifest JSON・文書リンク検査が成功した。実装commit [`a06ac21`](https://github.com/edamame-edame/freemvvm/commit/a06ac219984bf9e8955d34261ebb057038127ca9) に対する[Windows/Linux CI run 39953038033](https://github.com/edamame-edame/freemvvm/actions/runs/39953038033)も両runner成功。artifact内の`LastTest.log`で`wrapper.cpp.property.decimal`が両runnerとも成功したことを確認した。

## レビュー対象

承認対象は、`Decimal64`をparts factory/accessorsを持つcanonical-bits値型として追加すること、無効partsを`UI_INVALID_ARGUMENT`で拒否すること、`Property<Decimal64>`の直接mapping、上記試験範囲である。提案どおりなら「承認します」と返信してください。修正希望があれば本書を直して再提示する。

## 承認履歴

利用者は2026-10-23に「ok」と返信し、本計画の`Decimal64` API、invalid-parts status、Property mapping、および試験範囲を承認した。承認後、計画を先にcommit/pushしてから試験台帳登録とsource変更を開始する。
