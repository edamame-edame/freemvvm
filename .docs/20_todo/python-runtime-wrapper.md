# Python Runtime wrapper

- 計画版: 0.1
- 状態: 計画承認済み（利用者、「ok」、2026-10-25）。DD/test designと実装開始は別承認ゲート
- 作成日: 2026-10-25
- 対象: 0.0.1に向けたPython 3 `ctypes` Runtime基礎facadeの設計・試験・実装
- 作業branch案: `impl-python/runtime-wrapper`
- 親branch案: `impl-rust/core-ui-controls-c-abi`（version/0.0.1配下のCore APIs branch。Python wrapperはRust/C++/C# wrapperを親にしない）

## 背景と現状

0.0.1計画はC++、C#、Rust、Pythonから同じUI縦断機能を利用可能にすることを目標とする。RustのUI Control/Window close-request wrapper unitは完了し、branch `impl-rust/ui-controls-wrapper`のcommit `c6ed2d4`、Windows/Linux CI run `40187756235`まで記録済みである。C++とC#のUI Control実装は別々のsibling branchにある。

Python packageは開発環境・wheel骨格までで、公開Runtime/UI facadeは未実装である。Python wrapper用のimplementation unitは`tools/units.yaml`にplanned登録済み。Python DD v1.2は`ctypes`、opaque owner、cleanup、thread/GIL、shutdownとRuntime公開APIを定める。今回のunitはRuntime基礎に限定し、Property/UI facadeを含めない。

根拠: [0.0.1計画](release-0.0.1.md)、[Python wrapper DD](../13_detail_design/wrappers/python.md)、[Python要求](../11_requirements/RQ-03_language-support.md)、[ownership/callback要求](../11_requirements/RQ-05_ownership-callbacks.md)、[thread要求](../11_requirements/RQ-06_threading-dispatch.md)、[Runtime詳細設計](../13_detail_design/runtime-objects/runtime.md)、[共通C ABI詳細設計](../13_detail_design/runtime-objects/abi.md)。

## 目的

Python consumerが承認済みC ABIからRuntimeを作成し、UI-thread context・Runtime identity・shutdown・error結果を安全なPython APIとして扱える最小基礎を作る。その後のProperty/Binding/Command/UI Controlsは本unitへ混ぜず、Runtime基礎の設計と検証後に別unitとして提案する。

## 今回の範囲案

- 対象: Python公開Runtime owner/API、library loading seam、C ABI fixed-width declarationsの最小範囲、UiContext相当のRuntime/thread検査、明示shutdown/close、例外時の後始末、fake ABI focused test、別consumerでの実C ABI smoke。
- implementation unit: `UNIT-WRAP-PY-RUNTIME-001`。設計/test文書承認済みとして`tools/units.yaml`へ登録し、source/test commandを実装に合わせて設定した。
- 検証案: Python 3.13でfake ABIによる成功/失敗・wrong-thread・二重close/status mappingを確認し、独立consumerから実C ABIのRuntime create/shutdownを実行する。UI event loopがないconsumerで未保証の完了をassertしない。
- 対象外: Property/Binding/Command、Window/Control/tree/layout/Focus、close-request callback、Python analyzer、native library package/loader配布、PyPI upload、release/tag、WebEngine branch。

## 実施順・承認境界

1. [x] 現状とbranch境界を調査した。Pythonはpackage骨格のみで、Python wrapper implementation unitは未登録。最新の共通Core APIs branchを親とし、Rust/C++/C# wrapperを親にしない。
2. [x] Python DD v1.2改訂案として公開Runtime surfaceを提案した。type/method signature、Runtimeとcontextのowner、close/shutdown分離、wrong-thread/closed error mapping、private fake API seam、explicit cleanupとfinalizer境界を示し、Python DD文書全体の改訂案として利用者レビューに出す。承認済みPython所有/cleanup契約を独自に変更しない。
3. [x] fake ABIと実C ABI consumerの前提・操作・観測・期待値・後始末をunit-test文書0.4改訂案へ提案した。DD全文承認とtest design全文承認を別々に確認する。
4. [x] DD v1.2とtest design v0.4の全文承認を記録し、異Runtime context検査は利用側APIが定まる後続UI unitへ送る範囲変更を反映した。`UNIT-WRAP-PY-RUNTIME-001`を`tools/units.yaml`へ登録した。
5. [x] `version/0.0.1 -> impl-rust/core-ui-controls-c-abi -> impl-python/runtime-wrapper`のbranch関係を確認し、最新Core APIs revisionからPython branchを切った。Rust/C++/C# wrapper siblingの差分を混ぜていない。
6. [x] Runtime facade、focused fake ABI tests、独立consumer、検証commandを同じ実装unitで実装した。focused tests 9件、Windows実C ABI consumer、`tools/dev.py check`、`minimum`、Windows/Linux CI run 40190078806が実装commit `bdaf48e`で成功。
7. [ ] Property/Binding/CommandとUI Controlsへは、このunitを基礎とした別の小さな計画・設計・test reviewを作成してから進む。

## 完了条件案

- Python DDとunit-test設計が文書全体で承認され、公開Runtime API・error・ownership・thread境界が確定している。
- fake ABI testがRuntime作成失敗、wrong-thread/context使用、close失敗時のowner保全、shutdown/close分離を確認する。異Runtime context拒否は後続のcontext消費APIを持つunitで確認する。
- 独立Python consumerが実C ABI libraryへ接続し、Runtime lifecycleを確認できる。
- 対応unit、`tools/dev.py check`、`minimum`、Windows/Linux CIが同一実装revisionで成功する。
- まだ未実装のProperty/UI/package/release機能を完了扱いしない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-25 | 0.1 / Python Runtime wrapper | 利用者レビュー待ち | Python package骨格と承認済みPython DDを調査して作成。計画承認・DD/test承認・実装開始承認・release判断を意味しない |
| 2026-10-25 | 0.1 / Python Runtime wrapper | 利用者承認 | 「ok」により範囲案、設計/testレビュー手順、`impl-python/runtime-wrapper` branch関係を承認。DD/test全文承認と実装開始は別判断 |


## 実装・検証状況

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-25 | UNIT-WRAP-PY-RUNTIME-001 | 実装・検証完了 | commit `bdaf48e`。9 focused tests、実C ABI consumer、`tools/dev.py check`、`minimum`成功。[Windows/Linux CI run 40190078806](https://github.com/edamame-edame/freemvvm/actions/runs/40190078806) success |
