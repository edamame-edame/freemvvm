# Rust Property Binding wrapper

- 更新日: 2026-10-24
- 状態: 計画・Rust DD v1.5・test design v0.8を利用者承認済み。実装中
- 対象ブランチ: `impl-rust/wrappers-runtime-api`（親: `impl-rust/runtime-async-command`）
- 対象unit: `UNIT-WRAP-RS-BIND-001`（test inventory登録は計画承認後）

## 目的

承認済みRQ-07とイベントハンドリング設計にあるProperty BindingをRust safe wrapperから利用できるようにする。既存C ABI/CoreのBinding graph、伝播、初期同期、遅延errorをRust内へ複製せず、型付きAPIと所有・thread境界を提供する。

## 参照元

- 要求: [RQ-07 Binding](../11_requirements/RQ-07_binding.md)、[RQ-05 ownership](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06 threading](../11_requirements/RQ-06_threading-dispatch.md)
- 詳細設計: [Rust wrapper DD v1.5案](../13_detail_design/wrappers/rust.md)、[イベント通知とBinding](../13_detail_design/events/notifications-binding.md)、[C ABI](../13_detail_design/runtime-objects/abi.md)
- 試験: [wrapper test design v0.8案](../32_unit_test/wrappers.md)、[requirements test matrix](../30_release_test/requirement-test-matrix.md)
- 既存実装参考: [C++ Binding public API](../../wrappers/cpp/include/freemvvm/binding.hpp)、[C ABI header](../../crates/c-abi/include/freemvvm/abi.h)

## スコープ

- BOOL / I64 / UTF8 / F64 / DECIMALの同型Property接続。
- OneTime / OneWay / TwoWayのcreateと結果型。
- move-only Binding owner、cancelとclose/Dropの分離、失敗時cleanup保持。
- Binding error snapshotとerror notification subscription。
- Runtime/UI-context検証、C ABI異常結果の防御、callback panic境界。
- fake sys API試験とhost-loop付きnative C ABI integration test。

対象外: Core/C ABI仕様変更、異種型変換、OBJECT Binding、他wrapper、package artifact取得方法の変更、`tools/dev.py`編集。既存`rust-wrapper-runtime-test`でRust runtime integration testを起動できるため、専用command追加はしない。

## 実施手順

1. 利用者がRust DD v1.5案とtest design v0.8案、unitの分割粒度をレビューする。承認前にsource実装やunit manifestの状態変更をしない。
2. 承認後、承認結果をDD/test design/todoへ記録し、`tools/units.yaml`に`UNIT-WRAP-RS-BIND-001`をreadyとして追加する。source変更前にtest designとunit mappingを確定する。
3. Rust `-sys`に既存C ABI Binding関数を追加し、safe crateに`BindingMode`、`BindingDirection`、`Binding<T>`、`BindingError<T>`、`Property::bind_to`を実装する。safe crate内のみでunsafeを使い、`tools/dev.py`は変更しない。
4. fake APIでtoken ownership、結果shape、Runtime mismatch、close/cancel失敗再試行、snapshot保持、error callback移管・destroyを検証する。
5. native integration testで全5値型、3 mode、初期コピー/伝播方向、explicit cancel、closeのみの継続、遅延失敗と回復を確認する。
6. `cargo fmt`、Clippy、workspace test/build、Rust 1.85 minimum、`tools/dev.py check` / `minimum`、document-link/unit inventory validationを通す。
7. 変更ファイル一覧、実行結果、CI Windows/Linuxの合格を確認し、実装unitと同一revisionで試験記録を残す。unit完了後に実装・検証結果をcommit/pushする。

## 完了条件

- 公開APIが承認済みDDと一致し、native token/error ownershipを漏らさない。
- OneTime成功はBinding ownerなし、永続modeは明示cancelまで接続が有効。
- close/Dropだけでは接続が解除されず、明示close失敗ではtokenを保持する。
- 遅延error snapshotは後続回復後も不変で、rejected Propertyを独立ownerとして保持する。
- callback stateは登録成功時だけ移管され、cancel/destroy後に一度だけ破棄される。
- local gatesとWindows/Linux CIが成功し、docs/units/test inventoryが整合する。

## レビュー待ち事項

- Rust API名・enum variant・generic bounds・`Result<Option<Binding<T>>, UiError>`の形。
- C++ wrapper同等にBinding error snapshotとsubscriptionを初回Binding unitへ含める分割。
- integration testを既存`rust-wrapper-runtime-test` commandへ含める運用（`tools/dev.py`は変更しない）。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | 計画とRust DD v1.5・test design v0.8 | 利用者 | 「OK」により承認。実装開始 |
