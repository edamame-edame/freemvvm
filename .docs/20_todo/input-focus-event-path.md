# 0.0.1 Input / Focus event pathの次工程計画

- 計画版: 0.1案
- 状態: 利用者レビュー待ち。未commit・未実装
- 対象version: `0.0.1`
- 対象: 承認済みInput/Focus要求を、Core UI処理とWindows native event sourceへつなぐ前の設計・実装unit計画
- Core APIs branch案: `impl-rust/core-input-focus`
- Windows Platform子branch案: `impl-rust/platform-adapter-windows-input`
- 関連計画: [0.0.1 release計画](release-0.0.1.md)、[Windows Platform Adapter計画](windows-platform-adapter.md)

## 背景

Windows close-request接続までのCore/Windows unitは同revision CIを通過し、PR #4でCore APIs branchへ統合された。現行Coreには`FocusManager`とFocus request/Window activation状態があるが、承認済みInput/Focus DDの`InputRouter`とroute dispatchは実装されていない。Windows `WndProc`も現在は`WM_CLOSE`、`WM_SIZE`、`WM_PAINT`を扱い、keyboard/text/pointer eventをCoreへ渡していない。

一方、承認済みDDには実装前に具体化が必要な境界が残る。Input event field/ABI表現とpublic focus event APIは未決とされ、RQ-09-068もpointer event種類の一部を候補扱いにしている。ここを既存の承認済み範囲だけで実装できるか先に確定する。

## 根拠

- 要求: [RQ-09 UI](../11_requirements/RQ-09_ui-scope.md)（keyboard/focus/input routing）、[RQ-06 threading/dispatch](../11_requirements/RQ-06_threading-dispatch.md)
- DD: [UI InputとFocus](../13_detail_design/ui/input-focus.md) 1.1全文承認済み、[Platform抽象化](../13_detail_design/platform/abstraction.md) 1.2全文承認済み、[Platform内部層](../13_detail_design/platform/internal-layers.md) 2.2全文承認済み
- 試験設計: [UI unit test](../32_unit_test/ui.md)（`UT-INPUT-001`、`UT-FOCUS-001`、`UT-FOCUS-002`）
- 実装状況: `crates/core-runtime/src/ui_focus.rs`はFocus状態管理、`crates/platform-windows/src/win32.rs`はWindow message workerを持つ。InputRouterおよびWindows入力event接続は未実装

## 推奨方針

まず設計境界を閉じ、その承認済み範囲だけをCore / Windowsの二unitへ分けて実装する。公開C ABIやControl event subscriptionの追加は現行DDで一意に導けないため、初回提案では既存公開面へ追加しない。RQ-09-068のhandled/bubbling保証がpublic event surfaceを必要とする場合は、その要求・DD・test変更案を実装unitから切り離して先にレビューする。

特定keyの割当、Tab traversal、shortcut/action binding、mouse capture、未承認pointer event種別、IME compositionのOS固有詳細を実装都合で追加しない。実装可能範囲を確定できない場合はsource変更前にレビューへ戻す。

## 作業分割案

| unit | 責務 | 完了条件 |
| --- | --- | --- |
| 設計境界reconciliation（実装unit登録前） | Input event payload、Core/UI dispatcherへの投入点、Keyboard/Text区分、hit-test targetとLogical route、handled可否、focus/activation連携の承認状態を文書全体で確認する | public API追加の要否、初期event種別、pointer/IMEの対象外範囲をレビュー可能な提案にする。承認前の仕様・source変更をしない |
| `UNIT-CORE-091`候補 | private Core InputRouter / neutral event dispatchを実装し、Focus target、Logical route snapshot、detach再検証、handled後停止、focusなしkeyboard/text破棄を処理する | `UT-INPUT-001`、`UT-FOCUS-001`、`UT-FOCUS-002`の承認済み範囲をfake eventで検証。Core/workspace gates、Rust minimum、文書検査が同revisionで成功 |
| `UNIT-PLATFORM-WINDOWS-007`候補 | Win32 worker上の採用済みnative eventを中立eventへcopyし、generation付きprivate sink経由でCore UI dispatcherへ投入する。OS native focusはCore Focusと同期する | Windows messageをfake境界で検証。callbackをWndProc内で呼ばない。Windows target build、workspace gates、Rust minimum、文書検査が同revisionで成功 |

具体的なunit ID・path mappingは設計境界の確認後に`.docs/32_unit_test/ui.md`、implementation inventory、`tools/units.yaml`へ登録する。

## 必須確認

- key down/upとText commitを別eventとして扱い、Windows messageをCore/user callbackから同期waitしない。
- stale Window generation、inactive/detached/disabled対象、Runtime shutdown時にeventを誤適用しない。
- Pointer hit-test座標、DIP変換、clip、target-to-Logical routeの担当境界を承認済み設計から一意に確定する。追加仕様が必要なら別途承認する。
- Focus callback/public event APIの有無を実装上の都合で決めない。
- Windows実機keyboard/mouse/IME動作はfake/CIと区別し、必要なGUI smokeを0.0.1 release qualificationへ記録する。

## ブランチ・Git

- 計画のレビュー承認前にcommit/pushしない。
- 実装開始承認後は0.0.1のCore APIs childで`UNIT-CORE-091`を実装・検証し、日本語commit/pushする。
- Core unitの完了後、そのCore branchを親としてWindows Platform childを作り、`UNIT-PLATFORM-WINDOWS-007`を実装・検証・commit/pushする。
- 関連TODOが完了した後、Core/Windows branch依存順にmerge PRをまとめて提出する。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-01 | 計画0.1案 | 利用者レビュー待ち | Windows close-request統合後の候補。計画承認・commit・実装開始・公開C ABI追加を意味しない |
