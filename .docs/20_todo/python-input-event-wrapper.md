# Python Input event wrapper

- 計画版: 0.1
- 状態: 計画0.1・統合文書版は利用者承認済み。Python baseline統合、Input wrapper実装、local検証、Windows/Linux CI完了
- 対象version: `0.0.1`
- 作業branch: `impl-python-wrapper/input-focus`
- Core/C ABI branch: `impl-rust/core-input-focus`（revision `4fdbea1`）
- 取り込み対象: `impl-python-wrapper/window-background-config`（Python wrapper実装revision `3e018a0`、CI run `41178355548` success）
- 上位計画: [Input / Focus event path実装計画](input-focus-event-path-implementation.md) フェーズ5
- 対象unit: 予定登録済み `UNIT-WRAP-PY-INPUT-001`

## 背景

上位計画は4言語のInput wrapperを実装対象として承認済みで、PythonのInput event設計も承認済みである。PythonのRuntime・Property・Binding・Command・Controls/Tree/Focus・Window wrapperは`impl-python-wrapper/window-background-config`にあり、fake ABI testsとconsumer testsも存在する。同branchの最新commit `3e018a0`はCI run `41178355548`で成功している。一方、同branchにInput eventの`subscribe_input`、`InputEventView`、callback root/trampolineはまだ見当たらない。

作業branchはCore Input branchから作り、既存Python wrapper branchを同じ言語の実装として取り込む。別言語branchは取り込まない。取り込み手順は[Python Window背景RGB計画](python-window-background-wrapper.md)の先行wrapper統合方針に合わせる。

## 目的

承認済みPython wrapper設計と要求に沿って、各public WidgetへInput event subscriptionとcallback-scoped event viewを追加し、fake ABI testsとseparate public consumerで検証する。既存Runtime/UI APIやCore/C ABIの挙動は変更しない。

## 設計・要求根拠

- 要求: [RQ-03 Language support](../11_requirements/RQ-03_language-support.md)、特にPython 3、明示終了、owner回収、通常終了協調、base初期化範囲。
- Input/UI要求: [RQ-04 ABI boundary](../11_requirements/RQ-04_abi-boundary.md)、[RQ-05 Ownership/callbacks](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06 Threading/dispatch](../11_requirements/RQ-06_threading-dispatch.md)、[RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)。
- Python wrapper: [Python wrapper DD](../13_detail_design/wrappers/python.md) 2.2全文承認済み（2026-11-06）。Input event API、ctypes/GIL、callable root、error境界、owner/runtime class designを照合する。
- 共通ABI/UI: [共通C ABI](../13_detail_design/runtime-objects/abi.md)、[UI Controls](../13_detail_design/ui/controls.md)、[UI Tree/Layout](../13_detail_design/ui/tree-layout.md)。
- 試験: [wrapper test design](../32_unit_test/wrappers.md) 1.5全文承認済み（2026-11-06）のPython Input cases、[unit inventory](../32_unit_test/implementation-test-inventory.md)、[実装unit台帳契約](../32_unit_test/unit-test-strategy.md)。
- Python規則: [Python coding rule](../00_rule/coding_rule_Python.md)。
- Implementation memory: [Python Input wrapper readiness](../01_memory/python-input-wrapper-readiness.md)。

## 作業順と承認ゲート

1. [x] originを更新しPython wrapper最新branchを確認。Runtime/UI baselineは`impl-python-wrapper/window-background-config`の`3e018a0`、Input API/test pathは未実装。
2. [x] Core Input branchを先頭にし、同一Python wrapper branchの実装・test・examples・必要manifestを統合。競合解決では両側の承認済み内容を保持し、他言語wrapperは含めない。
3. [x] `tools/units.yaml`の`UNIT-WRAP-PY-INPUT-001` source/config/depends_on/test commandsを実在pathに照合し、`ready`へ更新。
4. [x] ctypes native declarations、`InputEventKind`/mask/payload view、Widgetの登録API、callback trampoline、callable/CFUNCTYPE root移管、cancel/close寿命、例外処理を実装した。
5. [x] fake ABI testsで5種類のevent/payload、wrong-kind、登録失敗、borrow失効、root/GC、self-cancel、callback中close、foreign-thread callback、callbackとfailure reporter例外を確認した（9 tests pass）。
6. [x] separate public consumerからfake hostでdispatch、text copy、handled、cancelを検証した。peer起動は実行していない。
7. [x] Python wrapper suite 155件、Python 3.13 AST、C ABI Input focused test、document links、manifest JSON、`git diff --check`はlocal pass。Ruff 0.11.13 format/lintもpass。CI run `42054917806`のWindows/Linux全体bootstrap gates、Windows adapter tests、Linux Platform device testsがpassした。
8. [x] 対応テスト成功後、日本語commit `a8a54dd`（実装）と`22a879f`（CI書式修正）を作成し、本branchへpushした。Input/Focus関連TODO batch完了後に集約PRを作成する。

CI初回runではRuff formatで停止したため`22a879f`で修正した。修正後runはLinux `fonts-noto-cjk` 61.2 MB取得中に20分timeoutとなったが、再実行では依存準備を含めて全jobが成功した。

## 対象外

- C ABI/CoreのInput、Focus、callback、Runtime semantics変更。
- Python Runtime/Window/Widget baselineの変更・再実装。既存Python branchから必要な実装を取り込む。
- Python package/native runtime distribution、PyPI、RID/loader、PyPy/free-threaded supportの新規決定。
- 未承認のPython public API、要求保証、例外方針、threading semanticsの追加。
- OS実機のkeyboard/IME/pointer合否判断。これはPlatform/release qualificationに分ける。

## 完了条件

- public Python consumerが各eventをcallback-scoped viewとして読み、textをPython `str`へcopyし、handled設定とsubscription cancel/releaseを利用できる。
- callback rootは登録成功後に限って保持され、C destroy callback後にsafe-point経由で一度だけ回収される。例外はC ABIを越境しない。
- 必要なfocused/consumer tests、manifest、documentation checksが同一revisionでpassし、Windows/Linux CIがsuccessする。

## レビュー対象

- 既存Python wrapper branchをCore Input branchへ取り込む範囲と競合解決方針。
- branch親子関係、test commands、完了条件。

計画0.1は利用者承認済み。承認済みDDの範囲で実装し、新規の設計事項が見つかった場合だけ別途レビューを挟む。

## 承認履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-05 | 計画0.1 | 利用者承認 | 既存Python wrapperをCore Input branchへ取り込んだ上で、Python Input wrapperを実装する計画を承認 |
| 2026-11-06 | 統合文書 | 利用者承認 | Python DD 2.2、wrapper試験設計1.5、implementation inventory 2.1の統合版を承認 |
