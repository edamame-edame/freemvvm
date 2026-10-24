# C# Runtime wrapper基盤

- 更新日: 2026-10-24
- 状態: 完了（実装commit `0fa8cc7`、Windows/Linux CI成功、2026-10-24）
- 対象: 承認済みC# wrapper DD v1.1に従い、C#からRuntime作成・UI thread確認・終了要求・明示破棄を安全に利用する最初の製品wrapper unit。

## 背景と範囲

現在の`wrappers/csharp/FreeMvvm`はnet10.0/C#14のbuild-only skeletonで、製品facade、P/Invoke宣言、native load、C#用unit-test harnessはない。承認済みC# wrapper DD v1.1は`UiRuntime`、fixed-width ABI mapping、`UiResult<T>`/`UiException`、`TokenOwner`/`CallLease`、release失敗時の`CleanupTicket`を定めている。本計画はこのうちRuntime所有基盤とRuntimeの最小操作を一つの動作検証可能な単位として実装する。

Property、Binding、Subscription、Command、callback trampoline、NuGet RID package配布は後続unitへ分ける。C# Runtime用native loadは承認済み設計のprocess-lifetime保持に従うが、新たな配布形式・外部保証は追加しない。

## 承認済み根拠

- 要求: [RQ-03-002](../11_requirements/RQ-03_language-support.md#rq-03-002)、[RQ-03-006](../11_requirements/RQ-03_language-support.md#rq-03-006)、[RQ-03-013](../11_requirements/RQ-03_language-support.md#rq-03-013)、[RQ-03-014](../11_requirements/RQ-03_language-support.md#rq-03-014)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)、[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)、[RQ-05-010](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-010)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)、[RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreとversioned C ABIを動作の正本とする。
- 詳細設計: [C# wrapper DD v1.1](../13_detail_design/wrappers/csharp.md)「公開API・call lease・native load」「Dispose/CallLeaseの線形化」「型の責務と所有」。
- 試験: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)のUT-WRAP-CS-001、[TC-WRAP-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)、[release test規約](../30_release_test/release-test.md)、[version test規約](../31_version_test/version-test.md)。

## 実装提案

- C#14/net10.0の内部interop層を作り、C ABIの`ui_handle`を`ulong`、`ui_status`をraw値保持型、`ui_bool`を`byte`として扱う。native tokenをpointerや`IntPtr`へ変換しない。
- `UiRuntime`と基底の`UiResult<T>`/`UiException`/`UiErrorSnapshot`を実装し、Runtime create、UI thread判定、同期・非同期shutdownのstatus差、明示`TryClose`/`Dispose`を提供する。`Dispose`は冪等でshutdownを暗黙に開始しない。
- owned tokenは`TokenOwner`が保持し、各native callは短命`CallLease`を取得する。Dispose開始後の新規leaseを止め、既存lease中のtokenを無効化しない。release失敗ではtokenを失わずcleanup ticketへ移し、再試行可能にする。
- unknown status/domain/codeとerror detailなしの失敗でraw statusおよび利用可能なsnapshotを保つ。未定義statusをmessage文字列で分類しない。
- native library resolverは承認済みC# DDに従ってprocess lifetimeに保持する。RID別NuGet資産の作成、single-file publish拡張、ARM profile追加は本unitの対象外。

## 試験計画

実装開始前に[wrapper test design](../32_unit_test/wrappers.md)へC# Runtime専用caseを追加し、`tools/units.yaml`へ`UNIT-WRAP-CS-001`と対応test commandを登録する。既存UT-WRAP-CS-001のCallLease/Dispose競合とrelease failure保全の観点を再利用し、C#製品facadeの実装単位を識別できるtest IDを採番する。

- fake native dispatchを使うfocused testで、C ABI幅、status/error変換、Runtime create/query/shutdown、unknown status、失敗時のowner保持を確認する。
- deterministic concurrency testで、native call lease中のDispose、Dispose後の新規call拒否、最後のlease後release、release失敗ticketの保持と再試行、一度だけの解放を確認する。
- 別processのC# consumerで実C ABIへ接続し、Runtime作成、UI thread判定、UI loop bridge未登録時のshutdown拒否（`UI_UNSUPPORTED`）のstatus保持、明示Disposeを確認する。shutdown receiptの`UI_PENDING`とHostUnavailable照会は、mode/stateを制御できるfake ABI focused testで確認する。実C ABIでは未登録loopを受付済みとして誤認しない。
- focused testとconsumer testをWindows/Linuxで実行し、C# build、既存全体gate、Rust minimum gate、文書検査を同じ実装revisionで確認する。test reportとSHAをplan・unit台帳・memoryへ記録する。

## 実施順

1. [x] 利用者が本計画の対象・API範囲・試験範囲をレビューし承認した（2026-10-24）。
2. [x] 承認された計画をcommit `c635470`で記録し、CSharp branchへpushした。
3. [x] 実装開始の承認後、C# Runtime向けUT設計と`UNIT-WRAP-CS-001` mappingをsource変更前に登録した。
4. [x] C# Runtime ownership/interop基盤とfocused testsを実装した（`0fa8cc7`）。
5. [x] C# consumerを追加し、実C ABIとの最小Runtime縦断を確認した（UI loop bridge未登録時の`UI_UNSUPPORTED`を確認）。
6. [x] local gatesとWindows/Linux CIが実装commit `0fa8cc7`で成功した。unit台帳・本計画・`.docs/01_memory/`を完了更新した。

## 検証結果

- 実装commit: `0fa8cc73cc18cf8eba2cf9bcffc07b809fcda20a`（`C# Runtime wrapper基盤を実装`）。
- ローカル: `tools/dev.py csharp-runtime-test`、`tools/dev.py csharp-runtime-consumer-test`、Windows `tools/dev.py check`、`tools/dev.py minimum`、`tools/check_docs.py`が成功。`git diff --check`も成功。
- GitHub Actions: [Development bootstrap run 39994623749](https://github.com/edamame-edame/freemvvm/actions/runs/39994623749)でWindows/Linux両job成功。
- C#言語別coding ruleは未作成のため、当該unitではC#14/.NET 10 project設定、nullable、warnings-as-errors、承認済みwrapper DDを適用した。

## 未決定の実装前確認

- `LibraryImport` source generationとinternal native dispatchの分離方法は、承認済みC# DDに従いつつ、fake ABI試験とAOT/trim設定への影響を確認して決める。新しい公開API契約は追加しない。
- `CleanupTicket`の再試行起点・safe pointとの接続は既存Core/C ABI契約を確認し、release failure時にtoken ownershipを失わない方式を選ぶ。仕様のない成功扱いやprocess終了一般のcleanup保証は追加しない。
- AGENTS.mdが参照する`.docs/00_rule/coding_rule_CSharp.md`は現在存在しない。実装開始前にリポジトリの実在ルールを再確認し、存在しない場合は既存のC#14/.NET analyzer設定と承認済みDDを規約根拠にする。
- 実native consumer試験で、UI loop bridge未登録の`runtime_request_shutdown`は`UI_UNSUPPORTED`を返すことを確認した（2026-10-24）。receipt付きpendingのtest oracleはfake ABIに限定する。

## 計画承認と実装承認

本計画草案の承認と実装開始承認を別々に記録する。草案の作成だけではAPI案の承認、unit設計承認、実装開始承認を意味しない。

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | 本計画と実装範囲 | 利用者 | 「ok」により計画を承認。実装開始は別途承認待ち |
| 2026-10-24 | 本計画に沿う実装開始 | 利用者 | 「始めてください」により実装を承認 |
