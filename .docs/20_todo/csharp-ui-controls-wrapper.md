# C# UI Control wrapper

- 計画版: 0.1
- 状態: 計画案（利用者レビュー待ち。未commit・未実装）
- 作成日: 2026-10-25
- 対象: 0.0.1初期UI Control Core/C ABIをC#から構成・操作するwrapper
- 作業branch案: `impl-csharp/ui-controls-wrapper`
- 親branch案: `impl-rust/core-ui-controls-c-abi`（Core APIs実装とC ABIが完了したrevision）

## 背景

[0.0.1 release計画](release-0.0.1.md)は、C++、C#、Rust、Pythonの各wrapperから同じUI縦断機能を利用可能にすることを目標とする。[初期UI Control C ABI Core実装](core-ui-controls-abi-implementation.md)は`UNIT-CORE-070..072`、`UNIT-ABI-020..021`まで完了し、最終revision `f87c082`のWindows/Linux CIが成功した。C# Runtime、Property、Binding、Command wrapperは既に実装済みだが、Window/Control/tree/layout/Focus/close requestをC#から利用するfacadeはまだない。

C# wrapper DD v1.4はRuntime/Property/Binding/Commandの所有・interop・例外境界を承認済みである。一方、今回追加するControl公開型/APIとC#からのownership/call flowは未定義なので、実装前にDDの差分提案と試験境界をレビューする。

## 目的

承認済み要求・C# wrapper DD・C ABI DDの範囲内で、C# consumerがWindow、StackPanel、Text、TextBox、Buttonを生成し、Property/Binding/Commandを接続し、Logical Treeと基本配置・Focusを操作できる初期Control facadeを実装する。C ABI status、thread/thread-affinity、token ownership、close requestの結果をC#層で変質させない。

この作業の完了は、OS固有描画peerの存在、実Window表示、NuGet配布、0.0.1 release合格を意味しない。native peer未接続時はC ABIのunsupported結果をそのままC#から確認できる形にする。

## 承認済み根拠

- 要求: [RQ-03 Language support](../11_requirements/RQ-03_language-support.md)、[RQ-04 ABI boundary](../11_requirements/RQ-04_abi-boundary.md)、[RQ-05 Ownership/callbacks](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06 Threading/dispatch](../11_requirements/RQ-06_threading-dispatch.md)、[RQ-07 Binding](../11_requirements/RQ-07_binding.md)、[RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)。対象はControlをC#から構成する範囲とし、各挙動はRQ本文に定められた結果を変えない。
- Architecture: [ADR-0001 Rust Core](../04_approval/adr-0001-rust-core.md)。C# facadeは共通versioned C ABIを利用し、Rust Coreへ直接依存しない。
- Basic design: [UI Tree/Layout/Controls](../12_design/ui/tree-layout-controls.md)、[Runtime](../12_design/runtime-objects/runtime.md)。設計文書の承認状態を着手前に照合する。
- Detailed design: [C# wrapper](../13_detail_design/wrappers/csharp.md)、[共通C ABI](../13_detail_design/runtime-objects/abi.md)、[UI Controls](../13_detail_design/ui/controls.md)、[UI Tree/Layout](../13_detail_design/ui/tree-layout.md)、[Input/Focus](../13_detail_design/ui/input-focus.md)。C# wrapperの新規公開API、callback/data/ownership flowはDD改訂案で明示し、全文承認前は確定仕様として扱わない。
- Unit test: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[UI test design](../32_unit_test/ui.md)。Core/C ABI側試験の結果をC# wrapperの試験合格へ読み替えない。

## 対象範囲

- C# typed facadeとC ABI Control symbolの対応。既存`UiRuntime`、`UiProperty<T>`、`UiBinding`、`UiCommand`との接続。
- Window、StackPanel、Text、TextBox、Buttonの生成・基本Property access・独立owner/Dispose。
- Window contentとContainer childrenの操作・child取得。入力不正・異Runtime・異種handle・失敗後の所有保全。
- StackPanel orientation/child alignment、Widget Focus要求。
- Buttonと既存`UiCommand`の接続・置換・取得時のowner管理。
- Window open/closeおよびclose-request callback/defer/resolveのC#公開surface。native peer未接続時のstatus保持。
- C# fake ABI focused testsと、実Core/C ABIを呼ぶ別process consumer test。C#製品例は依存APIの段階完了後に追加。

## 対象外

- Rust Core/C ABIの意味変更、OS固有Window/Renderer/Input/IME/Accessibility実装。
- C# NuGet RID native asset、loader、package identityや配布方法の変更。
- reflectionによるViewModel自動探索、新しいUI動作・要求保証・独自Dispatcher。
- 0.0.1 release/tag/publication判断、他言語wrapperへの変更。

## 実施順・承認境界

1. [x] Core/C ABI Control implementation plan完了と最終CIを確認。記録commit `9a19b95`、Core APIs branch `impl-rust/core-ui-controls-c-abi`。
2. [ ] 本計画案の対象範囲・branch親子関係・DD/test承認境界を利用者レビュー。
3. [ ] 計画承認後、計画文書をcommit/pushする。承認前はこの案をcommitしない。
4. [ ] C# wrapper DD v1.5の改訂案を作成する。公開型/API候補、C ABI mapping、typed ownership/Dispose、call lease、thread検査、callback rootと例外境界、Control/Command接続、主要class/call/data/ownership flowを必要箇所に追記する。Core側の既承認動作を繰り返し規定せず、RQ/DD差異が見つかれば別途記録する。
5. [ ] C# DD全文、特に公開surfaceとlifetime/callback契約について利用者承認を得る。実装承認は別に記録し、承認前にsourceを変更しない。
6. [ ] 実装開始承認後、wrapper test designへC# focused/consumer casesを登録し、`tools/units.yaml`へsource・design・requirements・dependencies・commandsを対応付ける。候補unitは`UNIT-WRAP-CS-CONTROLS-001`、`UNIT-WRAP-CS-WINDOW-001`とし、詳細設計と試験粒度に合わせて確定する。
7. [ ] `impl-rust/core-ui-controls-c-abi`の最新CI成功revisionから`impl-csharp/ui-controls-wrapper`を作成する。別言語branchを親にしない。C# coding ruleの実在と適用範囲を再確認する。
8. [ ] Control/tree/property/layout/Focus/Command facadeをunit別に実装し、各unitのfocused testと実C ABI consumer試験を同revisionに含めてcommit/pushする。
9. [ ] Window close callback/defer/resolveのwrapper lifetimeとexception境界をfocused/fake ABIおよび利用可能なC ABI consumerで検証する。
10. [ ] `tools/dev.py check`、`tools/dev.py minimum`、該当C# focused/consumer試験、文書リンク、unit manifest、`git diff --check`を実行し、実装commitのSHAに対するWindows/Linux CIを確認する。失敗時は原因修正後の新SHAで再確認する。
11. [ ] 計画・DD・test design・unit mapping・`.docs/01_memory/`へ完了結果とCIを記録する。C# Control wrapperの完了をOS peer、package、release完成とは分離する。

## 完了条件

- C#公開APIとownership/callback/error/thread動作が承認済みDD・C ABI契約に一致する。
- C#から初期UI Control treeを構成し、Property/Binding/Command接続とlayout/Focus要求をC ABI経由で行える。
- Window close requestのcallback・defer・resolve・cancel/Disposeが一度だけのcleanupとstale request拒否を保つ。
- wrapper focused tests、別process C ABI consumer、必要なC# build gates、Windows/Linux CIが同一の最終実装revisionで成功する。
- backendやpackageが未実装の場合、それらの未対応範囲を完了扱いしない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-25 | 0.1 / C# UI Control wrapper | 利用者レビュー待ち | 次のwrapper層に向けた計画案。未commit・未実装 |
