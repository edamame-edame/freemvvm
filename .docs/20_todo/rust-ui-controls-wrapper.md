# Rust UI Control wrapper

- 計画版: 0.1
- 状態: 計画・Rust wrapper DD v1.7・unit-test設計 v0.4・実装開始承認済み（利用者、2026-10-25）。UNIT-WRAP-RS-UI-CTRL-001実装中。focused fake ABI / C ABI consumerはlocal pass、全体gate/CI未確認
- 作成日: 2026-10-25
- 対象: 0.0.1初期UI Control Core/C ABIをRust safe wrapperから構成・操作する
- 作業branch案: `impl-rust/ui-controls-wrapper`
- 親branch案: `impl-rust/core-ui-controls-c-abi`（Core APIs/C ABIの最新CI成功revision `dfb86cf`）

## 背景

[0.0.1計画](release-0.0.1.md)ではC++、C#、Rust、Pythonの各wrapperから共通UI縦断機能を使う。C++初期UI Control wrapperは[`impl-cpp/ui-controls-wrapper`](cpp-ui-controls-wrapper.md)の`UNIT-WRAP-CPP-009`として実装され、CI run [40183464855](https://github.com/edamame-edame/freemvvm/actions/runs/40183464855)が成功した。C# UI ControlとWindow close-request wrapperも別ブランチの`UNIT-WRAP-CS-CONTROLS-001` / `UNIT-WRAP-CS-WINDOW-001`として実装され、CI run [40169552932](https://github.com/edamame-edame/freemvvm/actions/runs/40169552932)が成功している。

Rust safe wrapperにはRuntime、Property、Subscription、Binding、Async Commandがあるが、Window、Control、Logical Tree、layout、Focus、close-requestの公開APIはまだない。Core/C ABIの初期UI Controls実装計画と実装は`impl-rust/core-ui-controls-c-abi`で完了し、そのbranchのCI run [40164472118](https://github.com/edamame-edame/freemvvm/actions/runs/40164472118)が成功している。

承認済みRust wrapper DD v1.6は既存safe API・所有型・callback/panic境界・Binding・Commandを定めるが、UI Control safe APIとそのRust固有のborrow/lifetime表現はまだ定義しない。C++/C# APIをそのままRustへ移植せず、まずRust wrapper DDの改訂案とfocused/consumer試験案をレビューする。

## 目的

承認済み要求と共通C ABIをRust safe APIから利用できるようにする。Rust UI-affine ownerを`Send`/`Sync`にせず、`UiContext`、`Owned`/`Borrowed`、cleanup ticket、Runtime identity、callback panic containmentなど既存のRust wrapper契約を再利用する。各Control操作の意味・status・owner lifetimeをC ABIから変質させない。

この計画はRust公開型/API案、DD、test designを全文レビューし、個別に承認された後に実装へ進む。計画案への承認だけではDD承認・実装承認・release合格・公開許可を意味しない。

## 承認済み根拠

- 要求: [RQ-03 Language support](../11_requirements/RQ-03_language-support.md)、[RQ-04 ABI boundary](../11_requirements/RQ-04_abi-boundary.md)、[RQ-05 Ownership/callbacks](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06 Threading/dispatch](../11_requirements/RQ-06_threading-dispatch.md)、[RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)、[RQ-10 Accessibility](../11_requirements/RQ-10_accessibility.md)、[RQ-12 Build/distribution](../11_requirements/RQ-12_build-distribution.md)。既存RQの結果と例外を変えず、新しい外部保証を計画内で追加しない。
- Architecture: [ADR-0001 Rust Core](../04_approval/adr-0001-rust-core.md)。Rust safe wrapperは共通versioned C ABIを呼び、別Coreを内包しない。
- Basic design: [UI Tree/Layout/Controls](../12_design/ui/tree-layout-controls.md)、[Runtime](../12_design/runtime-objects/runtime.md)。対応RQとの整合を実装開始前に確認する。
- Detailed design: 承認済み[Rust wrapper DD v1.6](../13_detail_design/wrappers/rust.md)、[共通C ABI](../13_detail_design/runtime-objects/abi.md)の初期UI Control/Window close-request章、[UI Controls](../13_detail_design/ui/controls.md)、[UI Tree/Layout](../13_detail_design/ui/tree-layout.md)、[Input/Focus](../13_detail_design/ui/input-focus.md)。Rust-specific API、borrow/lifetime、owner、callback、call flowはRust wrapper DD改訂案で具体化し、全文承認前は確定仕様として扱わない。
- Coding rules: [Rust coding rule](../00_rule/coding_rule_Rust.md)。既存crateのEdition・lint・unsafe規則もsourceから確認する。
- Test: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper tests](../32_unit_test/wrappers.md)、[UI tests](../32_unit_test/ui.md)。Rust focused fake ABIと実C ABI consumerを分け、C++/C#のテスト結果をRust wrapper試験へ読み替えない。
- Prior implementation notes: [Rust UI wrapper preflight](../01_memory/rust-ui-controls-wrapper.md)、[Rust Runtime](rust-runtime-wrapper.md)、[Rust Property](rust-property-wrapper.md)、[Rust Subscription](rust-property-subscription.md)、[Rust Binding](rust-property-binding.md)、[Rust Async Command](rust-async-command-wrapper.md)。既存のticket/owner/callback基盤を再利用し、別cleanup registryを追加しない。

## 対象範囲案

- Rust safe wrapperによるWindow、generic Widget/Container、StackPanel、Text、TextBox、Buttonの生成・操作。
- Window content、container children、orientation/alignment、Focus、Control Property access、Buttonと既存Async Commandの接続。
- Window peer open/close、およびclose-requestのborrowed callback view、deferしたowned request、resolve、Subscription cancel/release。
- Rust API固有のborrow/lifetime/typed ownership・UI-context検証・callback panic containmentをDD改訂案で定義する。具体的な型名、method signature、generic/enum表現、borrow期間はレビュー前に確定しない。
- fake ABI focused tests、実C ABIを呼ぶRust integration/consumer tests、および既存Rust crate/build gates。

## 対象外

- Rust Core/C ABIの動作変更、RQ/BD変更、OS固有Window/Renderer/Input/IME/Accessibility backend。
- `Send`/`Sync`制約の緩和、独自executor/thread pool、reflection、自動ViewModel探索。
- Cargo package公開、native artifact loader/fetch、ABI/profile選択、release/tag/publication。
- C++/C# wrapper branchを親にすること、他wrapper実装の同時取り込み。

## 実施順・承認境界

1. [x] 現行状態を確認した。Core UI Control C ABI、C# UI Controls、C++ UI Controlsはそれぞれ独立branchで実装・CI完了。Rust crateにはUI Control surfaceがない。
2. [x] 本計画案の範囲、unit分割、Core APIsを親とするbranch関係を利用者がレビューし、「ok」で承認した。
3. [x] 計画を`version/0.0.1`へcommit/push (`956a447`) し、Rust wrapper branchへ同commitをcherry-pick (`60139e7`) した。実装branch `impl-rust/ui-controls-wrapper` はCore APIs CI成功revision `dfb86cf`から作成した。
4. [x] Rust wrapper DD v1.7の改訂案を作成した。型・method/API surface、child/content ownership、UiContext/Runtime identity、cleanup ticket ownership、Button/Command参照、callback scope/defer/resolve、panic/status境界を提案し、未承認案として区別した。
5. [x] Rust wrapper DD v1.7全文は利用者の「ok」で承認済み（2026-10-25）。wrapper sourceは未変更。
6. [x] approved DDに基づくfake ABI focused / linked C ABI consumer試験案をtest正本へ追加した（`.docs/32_unit_test/wrappers.md` 0.4案）。
7. [x] Rust向けtest設計0.4案の全文は利用者の「ok」で承認済み（2026-10-25）。試験設計承認と実装開始承認は別にする。
8. [x] 実装開始承認後、`tools/units.yaml`へ`UNIT-WRAP-RS-UI-CTRL-001`、依存Core/UI Command、要求・設計・test IDs/commands/source pathsを登録した。testとimplementationは同一unit revisionで検証する。
9. [x] `impl-rust/core-ui-controls-c-abi`のCI成功revision `dfb86cf`から`impl-rust/ui-controls-wrapper`を計画承認直後に作成し、計画コミットを載せた（レビュー文書用。wrapper source実装はDD・test・実装開始承認後）。C++/C# wrapper branchは親にしていない。
10. [ ] Rust UI Control/Tree/Property/Layout/Focus/Command facadeと、Window close-request lifetimeを、レビュー済みDDと単独実行可能なunit分割に従って実装・commit/pushする。Control/Tree/Layout/Property/Command API、unit fake test、consumer exampleは実装済み。
11. [ ] focused fake ABI / linked C ABI consumer (`rust-wrapper-ui-controls-test`)、`tools/dev.py check`、`tools/dev.py minimum`、document links、format、diff checksはlocal pass。実装SHAのWindows/Linux CIを確認し、失敗時は修正後SHAで再確認する。
12. [ ] 計画・DD・test design・unit state・`.docs/01_memory/`へ実装SHA、実行結果、CIを記録する。peer/backend、Cargo package配布、0.0.1 release判断とは分離する。

## 完了条件案

- Rust wrapper DDとtest designが全文承認済みで、safe API surfaceが承認要求/C ABI契約に一致する。
- Rust consumerが初期Control tree、Property/Binding/Command接続、layout/Focus、Window close requestをUI context下で操作できる。
- move-only owner、borrow scope、Drop cleanup ticket、callback destroy、panic/status/thread/Runtime identity境界をfocused fake ABI testsで確認する。
- 実C ABI consumerとcrate/build gates、unit対応、最終Windows/Linux CIが同一実装revisionで成功する。
- native peer、package artifact、releaseが未実装の場合、それらを完了扱いしない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-25 | 0.1 / Rust UI Control wrapper | 利用者レビュー待ち | 0.0.1計画および現行Core/C ABI、C++/C# wrapperの実装状態から次候補を整理。未commit・未実装 |
| 2026-10-25 | 0.1 / Rust UI Control wrapper | 利用者承認 | 「ok」により範囲・unit分割・branch親子関係を承認。計画commit `956a447`をpush。DD/test全文承認と実装承認は別ゲート |
| 2026-10-25 | Rust wrapper DD v1.7 | 利用者承認 | 「ok」によりUI Control safe API設計全文を承認。source実装・test設計承認・実装開始承認とは別ゲート |
| 2026-10-25 | Rust wrapper test design v0.4 | 利用者承認 | 「ok」によりfocused fake ABI/consumer試験とunit分割案を含む全文を承認。試験未実装・未実行、実装開始承認は未取得 |
| 2026-10-25 | Rust UI Control実装開始 | 利用者承認 | 「承認します、実装を進めてください」により承認済みDD/test designに従う実装開始を承認。Window close-request unitは後続の独立unitとして扱う |
