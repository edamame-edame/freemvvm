# C++ Input event wrapper実装計画

- 計画版: 0.1
- 状態: 実装・local検証済み。working branchへpush後、同revision CI確認待ち
- 対象branch: `impl-cpp-wrapper/input-focus`
- 親branch: `impl-rust/core-input-focus`
- unit: `UNIT-WRAP-CPP-INPUT-001`
- 対象version: `0.0.1`

## 目的

承認済みC++ wrapper DD 1.5と共通C ABI DD 1.6に従い、C++17から全Widgetの入力イベントを安全に購読し、callback中だけ利用できるInputEventView、move-only Subscription、例外隔離を提供する。

## 実装範囲

- `Widget::subscribe_input`、InputEventMask、5 event kind、key/modifier/text/pointer payload getter、set_handledを追加する。
- fake ABI境界を追加し、5 event、wrong-kind getter、callback view寿命、登録失敗、cancelとcloseの分離、callback例外とfailure handler/diagnosticを検証する。
- public consumerから全具象Widget wrapperで登録・cancel・releaseし、実C ABI symbolとの接続を確認する。OS event配送・実機入力は合格条件にしない。
- callback userdataは登録成功時だけCoreへ移し、destroy callbackで一度解放する。

## 対象ファイル

- `wrappers/cpp/include/freemvvm/ui_controls.hpp`
- `wrappers/cpp/src/ui_controls.cpp`
- `wrappers/cpp/src/runtime.cpp`
- `wrappers/cpp/src/runtime_internal.hpp`
- `wrappers/cpp/tests/input_event_tests.cpp`
- `wrappers/cpp/tests/package_consumer/main.cpp`
- `CMakeLists.txt`
- `tools/dev.py`
- `tools/units.yaml`
- `.docs/32_unit_test/wrappers.md`
- `.docs/32_unit_test/implementation-test-inventory.md`

## 実行・完了条件

1. `cpp-input-test` focused fake ABI suiteを通す。
2. `cpp-input-consumer-test`でpublic API・real C ABI registration/cancelを通す。
3. exception-enabled focused executionとGCC `-fno-exceptions`のInput implementation/test translation-unit compileを通す。
4. `cargo fmt --all -- --check`は変更なしでも維持し、利用可能なC++ format/lint、document links、`git diff --check`を通す。
5. unitとsource/testを同一revisionで検証し、実装完了後に日本語commit・pushする。CI状態はそのrevisionで別記録する。

## 根拠

- 設計: [C++ Input event wrapper API](../13_detail_design/wrappers/cpp.md)、[共通Input event subscription ABI](../13_detail_design/runtime-objects/abi.md)
- 試験: [wrapper試験設計](../32_unit_test/wrappers.md#ui-input-event-callbackborrowed-view) `UT-WRAP-CPP-INPUT-001`、`UT-CONSUMER-CPP-INPUT-001`
- 親計画: 0.0.1 Input / Focus event path実装計画。Rust/C++/C#/Python wrapperは共通Core branchを親とし、互いを親にしない。

## 作業記録

| 日付 | 状態 | 内容 |
| --- | --- | --- |
| 2026-11-03 | 着手 | Core Input branchを親にC++ wrapper branchを作成し、承認済みC++ UI Control branchを統合・再検証。commit `35b10f1` |
| 2026-11-03 | local検証 | MSVC focused fake ABI、C++ CTest全15件、relocated installed-package consumer、GCC `-fno-exceptions`でInput implementation/test TU compileをpass。document link check、Rust fmt、`git diff --check`もpass。push後に同revision CIを確認する |
