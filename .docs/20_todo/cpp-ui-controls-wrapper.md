# C++ UI Control wrapper

- 計画版: 0.2改訂案
- 状態: 改訂案・利用者レビュー待ち。0.1はcommit `ac3e7af`としてpush済み。未実装
- 作成日: 2026-10-25
- 対象: 0.0.1初期UI Control Core/C ABIをC++17から構成・操作するwrapper
- branch案: `impl-cpp/ui-controls-wrapper`
- 親branch案: `impl-rust/core-ui-controls-c-abi`（C++ UI Control facadeが依存する最新Core APIs）

## 背景

[0.0.1計画](release-0.0.1.md)はC++、C#、Rust、Pythonから共通UI縦断機能を利用できる状態を目指す。C#は[`csharp-ui-controls-wrapper.md`](csharp-ui-controls-wrapper.md)に従い、Window、StackPanel、Text、TextBox、Button、Property/Binding/Command接続、Focus、Window close requestのfacadeを実装し、commit `b9e2cec`のWindows/Linux CIが成功した。

C++ wrapperにはRuntime、Property、Binding、Subscriptionの実装があるが、Command wrapperはまだない（C++ public includeに`command.hpp`がなく、C++ unit manifestにもCommand unitの完了記録はない）。一方、承認済み[C++ wrapper DD v1.3](../13_detail_design/wrappers/cpp.md)はCommandおよびUI Control/WindowのC++公開型・method・ownership/callback flowを定義していない。Buttonからtyped Commandを接続するにはC++ Command wrapperが前提になるため、Async Command API設計・実装を独立unitとして先行させ、その後にControl facadeと統合する。C# facadeを機械的に移植してC++仕様を確定させず、C++ DDの文書全体レビューを先に行う。

## 目的

承認済みRQ、Core/C ABI UI/Command契約、および利用者が全文承認したC++ wrapper DDに基づき、C++ consumerがAsync Command wrapperを利用し、初期UI treeとlayoutを構成してButtonへCommandを接続できるようにする。Command wrapperとUI Control facadeは独立してfocused/consumer検証できるunitに分ける。Coreのstatus、thread制約、handle ownership、callback lifetimeをC++ APIで変質させない。

本作業はOS固有描画peerの実装、UI表示成功、CMake package変更、0.0.1 release合格を意味しない。native peer未接続時の結果は共通C ABIの挙動を保つ。

## 承認済み根拠

- 要求: [RQ-03](../11_requirements/RQ-03_language-support.md)、[RQ-04](../11_requirements/RQ-04_abi-boundary.md)、[RQ-05](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06](../11_requirements/RQ-06_threading-dispatch.md)、[RQ-07](../11_requirements/RQ-07_binding.md)、[RQ-09](../11_requirements/RQ-09_ui-scope.md)。利用者可視の結果と例外条件は各RQ本文に従い、新しい外部保証を本計画・DD内で追加しない。
- Architecture: [ADR-0001 Rust Core](../04_approval/adr-0001-rust-core.md)。C++はRust Coreに直接依存せずversioned C ABIを使う。
- Basic design: [Runtime](../12_design/runtime-objects/runtime.md)、[UI Tree/Layout/Controls](../12_design/ui/tree-layout-controls.md)。
- Detailed design: [C++ wrapper](../13_detail_design/wrappers/cpp.md) v1.3、[共通C ABI](../13_detail_design/runtime-objects/abi.md) v1.3、[Async Command](../13_detail_design/events/commands.md)、[UI Controls](../13_detail_design/ui/controls.md) v1.1、[UI Tree/Layout](../13_detail_design/ui/tree-layout.md)、[Input/Focus](../13_detail_design/ui/input-focus.md)。C++ Async CommandとUI公開APIをDD v1.4案として具体化し、文書全体の利用者承認後に実装する。
- Coding rule: [C++ coding rule](../00_rule/coding_rule_C++.md)。現行ruleはGoogle C++ Style Guideの全面適用を指定する。
- Test: [wrapper test design](../32_unit_test/wrappers.md)、[unit-test strategy](../32_unit_test/unit-test-strategy.md)、[UI test design](../32_unit_test/ui.md)。C++向けfocused/consumer caseとunit mappingは、承認済みC++ DDと既承認test契約を照合し、source変更前に登録する。
- Prior implementation notes: [C++ Runtime wrapper](cpp-wrapper-runtime-api.md)、[C++ Binding wrapper](cpp-binding-wrapper.md)、[C++ Property wrapper](cpp-property-wrapper.md)。既存owner、cleanup supervisor、Result、callback handlerの契約を再利用し、別のcleanup経路を重複実装しない。実装開始前に各文書、source、unit manifestの差を照合し、実装済み範囲とunit stateを正しく記録する。

## 対象範囲案

- Window、共通Widget/Container、StackPanel、Text、TextBox、ButtonのC++17 facade。具体的な公開型名・method・constructorはDD案で示し、承認前に確定しない。
- Treeのcontent/children操作、orientation/alignment、Focus要求、初期ControlのProperty access。
- 既存C++ Property/Bindingとの値接続、および新しいC++ Async Command型とButton接続。異なるRuntime・値型・不正enum・native failureをC ABIへ渡す条件はC ABI契約に従う。
- Window peer open/closeとclose-request callback/defer/resolve。callback root、例外handler、borrowed request scope、deferred owned token、Subscription cancel/releaseの分担をDD案で明示する。
- fake C ABI focused testsと、実Core/C ABIを使う別process consumer test。UI描画peerのない環境で描画成功は主張しない。

## 対象外

- Rust Core/C ABI、承認済み要求、OS固有Window/Renderer/Input/IME/Accessibilityの意味変更。
- CMake package、配布header/package identity、install/export/RID/native assetの変更。
- reflection、自動ViewModel探索、新Control機能・新しい外部保証の追加。
- C#、Rust、Python wrapperへの変更、および他wrapper branchを親とすること。
- OS peer未接続をUI表示の成功とみなすこと、0.0.1 release/tag/publication判断。

## 実施順・承認境界

1. [x] 0.0.1計画と現行C++/C# wrapper状態を照合し、次のC++ UI wrapper候補を選定した。
2. [x] 計画0.1承認後のpreflightでC++ Command wrapper未実装を確認した。Button連携の前提としてAsync Command wrapperを独立unitに追加する。
3. [ ] 改訂計画0.2のAsync Command先行、unit分割、branch親子関係を利用者がレビューする。承認前は改訂をcommit/pushせず、DD案・source実装へ進まない。
4. [ ] 承認後、本改訂計画を`impl-cpp/ui-controls-wrapper`でcommit/pushし、Windows/Linux CIの文書gateを確認する。branch親は`impl-rust/core-ui-controls-c-abi`であり、C# wrapper branchを親にしない。
5. [ ] 実装状態を照合し、既存C++ Runtime/Property/Binding/Subscription source、tests、plan docs、`tools/units.yaml`の差を記録する。実装済みsourceを重複作成せず、unit stateを根拠に合わせる。
6. [ ] C++ wrapper DD v1.4案を作る。C++17 public types/signatures、C ABI mapping、move-only ownership/cleanup node、Runtime anchor、Async Command execute/state/cancel/completion/failure、Property/Binding連携、Control/Tree/Layout/Focus、ButtonのCommand接続、callback root/例外遮断、borrowed/deferred close request lifetime、status/error/threadを具体化する。class/call/data/ownership flow図は必要箇所に限る。
7. [ ] C++ wrapper DD v1.4全文を利用者レビューに提示し、承認前にAPIを確定扱い・実装しない。要求/承認済みC ABIと矛盾が見つかったら差異と影響先を記録し、上位文書レビューへ戻す。
8. [ ] 承認済みDDに基づきAsync CommandとUI Controlのfocused fake-ABI/real C ABI consumer case、およびunit dependency/source/command mappingをtest正本・`tools/units.yaml`へsource変更前に登録する。test文書の改訂は文書全体をレビュー・承認後に適用する。
9. [ ] C++ Async Command wrapperを独立unitとして実装・検証・commitする。focused testと必要なC ABI consumerを同一revisionへ含め、既存C++ APIの契約に従う。
10. [ ] C++ Window/Widget/Container/StackPanel/Text/TextBox/Button facadeを独立unitで実装・検証・commitし、ButtonとAsync Commandの接続を含める。
11. [ ] fake ABI focused tests、実C ABI consumer、既存C++ package consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書/manifest/diff checksを実行する。各実装SHAに対するWindows/Linux CIを確認し、失敗時は修正後の新SHAで確認する。
12. [ ] 実装revision/CI証跡を計画、test design、unit state、`.docs/01_memory/`に記録する。OS peer/package/release完了とは区別する。

## 完了条件案

- C++ wrapper DD v1.4全文が利用者承認済みで、Async Commandおよび公開Control型の所有/lifetime/error/thread動作が対応RQとC ABI契約に整合する。
- C++ consumerから初期Control tree、layout/Focus、Property/Binding/Command接続、Window close requestを利用できる。native peerの未対応結果はそのまま保持する。
- fake ABIと別process実Core/C ABI consumerのfocused testsが成功し、既存C++ package integrationに回帰がない。
- unit mappingが実装revisionに対応し、最終Windows/Linux CIで必要gateが成功する。
- backend/package/releaseが未実装の場合、それを完了として記録しない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-25 | 0.1 / C++ UI Control wrapper | 利用者承認 | 計画commit `ac3e7af`を`impl-cpp/ui-controls-wrapper`へpush。preflightでCommand wrapper未実装を確認したため、下記0.2改訂案で依存・unit順を修正 |
| 2026-10-25 | 0.2 / C++ Async Command + UI Control wrapper | 利用者レビュー待ち | C++ Async CommandをButton接続の先行unitとして追加。未commit・未実装。C++ DD v1.4全文承認をsource実装前条件とする |
