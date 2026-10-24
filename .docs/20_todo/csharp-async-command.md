# C# 非同期Command wrapper

- 更新日: 2026-10-24
- 状態: 完了。計画・C# wrapper DD v1.4全文は利用者承認済み。C# Command実装・focused/consumer試験・local gates・Windows/Linux CIが成功
- 対象: 承認済み非同期Command C ABIをC#から利用し、実行可否・実行状態・協調キャンセル・完了/失敗報告を扱うwrapper unit
- 作業ブランチ: `impl-csharp/runtime-async-command`（`impl-rust/runtime-async-command`を親とする）

## 背景と範囲

C# Runtime、全5種Property、notification Subscription、Property Bindingは実装・CI確認済み。共通C ABIとイベントハンドリングDDには非同期Commandの生成、実行、cancel、close、state/can-execute Property、failure snapshot、workerからの完了報告が定義されている。Rust Core/C ABIのCommand実装は`UNIT-CORE-066`として完了し、CI run 40072910815もWindows/Linuxで成功した。C# wrapper DD v1.4は公開API、callback root、worker ownership、post、dispose/closeと結果表現を定め、利用者承認済み。

既存の承認済みC ABIを使うC# facadeと試験を実装する。Rust Core、C ABI意味論、進捗専用API、package配布、Button/UI integrationは対象外とする。

## 承認済み根拠

- 要求: [RQ-02-004](../11_requirements/RQ-02_core-features.md#rq-02-004)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)、[RQ-06-009](../11_requirements/RQ-06_threading-dispatch.md#rq-06-009)、[RQ-06-010](../11_requirements/RQ-06_threading-dispatch.md#rq-06-010)、[RQ-06-011](../11_requirements/RQ-06_threading-dispatch.md#rq-06-011)、[RQ-09-006](../11_requirements/RQ-09_ui-scope.md#rq-09-006)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。C# wrapperはRust Coreのversioned C ABIを利用する。
- 詳細設計: [イベントハンドリング：非同期Command](../13_detail_design/events/commands.md)、[共通C ABI](../13_detail_design/runtime-objects/abi.md)のCommand API、[C# wrapper DD v1.3](../13_detail_design/wrappers/csharp.md)。Commandのworker結果、state、operation ID、cancel/close、遅延報告の正本は承認済み共通DDに従う。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[requirements test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装提案

- C# wrapper DD改訂案でCommand facade、実行context/cancellation token、state/can-execute Property view、failure snapshot、close/dispose、callback例外境界、worker完了をUI threadへ戻す方法を具体化する。公開型名・戻り値・async delegate shapeは承認前の提案として記述し、共通DDにない保証を追加しない。
- `command_create_async`のstart callbackはUI thread上でworkerを速やかに開始して戻る。managed exceptionはABI境界へ出さず、承認済みC# callback failure handler規則との整合を示す。
- workerはCommand/cancel token等、非同期利用に必要なnative参照をstart callback中にretainし、終了時に解放する。完了報告はRuntime postでUI threadへ戻し、post拒否・Runtime close時もclosure/root/tokenを一度だけ回収する。
- `execute`受付と業務結果を別に表現し、operation IDで現在実行だけを更新する。cancel要求と実際のキャンセル終端を混同しない。Command handle releaseと`TryClose`を分ける。
- TextBox/Buttonへの接続、進捗の専用API、Core/ABI変更、NuGet RID package作成は含めない。

## 試験計画案

- DD承認後、wrapper test designへfocused fake ABI case `UT-WRAP-CS-COMMAND-001`と実C ABI consumer case `UT-CONSUMER-CS-COMMAND-001`を登録し、`tools/units.yaml`へunit mappingをsource変更前に追加する。
- fake ABIでcreate成功/失敗時のclosure ownership、execute受付/不可、operation ID、callback throw、cancel token照会、完了のpost、close/cancel/release、失敗snapshot所有、close後の遅延報告、callback rootとtoken cleanupを検査する。
- 実C ABI consumerでCommandを生成し、state/can-execute Property、開始とUI非占有、成功/失敗/cancel終端、重複/古いoperation報告、close/dispose境界を制御可能なhost loopで確認する。worker完了の同期にはdispatch turnを用いる。
- focused/consumer試験、C# format/build、`tools/dev.py check`、`tools/dev.py minimum`、文書リンク、manifest parse、`git diff --check`を同じ実装revisionで実行し、Windows/Linux CIと結果を本計画・unit台帳・`.docs/01_memory/`へ記録する。

## 実施順と承認境界

1. [x] 計画案のunit範囲・試験境界をレビューし、承認済み。
2. [x] 計画をcommit/push済み。
3. [x] C# wrapper DD v1.4を改訂し、クラス図・call/data/ownership flowを含めて文書全体の承認済み。
4. [x] 利用者の実装開始指示を受領。Rust Core/C ABI `UNIT-CORE-066`の実装とWindows/Linux CIが完了。
5. [x] C# test design v0.3と`UNIT-WRAP-CS-COMMAND-001` mappingを登録。
6. [x] C# Command facade、fake ABI focused tests、real C ABI consumerを実装。両Command試験はlocal pass。
7. [x] `tools/dev.py check`・`minimum`・Windows/Linux CI run 40074886918を通し、結果を記録して実装unitをcommit/pushした。

計画承認、DD全文承認、実装開始承認は別々に記録する。計画・DD・実装開始・試験設計登録・C#実装・focused/consumer試験・repository gates・Windows/Linux CIが完了し、実装unitと検証記録をcommit/push済み。
