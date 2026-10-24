# Rust Runtime safe wrapper基盤

- 更新日: 2026-10-24
- 状態: 計画承認済み（利用者、2026-10-24）。計画commit/push後、実装開始の承認を待つ
- 対象: 共通C ABIのRuntime lifecycle・UI thread context・owned handle基盤をRust `freemvvm-sys` / `freemvvm`から利用する最初のwrapper unit
- 作業ブランチ: `impl-rust/wrappers-runtime-api`（親: `impl-rust/runtime-async-command`）

## 背景と範囲

Rust用の[`freemvvm-sys`](../../wrappers/rust/freemvvm-sys/src/lib.rs)とsafe [`freemvvm`](../../wrappers/rust/freemvvm/src/lib.rs)はworkspaceに登録済みだが、両crateはskeletonのみで、raw ABI宣言・native runtime接続・safe facadeは未実装である。C# Command unitの完了を受け、依存層であるCore APIs枝を親にしたRust wrapper枝を用意した。

最初のunitはRust wrapper全機能を一括実装せず、他のsafe APIが前提とするRuntime lifetimeとUI contextを対象にする。Rust Coreをwrapperへ含めず、共通native C ABI artifactを利用する。Property、Subscription、Binding、Command、optional Future、crate配布・artifact取得機能は後続unitとする。

## 承認済み根拠

- 要求: [RQ-03-003](../11_requirements/RQ-03_language-support.md#rq-03-003)、[RQ-03-007](../11_requirements/RQ-03_language-support.md#rq-03-007)、[RQ-03-014](../11_requirements/RQ-03_language-support.md#rq-03-014)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)、[RQ-04-015](../11_requirements/RQ-04_abi-boundary.md#rq-04-015)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)、[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)、[RQ-06-006](../11_requirements/RQ-06_threading-dispatch.md#rq-06-006)、[RQ-12-003](../11_requirements/RQ-12_build-distribution.md#rq-12-003)、[RQ-12-005](../11_requirements/RQ-12_build-distribution.md#rq-12-005)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md) — Rust Coreとversioned C ABIを採用し、各言語は共通native runtimeを使う。
- 詳細設計: 承認済み[Rust wrapper DD v1.1](../13_detail_design/wrappers/rust.md)の「所有型・借用・Drop・panic境界」「公開API・registry・Future」。
- 基本設計: 承認済み[Runtimeの責務・終了設計](../12_design/runtime-objects/runtime.md)。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[requirements test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装提案

- `freemvvm-sys`にversioned ABIの固定幅status/handle型と、Runtime create・UI thread query・Runtime shutdown/receipt query・handle retain/release・error snapshot読取に必要なraw FFIを宣言する。safe crateは内部Coreに依存しない。
- `freemvvm`に`UiRuntime`、`UiContext`、statusを保持する`UiError`、typed `Owned<T>`/`Borrowed<'a, T>`のRuntime用基盤を実装する。Runtimeに属するUI-affine ownerは`Send`/`Sync`にせず、UI call直前にもnative thread statusを検査する。
- 明示close失敗では所有tokenをownerに残し、Drop経路でreleaseが失敗したときは生成前に確保したcleanup ticketへ移管する。Runtime anchor、ticket、tokenの解放順はDDに従う。panicはC ABI境界を越境させない。
- dev/testは既存のC ABI shared artifactを使用し、Coreの静的複製を作らない。artifactの配布、fetch、target/profile selection機能は本unitに追加しない。開発時link経路の具体化でRQ/DDにない利用者向け挙動が必要と判明した場合は、その挙動を確定せず差異と変更対象を記録して止める。

## 試験計画案

- source変更前に[wrapper test design](../32_unit_test/wrappers.md)へRust Runtime focused case `UT-WRAP-RS-RT-001`と実C ABI consumer case `UT-CONSUMER-RS-RT-001`を登録し、`tools/units.yaml`へ`UNIT-WRAP-RS-001`を追加する。
- focused testではfake sys APIを使い、create/retain/release status、unknown status/domain/code保持、UI-thread拒否、explicit close retry、Drop release失敗のticket移管、Runtime anchor順序を検証する。compile-fail fixtureでUI-affine ownerが`Send`/`Sync`でないこととBorrowed lifetime制約を確認する。
- consumer testでは別Rust executableから実C ABIを呼び、Runtime create、UI thread query、shutdown request/query、明示closeを検証する。異なるtargetのnative artifactを使わない。
- `cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、workspace build/test、Rust 1.85 minimum、wrapper focused/consumer tests、`tools/dev.py check`、文書リンク・manifest検査を実装と同じrevisionで通す。Windows/Linux CIの結果を本計画・test design・unit台帳・`.docs/01_memory/`へ記録する。

## 実施順と承認境界

1. [x] 利用者が本計画案のunit範囲・ABIとsafe-wrapperの試験境界を承認する（2026-10-24）。
2. [ ] 承認済み計画をcommit/pushする。
3. [ ] 明示的な実装開始指示後、test designとunit mappingをsource変更前に登録する。
4. [ ] Rust `-sys`宣言、Runtime safe facade、focused tests、real C ABI consumerを実装する。
5. [ ] local gatesとWindows/Linux CIを通し、結果を記録して実装unitをcommit/pushする。

## 承認履歴

| 日付 | 対象 | 承認者 | 結論 |
| --- | --- | --- | --- |
| 2026-10-24 | 本計画全文 | 利用者 | 「OK」によりRust Runtime safe wrapper unitの範囲・試験境界を承認。実装開始は別途承認 |

計画承認、実装開始承認、検証完了は別々に記録する。計画は承認済み。製品source変更は未着手で、実装開始承認を待つ。
