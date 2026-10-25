# Python AsyncCommand wrapper

- 計画版: 0.1
- 状態: 計画・DD・試験設計・実装開始承認済み。実装とWindows local検証完了、同一実装revisionのCI確認待ち
- 作成日: 2026-10-26
- 実装branch: `impl-python/runtime-wrapper`
- 親branch: `impl-rust/core-ui-controls-c-abi`から派生したPython wrapper branchを継続。C++/C# sibling wrapper branchはmergeしない

## 目的と境界

Pythonから承認済みAsyncCommand C ABIを使い、UI thread上の開始callback、アプリ選択worker、cancel token、UI dispatch経由の終端結果、state/can-execute/failure snapshotを扱うfacadeを追加する。公開面は[Python wrapper DD](../13_detail_design/wrappers/python.md) v1.8に従い、Python側独自のCore状態機械を実装しない。

対象外: Rust Core/C ABI変更、他言語wrapper変更、内蔵executor/thread pool、`asyncio`統合、Window/Control facade、専用progress object/API、Command通知/Subscription、package配布、release/tag。本案の`CommandOperation.post`は既存`runtime_post`をPythonから使うための最小callback dispatchであり、一般的なRuntime dispatcher APIへ拡張しない。

## 前提と承認境界

- 依存: 完了済みPython Runtime/Property/Binding wrapper、承認済みAsyncCommand C ABIとCore command lifecycle、Python Runtime callback root/exception handler設計。
- 要求根拠: RQ-02-004、RQ-03-004、RQ-05-002/006/007/008/009/012、RQ-06-001/002/003/004/008/009/010/011/013。
- DD: Python wrapper v1.8、test design: wrapper単体試験 v1.0。両文書とも利用者が2026-10-26に全文承認済み。
- implementation unit: `UNIT-WRAP-PY-COMMAND-001`、focused fake API `UT-WRAP-PY-COMMAND-001`、real C ABI consumer `UT-CONSUMER-PY-COMMAND-001`。unit inventoryへplanned登録済み。
- DD承認、test-design承認、implementation-start承認は別々に確認する。計画更新はcommit前に利用者レビューする。実装承認後は同一unitのlocal checksとWindows/Linux CIを経て日本語commit/pushする。

## 段階

1. [x] Python Binding unitを完了し、branch headへpushした（実装SHA `d5cfc1f3`、Windows/Linux CI run 40212415054 success）。
2. [x] 要求、ADR-0001、Command/Runtime/ABIの承認済み上位設計、既存Rust/C# wrapper案を照合した。
3. [x] Python DD v1.8 AsyncCommandとwrapper test design v1.0を作成した。
4. [x] 利用者の「ok」により、計画・DD v1.8・試験設計v1.0を文書単位で承認（2026-10-26）。
5. [x] `UNIT-WRAP-PY-COMMAND-001`をplanned登録。承認された計画・試験設計・台帳更新を日本語commit/pushした。
6. [x] 実装開始承認後にPython `AsyncCommand`/worker operation facade、Ctypes adapter `command_*`/`runtime_post`、fake API tests、別process consumerを実装した。
7. [x] focused/consumer tests（fake API 4件、実C ABI success/failure/cancel/re-execute/post/property）、`tools/dev.py check`、`minimum`、docs links、unit manifest設定、`git diff --check`をWindows localで検証した。
8. [ ] 日本語commit/pushして同じ実装SHAのWindows/Linux CIを確認し、成功後にunitをcompleteとして記録する。

## 完了条件

- Python callable/rootはcommand create成功時だけCoreへ移り、start callback例外やcallback cleanupがC ABIを越境しない。
- worker用Command/cancel/Runtime参照は所有期間中維持され、workerはUI-affine操作を直接行わない。
- cancel request、終端結果、post受付、UI上のstate commitを区別する。
- closeとhandle releaseを分け、close/failure/release失敗でowner tokenやrootを失わない。closeはworkerを待たない。
- fake ABIで失敗・不正出力・所有競合を検証し、実C ABI consumerで代表的な実行flowを確認する。Core内部状態機械の正しさをwrapper testへ転嫁しない。
- 実装unitとtestが同一revisionでWindows/Linux CIを通過する。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-10-26 | 計画0.1 / Python AsyncCommand wrapper | 利用者承認 | 「ok」により計画・DD v1.8・test design v1.0を承認。unit planned登録済み。実装開始は別承認 |
