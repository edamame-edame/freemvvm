# Rust Async Command wrapper

- 更新日: 2026-10-24
- 状態: Rust DD v1.6とtest design v0.9に基づく実装・local gates完了。Windows/Linux CI待ち
- 対象ブランチ: `impl-rust/wrappers-runtime-api`（親: `impl-rust/runtime-async-command`）
- 対象unit: `UNIT-WRAP-RS-CMD-001`（unit台帳登録は計画承認後）

## 目的

承認済み非同期Command Core/C ABIをRust safe APIから利用可能にする。Rust側はexecutor/thread poolを持たず、start callbackが受け取るworker operation contextをアプリ側のworkerへ渡す。workerのcancel照会と終端報告はCore/C ABIのthread・lifetime境界に従う。

## 根拠

- 要求: [RQ-02-004](../11_requirements/RQ-02_core-features.md#rq-02-004)、[RQ-05 ownership](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06 threading/Command](../11_requirements/RQ-06_threading-dispatch.md)
- 詳細設計: [Rust wrapper DD v1.6案](../13_detail_design/wrappers/rust.md)、[Async Command](../13_detail_design/events/commands.md)、[C ABI](../13_detail_design/runtime-objects/abi.md) §15
- 試験: [wrapper test design v0.9案](../32_unit_test/wrappers.md)、[test matrix](../30_release_test/requirement-test-matrix.md)
- 既存実装: Core/C ABI Commandは完了済みの[UNIT-CORE-066](../../tools/units.yaml#L2059)と[Command C ABI header](../../crates/c-abi/include/freemvvm/abi.h)。C ABI非同期Commandの以前の計画・実装記録は[abi-async-command](./abi-async-command.md)。

## スコープ案

- `AsyncCommand` create / execute / cancel / close。
- start closureの登録所有、panic containment、callbackで渡すSend worker operation context。
- worker-safe cancellation queryと`runtime_post`経由のsuccess/failure/cancelled報告。
- state / can-execute read-only Property取得、immutable failure snapshot。
- `OperationId`の非zero/raw mappingと未知state値の保存。
- focused/native integration testsおよび別crate public API compile test。

対象外: executor/thread pool、Future return API、progress専用API、UI Control/Button hookup、Core/C ABI変更、`tools/dev.py`編集。既存`rust-wrapper-runtime-test` commandにintegration testを含める案。

## 実施手順案

1. 利用者がRust DD v1.6案・test design v0.9案とworker lease/API分割をレビューする。承認前にsource・unit manifest・branch変更を行わない。
2. 承認後に計画と設計案をcommit/pushし、`UNIT-WRAP-RS-CMD-001`を`tools/units.yaml`へreadyとして登録する。
3. `freemvvm-sys`へCommand/CancelToken/runtime_post ABI宣言を追加し、safe crateへAsyncCommand/OperationId/CommandOperation/CancellationToken/CommandState APIを追加する。workspace `src/lib.rs`のexportはこの実装で変更対象となる。添付snapshot自体は変更しない。
4. command start callback userdataを成功時のみCoreへ移転し、callback panic時にFFI境界を越えない。worker contextのretained tokens、Runtime anchor、cleanup ticket、sendabilityを型で分離する。
5. worker terminal methodsは`runtime_post`へcallbackを一回enqueueし、Core complete APIはUI threadでのみ呼ぶ。post受付と終端状態反映を区別し、enqueue失敗はreport retry資源を保持する。
6. native integration testsでexecute/reentry/state transitions、cancel token、worker complete post、failure snapshot、stale report、explicit close、shutdown destroyを検査する。compile-fail/API integration testでSend境界とpublic surfaceを検査する。
7. `cargo fmt`、Clippy、workspace tests/build、Rust 1.85 minimum、`tools/dev.py check` / `minimum`、docs/manifest validationを通し、Windows/Linux CI後に結果を記録してunitをcompleteにする。

## 完了条件

- C ABIのthread/lifecycle契約をsafe Rustへ正確に写し、Coreロジックをwrapper内で再実装しない。
- workerへ渡る型だけがSend可能で、UI-affine Command/Propertyは`!Send/!Sync`を保つ。
- operation ID・cancel request・terminal outcomeを区別し、stale/duplicate completionが新しい実行を上書きしない。
- worker completionはpost経由、shutdown後の未反映終端を成功扱いにしない。
- local gatesおよびWindows/Linux CIに合格し、コードと対応testが同一実装commitに含まれる。

## レビュー待ち事項

- `CommandOperation`と`CompletionReporter`のSend境界・report APIの再試行/重複呼出し semantics。
- `create` closure panicをCore generic failureに写像する扱い。
- `Command::close`にnative closeとpublic token releaseを含めるか、両操作を別メソッドにするか。
- state Propertyを`Property<i64>`で公開し未知値を保持する案。
- scopeを1 unitとし、worker lease/cleanupも同じunitへ含める粒度。

## 実施記録

- Rust `-sys`にCommand/CancelToken/Runtime post ABI宣言を追加し、safe crateに`AsyncCommand`、`CommandOperation`、`CancellationToken`、`OperationId`、`CommandState`を実装。
- `AnyThreadOwned`の独立retainをworker operation leaseに利用。UI-affine ownerは`!Send`を維持し、workerの終端結果は`runtime_post`でUI threadへ渡す。
- Rust runtime integration tests: 17 passed。別crate public API test: 1 passed。compile-fail doctest: 4 passed。
- Local gates: `cargo clippy --workspace --all-targets --locked -- -D warnings`、`tools/dev.py check`、Rust 1.85 `tools/dev.py minimum`、document linksとunit manifest JSONがpass。
- Windows/Linux CI runと完了記録はpush後に追記する。`tools/dev.py`と添付read-only snapshotは変更していない。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | 計画・Rust DD v1.6・test design v0.9 | 利用者 | 「OK」により承認。実装開始 |
