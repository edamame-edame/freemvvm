# Python Binding・Command・UI Controls wrapper

- 計画版: 0.2改訂案
- 状態: 計画v0.2を利用者承認済み（「ok」、2026-10-26）。Python Controls unit完了。次段階はWindow peer/close request
- 作成日: 2026-10-26
- 対象: 0.0.1向けPython MVVM/初期UI縦断に必要なBinding・AsyncCommand・UI Controls wrapper
- 作業branch案: `impl-python/runtime-wrapper`を継続
- 親branch: `impl-rust/core-ui-controls-c-abi`由来。現在のPython branchはこのCore APIs branchを祖先に持つ。C++/C# wrapper branchを親にせず、その差分をmergeしない

## 背景と改訂理由

[0.0.1 release計画](release-0.0.1.md)はC++、C#、Rust、PythonそれぞれからWindow、基本Control、表示/入力、Binding、Commandを利用できることを目標とする。Python RuntimeとProperty（BOOL/I64・UTF-8・F64・DECIMAL）は完了済みで、初期UI Control Core/C ABIも`impl-rust/core-ui-controls-c-abi`で完了している。現在のPython branchは同Core API branchの子孫である。C# UI Control wrapperは別のsibling branchで完了しているが、Python実装へは取り込まず、要求・共通C ABI・Python自身の承認設計を正本とする。

v0.1ではBindingとCommandがPythonに既にあるものとしてControls計画を作成したが、source/DD確認の結果、Python wrapperはRuntimeとPropertyまでで、BindingとAsyncCommand公開APIが未実装だった。RQ-09-004/005/006とrelease目標を満たすには必要な依存であるため、v0.2で先行unitを追加する。API・caseはPython DDと試験設計で個別に提案・承認し、未承認設計を確定仕様にしない。

## 目的と段階

Python consumerが承認済みCore/C ABIのProperty・Binding・AsyncCommand・初期Control機能を使い、WindowをrootとするUI treeで表示・入力・Command起動を構成できるようにする。C ABIのstatus、thread制約、handle/registration ownership、Coreが返すnative peer unsupported結果をPython層で変質させない。

段階はPython Binding、Python AsyncCommand、Controls/Tree/Focus facade、Window peer/close requestの順とし、各段階のDD・試験設計・実装承認および実装unitを分ける。

実画面表示、OS固有peer、配布package、PyPI公開、0.0.1 release判定は本計画の完了に含めない。

## 根拠・対象要求

- 要求: [要求仕様索引](../11_requirements/reqirements-specification.md)、[Core機能](../11_requirements/RQ-02_core-features.md)（RQ-02-001～004、RQ-02-007～014）、[言語対応](../11_requirements/RQ-03_language-support.md)（RQ-03-004）、[所有・callback](../11_requirements/RQ-05_ownership-callbacks.md)（RQ-05-001～003、RQ-05-010～013）、[thread/dispatch](../11_requirements/RQ-06_threading-dispatch.md)（RQ-06-001、RQ-06-003、RQ-06-010）、[UI範囲](../11_requirements/RQ-09_ui-scope.md)（RQ-09-001～006、RQ-09-043～049、RQ-09-059、RQ-09-067、RQ-09-070）。要求本文と承認履歴をAPI提案の前に再確認する。
- 基本設計: [UI Tree/Layout/Controls](../12_design/ui/tree-layout-controls.md)、[Runtime](../12_design/runtime-objects/runtime.md)、[Property/Binding](../12_design/events/notifications-commands.md)。文書版全体の承認状態を着手前に確認する。
- 詳細設計: [Python wrapper](../13_detail_design/wrappers/python.md) v1.6、[共通C ABI](../13_detail_design/runtime-objects/abi.md)、[UI Controls](../13_detail_design/ui/controls.md)、[UI Tree/Layout](../13_detail_design/ui/tree-layout.md)、[Input/Focus](../13_detail_design/ui/input-focus.md)、[objects/values](../13_detail_design/runtime-objects/objects-values.md)。Python API、owner関係、call flowが未定義の範囲をPython DD改訂案へ提案し、全文承認前に確定仕様として扱わない。
- 試験設計: [wrapper単体試験](../32_unit_test/wrappers.md)、[UI単体試験](../32_unit_test/ui.md)、[単体試験方針](../32_unit_test/unit-test-strategy.md)。wrapper fake ABI試験とCore/C ABIの既存試験を区別し、既存UI設計の合格をPython wrapper合格に読み替えない。
- 作業規則: [Python coding rule](../00_rule/coding_rule_Python.md)、[Git rule](../02_git/rule.md)、[implementation unit inventory](../32_unit_test/implementation-test-inventory.md)。

## 提案範囲

- 先行公開facade: `Binding`（既存Property間）と非同期`Command`（既存C ABI command lifecycle）。公開型・mode・callback signature・戻り値/例外・解除・cancel・close semanticsをそれぞれのPython DD改訂で提案し、raw ctypes handlesは公開しない。
- UI公開facade候補: Window、StackPanel、Text、TextBox、Button、および共有Widget/Container契約。先行するProperty/Binding/Command wrapperを再利用する。
- 生成、Property取得、Container/Window contentの設定・取得、StackPanel設定、Button-Command接続、Focus要求をPython owner/call lease/error規則へ接続する。各段階の正確な型名・メソッド・戻り値・例外写像はPython DDを順次改訂して提案する。
- `Window.open/close`、close-request callback/defer/resolveとPython callback rootの寿命は複雑なため、基本Control/tree facadeとは別implementation unitに分ける。Core/C ABIに存在しない動作は追加せず、peer未接続の結果をそのまま表す。
- focused fake `NativeApi` testsと別process real C ABI consumerを追加する。consumerはGUI表示を前提にせず、利用できるC ABI操作の結果を確認する。

対象外: Core/Rust/C ABI変更、他言語wrapper変更、OS固有Window/Renderer/Input/IMEの新規実装、配布/loader、release/tag、analyzer実装、要求や既承認Core設計の変更。

## 実施順と承認境界

1. [x] 現在のbranch/worktree、unit台帳、0.0.1計画、Python Runtime/Property実装、Core API branch祖先関係を確認した。C# sibling branchは別に完了しておりmergeしない。
2. [x] 計画v0.1を利用者承認後、日本語commit `789cf3b`でpushした。
3. [x] Python public surfaceとrelease planを照合し、Python Binding/AsyncCommand wrapperが未実装というv0.1の前提誤りを確認した。
4. [x] v0.2の対象範囲・依存順について利用者の「ok」により承認を得た（2026-10-26）。これはDD・試験設計・実装の承認ではない。
5. [x] 承認後、改訂計画を日本語commit/pushする。
6. [x] 各段階で要求・承認履歴、BD/DD、C ABIを照合し、順に設計案を作成・承認する。Python DD v1.7 Binding、v1.8 AsyncCommand、v1.10 UI Controls/Tree/Focusは2026-10-26に全文承認済み。次候補はv2.0 Window/close requestで、その全文承認は別途行う。
7. [x] 各段階に対応するwrapper fake ABI/real C ABI consumer casesを`.docs/32_unit_test/wrappers.md`へ追加する。Binding caseを含む0.9、AsyncCommand caseを含む1.0、Controls caseを含む1.1は承認済み。Binding/Command/Controls implementation unitsを登録済み。
8. [x] 実装開始承認後、以下の独立unitを一つずつplanned登録し、各focused/consumer testを同一revisionのCI gateへ結ぶ。
   - `UNIT-WRAP-PY-BINDING-001`: Property Binding作成・mode・cancel/close・error snapshot・所有。
   - `UNIT-WRAP-PY-COMMAND-001`: 非同期Command callback root・execute/cancel/completion/state・例外/close lifecycle。
   - `UNIT-WRAP-PY-CONTROLS-001`: Window/StackPanel/Text/TextBox/Button facade、Property/Binding/Command mapping、Tree、StackPanel設定、Focus要求。Window peer open/close callbackを含めない。
   - `UNIT-WRAP-PY-WINDOW-001`: Window peer open/closeとclose-request callback/defer/resolveのowner/root lifecycle。Controls unitに依存させる。
9. [ ] `impl-python/runtime-wrapper`だけでunitを一つずつ実装する。各unitのlocal gatesとそのcommit SHAのWindows/Linux CI成功後に日本語commit/pushし、completeにする。Binding（SHA `d5cfc1f3`、run 40212415054）、Command（SHA `aaf1ce68`、run 40214188291）、Controls（14 fake ABI cases、実C ABI consumer、local `check`/`minimum`成功。最終SHA `84e9947`、Windows/Linux run 40216602669成功）は完了。次のWindow unitは未着手。
10. [ ] 全unit完了後、`tools/dev.py check`、`minimum`、Python focused/consumer tests、document links、unit manifest、`git diff --check`を検証し、最終CI SHAと結果を計画・試験記録・memoryへ反映する。

## 完了条件

- Python Binding/AsyncCommandおよびControl公開APIとtest designがそれぞれ文書全体で承認されている。
- Python consumerが承認済みC ABIを通じ、Property/Binding/CommandをWindow/Control treeへ接続し、StackPanel設定とFocus要求を利用できる。
- close-request callbackのPython root/ownerは登録・解除・in-flight完了時に設計どおり保持・回収され、C ABI境界へ例外を漏らさない。
- 各実装unitとそのtestが同一revisionにあり、Windows/Linux CIが成功する。
- OS peer、実画面描画、package配布やrelease判定を完了と誤認しない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-26 | 計画0.1 / Python UI Controls wrapper | 利用者承認・commit済み | 利用者「ok」。commit `789cf3b`。Python Binding/Commandが既存という誤った前提を含んでいたため、実装着手前に改訂 |
| 2026-10-26 | 計画0.2 / Python Binding・Command・UI Controls wrapper | 利用者承認 | 「ok」によりPython Binding/AsyncCommand先行unitと依存順を追加した改訂を承認。設計・試験設計・実装の承認は別 |
| 2026-10-26 | UNIT-WRAP-PY-CONTROLS-001 | 実装開始承認 | 利用者の「実装開始」により、承認済みDD v1.10・test design v1.1のControls/Tree/Focus範囲の実装を開始 |
| 2026-10-26 | UNIT-WRAP-PY-CONTROLS-001 | 完了 | 14 fake ABI tests、実C ABI consumer、Windows local `check`/`minimum`、Windows/Linux CI run 40216602669を確認。SHA `84e994715c5fbe189e4fd3d68f351d684516440a` |
