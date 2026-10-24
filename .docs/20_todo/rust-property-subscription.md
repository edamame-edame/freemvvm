# Rust wrapper: Property Subscription

- 更新日: 2026-10-24
- 状態: 計画/DD 1.4承認済み（利用者、2026-10-24）。実装済み・ローカル検証済み、CI待ち
- 対象ブランチ: `impl-rust/wrappers-runtime-api`
- 親: [Rust DECIMAL Property](rust-property-decimal.md)

## 目的と範囲

承認済みRust callback境界と共通C ABIに従い、RustからProperty notificationを登録・cancel・closeできるsafe APIを追加する。添付`property.rs` snapshotは変更しない。実装時に必要なworkspace側`property.rs`の変更は、Rust DD 1.4と本計画のレビュー後に行う。

Binding、Validator、Dispatcher連携、async streamは別unitとする。通知値はcallback引数として渡さず、notification後に利用者がPropertyを読む現行C ABI規則を維持する。

## 提案するRust API

```rust
impl<T: PropertyValue> Property<T> {
    pub fn subscribe<F>(
        &self,
        ui: &UiContext<'_>,
        callback: F,
    ) -> Result<Subscription, UiError>
    where
        F: FnMut() + 'static;
}

impl Subscription {
    pub fn cancel(&mut self, ui: &UiContext<'_>) -> Result<(), UiError>;
    pub fn close(&mut self) -> Result<(), UiError>;
}
```

CallbackはUI-onlyなので`Send`を要求しない。subscribeは初期通知を行わない。`cancel`は新規callback開始を止める。`close`/Dropは公開Subscription tokenのみをreleaseし、registrationをcancelしない。failed cancelは所有tokenを保持してretry可能にする。in-flight callbackは終了後にclosure destroyを一度だけ行う。

## 根拠

- 要求: [RQ-03-003](../11_requirements/RQ-03_language-support.md#rq-03-003)、[RQ-03-007](../11_requirements/RQ-03_language-support.md#rq-03-007)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- 設計: [Rust wrapper DD 1.4改訂案](../13_detail_design/wrappers/rust.md)、[通知とBinding伝播](../13_detail_design/events/notifications-binding.md)、[共通C ABI](../13_detail_design/runtime-objects/abi.md) Property subscribe/cancel
- 試験方針: [unit test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)

## 実装・確認項目

1. Rust DD 1.4でmethod signatures、closure Send/lifetime、close-vs-cancel、panic diagnostic、Runtime/Property teardownの責務を文書全体で承認する。
2. `freemvvm-sys`へsubscribe/cancel raw symbolsを追加し、safe trampoline/destroy thunkは新しいSubscription moduleへ局所化する。
3. subscribe成功時だけclosure stateをCoreへ移す。失敗時はRust側でdropする。ticketとRuntime anchorはnative owned output受領前に確保する。
4. Property::subscribeはRuntime identity/UI threadを検査する。Subscription::cancelは明示解除、close/Dropはtoken releaseだけを行う。
5. callback panicはABIをunwindせず捕捉し、後続subscriberへ配信を継続する。destroy中panicもABIを越えない。
6. crate-level integration testでtest host bridgeとlinked native C ABIを使い、初期通知なし、値変更通知、same-value無通知、close後もactive、cancel後停止、Property teardown cleanup、callback panic後の継続を確認する。
7. unit-test design、manifest、dev command、local full gates、Rust 1.85、Windows/Linux CIを記録する。

## 完了条件

- DD 1.4全文承認後、提案APIの名前・ownership・failure behaviorを変更せず実装する。
- 登録成功/失敗、callback root、cancel/close分離、in-flight終了、Property teardown/shutdown cleanupを確認する。
- 添付snapshotは編集しない。既存Property値mapping・handle実装を壊さない。
- unit stateとverification recordがCIを実行したcommitを示す。

## 履歴

| 日付 | 対象 | 内容 |
| --- | --- | --- |
| 2026-10-24 | 計画・DD案 | Rust Subscription APIとowner lifecycleがDD 1.2に未定義のため、公開API案・test plan・実装境界を作成。DD 1.4と計画の全文レビュー前にsource変更・commit/pushを行わない。 |
| 2026-10-24 | 計画・DD承認 | 利用者の「OK」によりRust DD 1.4全文、Subscription計画、test designを承認。workspace側`property.rs`を変更して実装する。 |
| 2026-10-24 | 実装案・ローカル検証 | subscribe/cancel/closeとC ABI host-dispatch integration testを追加。focused runtime tests、clippy、全bootstrap `check`とRust 1.85 `minimum`は成功。Windows/Linux CIは未実行。 |
