# Window背景色wrapper横断実装（Rust完了、C++/C#/Python）

- 計画版: 0.2
- 状態: 承認済み（利用者、2026-10-31。関連wrapper Todoをまとまりとして完了後にPRを発行する）
- 作成日: 2026-10-31
- 対象version: `0.0.1`
- 対象branch: Rust `impl-rust-wrapper/window-background-config`（完了）、C++ `impl-cpp-wrapper/window-background-config`（作業中）、後続C#/Pythonは各々`impl-rust/core-apis-window-background`から派生する。
- 前提: Core/C ABI Window background RGB API、RQ-09-074、各wrapper背景色設計は承認済み。C++ wrapperにはWindow safe type/生成APIがない。

## 背景

C++ DD 1.4には`Window::background_rgb()` / `set_background_rgb()` / `reset_background_rgb()`と`RgbColor`があるが、C++ wrapperにWindow型・Window生成経路はまだ存在しない。既存C ABIの`window_create`とbackground get/set/resetは利用できるため、承認済み背景色APIを利用可能にするにはC++ safe ownerを追加する必要がある。

C++ Runtime wrapperはmove-only owner、`RuntimeCleanupState`、事前確保PendingNode、`Result<T>`によるnon-throw error表現を持つ。Window ownerもこの既存寿命・失敗契約へ接続し、Core状態の二重cacheや例外依存を導入しない。

## 計画スコープ

1. Rust wrapper Window ownerとbackground RGB APIは実装・検証済み（`e66c57f`）。
2. C++ DDとUI unit-test設計を改訂・全文承認後、C++専用ブランチで`Window::create(Runtime&) -> Result<Window>`、背景色get/set/resetを実装する。move-only ownerとしてtoken、Runtime cleanup state、PendingNodeを保持する。
3. C++ `Window::close()`とdestructorはowner tokenのreleaseだけを行う。native Window peerのopen/close、title/content、Control tree、Layout、表示は対象外。
4. C++ fake ABI testsでdefault/set/reset、invalid input atomicity、C ABI error、move/close/release retryを確認し、real C ABI consumerでcreate/get/set/reset/releaseを確認する。
5. C# wrapperで同等のWindow safe owner・背景色API・focused/consumer testsを実装し、C# DD・test designとunit mappingを更新する。
6. Python wrapperで同等のWindow safe owner・背景色API・focused/consumer testsを実装し、Python DD・test designとunit mappingを更新する。
7. 各言語は最新Core APIブランチから独立して分岐する。全wrapperのTodoを消化後、対応するPRをまとめて発行する。PR完了・main反映を確認した後、不要な作業ブランチを削除する。
8. 実装と同一revisionでfocused/consumer tests、workspace gates、minimum Rust toolchainを実行し、`.docs/32_unit_test/implementation-test-inventory.md`、`tools/units.yaml`、`.docs/01_memory/`を更新する。
9. 各実装unitの対応テストが通った応答内で日本語commitし、対象言語ブランチへpushする。計画完了前のpushは許可するが、マージPRは計画全体完了後に作成する。

## 対象外

- C++ Window peer open/closeとOS上の表示確認。
- Platform peerとOS上の表示確認。
- Style/Theme/Resource、alpha/gradient、子Control背景。

## 承認境界

- 計画およびC++ Window safe owner追加は利用者承認済み。実装はC++ DDおよびunit-test designの改訂案が文書単位で全文承認された後に開始する。
- 既承認のRQ-09-074とC++ DD 1.4背景色契約は変更しない。
- C++/C#/Pythonの各wrapper設計文書と試験設計は、各言語の実装前に文書単位で承認を得る。
- 計画全体のTodo消化後に関連PRを発行する。PRマージ後は未マージ依存ブランチや別案件を除外して、完了済み枝を整理する。

## 実施順

1. [x] 計画0.2（関連wrapperをまとめ、全Todo消化後にPR）を承認。
2. [x] C++ DD 1.5とUI unit-test design 1.0を文書単位で改訂・全文承認。
3. [x] 文書承認後、C++-only branchでsafe Window ownerとRGB APIを実装する。
4. [x] C++ focused/consumer tests・同revision gatesを通し、unit ledger/memoryを更新する。
5. [ ] C# wrapperの設計承認・実装・同revision検証を完了する。
6. [ ] Python wrapperの設計承認・実装・同revision検証を完了する。
7. [ ] 計画全体の完了時に関連PRを発行し、レビュー・mainへのマージ完了を確認する。
8. [ ] main反映確認後、完了済み作業ブランチをローカル・リモートから整理する。

## C++実装unit完了記録（2026-10-31）

- `UNIT-WRAP-CPP-008`は実装・local検証完了。詳細は[実装unit台帳](../32_unit_test/implementation-test-inventory.md)を参照。
- C++ focused・package consumer・workspace check・minimum Rust checkはpass。Windows local環境でVS2022 BuildTools/MSVC 19.44.35224を使用。CIは未実施。
- 今回はC++ branchへcommit/pushする。C#/Python Todoが残るためPRはまだ作成しない。
