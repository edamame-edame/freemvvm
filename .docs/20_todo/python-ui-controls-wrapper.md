# Python UI Controls wrapper

- 計画版: 0.1
- 状態: 計画案（利用者レビュー待ち。未commit・未実装）
- 作成日: 2026-10-26
- 対象: 0.0.1初期UI Control Core/C ABIをPython `ctypes` wrapperから構成・操作する
- 作業branch案: `impl-python/runtime-wrapper`を継続
- 親branch: `impl-rust/core-ui-controls-c-abi`由来。現在のPython branchはこのCore APIs branchを祖先に持つ。C++/C# wrapper branchを親にせず、その差分をmergeしない

## 背景

[0.0.1 release計画](release-0.0.1.md)はC++、C#、Rust、Pythonから共通の初期UI縦断機能を利用できることを目標とする。Python Runtime、Property（BOOL/I64・UTF-8・F64・DECIMAL）は完了済みだが、Python公開APIにWindow/Control/Tree/Layout/Focus facadeはない。初期UI Control Core/C ABIは`impl-rust/core-ui-controls-c-abi`で完了し、現在のPython branchはそのbranchの子孫である。C# UI Control wrapperも別のsibling branchで完了しているが、Python実装へは取り込まず、要求・共通C ABI・Python自身の承認設計を正本とする。

## 目的

Python consumerから承認済みCore/C ABIの初期Control機能を使い、WindowをrootとするUI treeを構成し、既存のProperty/Binding/CommandをControlへ接続できるPython facadeを用意する。C ABIのstatus、thread制約、Property/Commandの所有、Coreが返すnative peer unsupported結果をPython層で変質させない。

実画面表示、OS固有peer、配布package、PyPI公開、0.0.1 release判定は本計画の完了に含めない。

## 根拠・対象要求

- 要求: [要求仕様索引](../11_requirements/reqirements-specification.md)、[Core機能](../11_requirements/RQ-02_core-features.md)（RQ-02-001～004、RQ-02-007～014）、[言語対応](../11_requirements/RQ-03_language-support.md)（RQ-03-004）、[所有・callback](../11_requirements/RQ-05_ownership-callbacks.md)（RQ-05-001～003、RQ-05-010～013）、[thread/dispatch](../11_requirements/RQ-06_threading-dispatch.md)（RQ-06-001、RQ-06-003、RQ-06-010）、[UI範囲](../11_requirements/RQ-09_ui-scope.md)（RQ-09-001～006、RQ-09-043～049、RQ-09-059、RQ-09-067、RQ-09-070）。要求本文と承認履歴をAPI提案の前に再確認する。
- 基本設計: [UI Tree/Layout/Controls](../12_design/ui/tree-layout-controls.md)、[Runtime](../12_design/runtime-objects/runtime.md)、[Property/Binding](../12_design/events/notifications-commands.md)。文書版全体の承認状態を着手前に確認する。
- 詳細設計: [Python wrapper](../13_detail_design/wrappers/python.md) v1.6、[共通C ABI](../13_detail_design/runtime-objects/abi.md)、[UI Controls](../13_detail_design/ui/controls.md)、[UI Tree/Layout](../13_detail_design/ui/tree-layout.md)、[Input/Focus](../13_detail_design/ui/input-focus.md)、[objects/values](../13_detail_design/runtime-objects/objects-values.md)。Python API、owner関係、call flowが未定義の範囲をPython DD改訂案へ提案し、全文承認前に確定仕様として扱わない。
- 試験設計: [wrapper単体試験](../32_unit_test/wrappers.md)、[UI単体試験](../32_unit_test/ui.md)、[単体試験方針](../32_unit_test/unit-test-strategy.md)。wrapper fake ABI試験とCore/C ABIの既存試験を区別し、既存UI設計の合格をPython wrapper合格に読み替えない。
- 作業規則: [Python coding rule](../00_rule/coding_rule_Python.md)、[Git rule](../02_git/rule.md)、[implementation unit inventory](../32_unit_test/implementation-test-inventory.md)。

## 提案範囲

- 公開facade候補: Window、StackPanel、Text、TextBox、Button、および共有Widget/Container契約。既存Runtime/Property/Binding/Commandを再利用し、raw ctypes handlesは公開しない。
- 生成、Property取得、Container/Window contentの設定・取得、StackPanel設定、Button-Command接続、Focus要求をPython owner/call lease/error規則へ接続する。正確な型名・メソッド・戻り値・例外写像はPython DD v1.7で提案する。
- `Window.open/close`、close-request callback/defer/resolveとPython callback rootの寿命は複雑なため、基本Control/tree facadeとは別implementation unitに分ける。Core/C ABIに存在しない動作は追加せず、peer未接続の結果をそのまま表す。
- focused fake `NativeApi` testsと別process real C ABI consumerを追加する。consumerはGUI表示を前提にせず、利用できるC ABI操作の結果を確認する。

対象外: Core/Rust/C ABI変更、他言語wrapper変更、OS固有Window/Renderer/Input/IME、配布/loader、release/tag、analyzer実装、要求や既承認Core設計の変更。

## 実施順と承認境界

1. [x] 現在のbranch/worktree、unit台帳、0.0.1 plan、Python完了済みRuntime/Property、およびCore API branch祖先関係を確認した。worktree clean。
2. [ ] 本計画0.1のscope・branch方針について利用者承認を得る。承認前は計画をcommit/pushしない。
3. [ ] 計画承認後、この計画を日本語commit/pushする。これはPython DD・試験設計・実装の承認ではない。
4. [ ] 要求本文・承認履歴と上位設計を確認し、Python wrapper DD v1.7へControl公開surface、Treeと独立ownerの関係、既存Property/Binding/Commandとの接続、thread/error/cleanup flow、native peer非対応結果を提案する。Python公開APIが要求・C ABIより強い保証を足さないことを差分確認する。
5. [ ] Python wrapper DD v1.7全文の承認を得る。DD承認は実装開始承認と分ける。
6. [ ] wrapper test designをv0.9へ改訂し、fake ABI/real C ABI consumerの操作・期待値・後始末・対象RQを具体化して文書全体の承認を得る。
7. [ ] 実装開始承認後、次の独立unitを`tools/units.yaml`へplanned登録し、test commandをCI gateに結ぶ。
   - `UNIT-WRAP-PY-CONTROLS-001`: Window/StackPanel/Text/TextBox/Buttonのtyped facade、property/command mapping、Tree操作、StackPanel設定、Focus要求。Windowのnative peer open/close callbackは含めない。
   - `UNIT-WRAP-PY-WINDOW-001`: Window peer open/closeとclose-request callback/defer/resolveのowner/root lifecycle。Controls unitに依存させる。
8. [ ] Python wrapper branchだけでimplementation unitを一つずつ実装し、focused fake ABI testと対応するreal C ABI consumerを同revisionに含める。各unitのlocal gatesとWindows/Linux CI成功後に日本語commit/pushし、unitをcompleteにする。
9. [ ] 全unit完了後、`tools/dev.py check`、`minimum`、Python focused/consumer tests、document links、unit manifest、`git diff --check`を検証し、最終CI SHAと結果を計画・試験記録・memoryへ反映する。

## 完了条件

- Python wrapper DDとtest designがそれぞれ文書全体で承認されている。
- Python consumerが承認済みC ABIを通じ、初期Window/Control tree、Property/Binding/Command連携、StackPanel設定、Focus要求を利用できる。
- close-request callbackのPython root/ownerは登録・解除・in-flight完了時に設計どおり保持・回収され、C ABI境界へ例外を漏らさない。
- 各実装unitとそのtestが同一revisionにあり、Windows/Linux CIが成功する。
- OS peer、実画面描画、package配布やrelease判定を完了と誤認しない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-26 | 計画0.1 / Python UI Controls wrapper | 利用者レビュー待ち | Python DECIMAL Property完了後の次候補として作成。未commit・未実装 |
