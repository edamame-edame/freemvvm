# Rust Core / C ABI 非同期Command基盤

- 更新日: 2026-10-24
- 状態: local実装・全gate pass。Windows/Linux CI確認待ち
- 対象: 承認済み非同期Command DDにあるCore object/state machineとversioned C ABI surfaceを実装し、Rust/C ABI testsで動作確認する
- 作業branch案: `impl-rust/runtime-async-command`

## 背景と範囲

非同期Command契約は[RQ-02-004](../11_requirements/RQ-02_core-features.md#rq-02-004)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)、[RQ-06-009](../11_requirements/RQ-06_threading-dispatch.md#rq-06-009)、[RQ-06-010](../11_requirements/RQ-06_threading-dispatch.md#rq-06-010)、[RQ-06-011](../11_requirements/RQ-06_threading-dispatch.md#rq-06-011)、[RQ-09-006](../11_requirements/RQ-09_ui-scope.md#rq-09-006)で承認済みであり、[イベントハンドリング：非同期Command](../13_detail_design/events/commands.md) v1.1と[共通C ABI](../13_detail_design/runtime-objects/abi.md) v1.2に実現契約・レビュー用API宣言がある。しかしCommand functionは配布header、`crates/c-abi`、`crates/core-runtime`にまだ存在しない。C# wrapperはこのAPIを前提にするため、先にnative基盤を実装する必要がある。

既存DDの意味論を変更せず、Core Command state/objectとC ABI bridgeを一単位で実装する。C# wrapper、Button/UI hookup、進捗専用API、NuGet/配布設定は対象外とする。C# wrapperは後続の別unitとして扱う。

## 根拠

- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreを動作の正本とし、versioned C ABIから各言語wrapperへ公開する。
- 基本設計: [イベントハンドリング：通知・入力・Command](../12_design/events/notifications-commands.md)、[オブジェクト管理：値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)、[ランタイム：実行context・所有・終了](../12_design/runtime-objects/runtime.md)。
- 詳細設計: [イベントハンドリング：非同期Command](../13_detail_design/events/commands.md) v1.1、[共通C ABI](../13_detail_design/runtime-objects/abi.md) v1.2のreview declaration。これらは設計根拠であり既存実装とは扱わない。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[requirements test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装案

- CoreにRuntime所有のCommand object、read-only state/can-execute Property、active operation record、非zero・Runtime内非再利用operation ID、thread-safe cancel token、immutable failure snapshotを追加する。
- operationの受付、RUNNING/CANCELLING/terminal遷移、既定再入拒否、重複/古いID拒否、failure clear、状態/can-executeの一貫したcommitを承認済みDDの順序どおり実装する。
- Command create/execute/cancel/close、state/can-execute Property取得、failure snapshot、cancel token query、success/failure/cancelled completionのC ABI関数をpublic headerとRust exportsへ追加する。固定幅型、out初期化、wrong-thread、runtime/type identity、handle ownership、error snapshotをABI DDに照合する。
- start/userdata/destroy closureはcreate成功後だけCoreが取得し、failure時は呼出側に残す。borrowed callback handleからworker用参照をretain可能にし、close/shutdownはworkerをjoinせずcancelを要求する。
- worker完了を直接worker threadからUI stateへ反映しない。Runtime post後のUI callbackでoperation IDを照合して一度だけ終端結果を適用する。post failureはclosureとretained handleを呼出側が回収できるようにする。
- C ABI宣言はレビュー用ではなく実装済み配布APIとなるため、ABI version/feature policy、header probes、symbol exports、status/error contractを関連DDと既存CIに沿って確認する。既存シンボル互換性を壊さない。

## 試験案

- 実装前に専用unit/test mappingを登録する。core testsでは、state/can-execute遷移、同一Command再入拒否、operation ID一意/枯渇時失敗、cancel要求とworker終端の分離、重複/過去完了、failure snapshot不変性、close後/shutdown後の抑止、closure destroy一回を検査する。
- C ABI testsではCommand create/execute/cancel/close、invalid/wrong-runtime/wrong-thread/status/out初期化、state/can-execute Property read-only、failure snapshot ownership、worker token retention、runtime_post completion、create/post failureのownershipを検査する。
- C/C++ header probesとABI symbol/consumer smokeで、宣言のC/C++互換性と配布libraryからのlinkを確認する。C# consumerはこの基盤unitでは実行せず、後続C# wrapper unitで実行する。
- `tools/dev.py check`、`tools/dev.py minimum`、document links、unit-manifest validation、`git diff --check`を同じ実装revisionで通し、Windows/Linux CIを確認する。実装結果を本計画・unit台帳・`.docs/01_memory/`に記録する。

## 作業順と承認境界

1. [ ] 利用者が基盤unitの範囲と試験境界をレビューする。
2. [ ] 計画承認後に計画をcommit/pushする。
3. [x] `UT-CORE-COMMAND-001` / `UT-ABI-COMMAND-001` / `UT-ABI-COMMAND-SMOKE-001`と`UNIT-CORE-066` mappingをsource変更前に登録した。
4. [x] 利用者の「ok」により計画と基盤実装の開始を承認された。Runtime共有operation ID issuerを含むCore state machineとfocused testsを実装し、7 testsがpass。
5. [x] C ABI exports/header・Command state Properties・read-only setter・C11 consumer smokeを実装。C ABI focused testsと`abi-command-smoke`がpass。
6. [x] `tools/dev.py check`、`tools/dev.py minimum`、`cargo test --workspace --all-targets --locked`、document/manifest checksがlocal pass。
7. [ ] implementation commitをpushし、Windows/Linux CIを確認する。
8. [ ] CI結果を本計画・unit台帳・memoryへ反映してunitを完了記録する。
9. [ ] このunit完了後、C# wrapper planへ戻り、C# test design/mappingとC# implementationを進める。

この計画案はC# wrapper計画から見つかった未実装依存を切り出したもの。計画承認とnative実装開始承認は別々に記録し、現時点では製品source変更・計画commitを行っていない。
