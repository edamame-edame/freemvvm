# C# BOOL/I64 Property wrapper

- 更新日: 2026-10-24
- 状態: 完了（実装commit `0330ed2`、Windows/Linux CI成功、2026-10-24）
- 対象: 承認済みC# wrapper DD v1.1の`UiProperty<T>`を実装し、BOOL/I64 Propertyの作成・読取・設定・closeを一つの検証可能なunitとして提供する。

## 背景と範囲

先行unit [C# Runtime wrapper基盤](csharp-runtime-wrapper.md)で`UiRuntime`、ABI interop、`TokenOwner`/`CallLease`、error snapshot、cleanup retryを実装した。現在はProperty APIがなく、C#利用者がRuntime上に値を作成・共有できない。

初回Property unitはBOOLとsigned I64に限定する。これらのABI mappingは`byte`/`long`で曖昧さがなく、C++ wrapperのBOOL/I64 Propertyと同じ基本fixtureで実動作を確認できる。UTF-8の厳密なmanaged string変換、F64のbit preservation、Decimal64変換はそれぞれ別unitに分ける。Subscription、Binding、callback trampoline、配布packageも対象外とする。

## 承認済み根拠

- 要求: [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)。RQ本文と改訂履歴で承認済み。
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust CoreのPropertyをversioned C ABI越しにC# facadeへ提供する。
- 詳細設計: [C# wrapper DD v1.1](../13_detail_design/wrappers/csharp.md)の「公開API・call lease・native load」「型の責務と所有」、[共通ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)のProperty create/get/set・handle release。
- 先行実装: [C# Runtime wrapper](csharp-runtime-wrapper.md)。Property操作は既存`TokenOwner`/`CallLease`と`UiResult<T>`を再利用する。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[release-test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装提案

- 公開API案: `UiProperty<T>.TryCreate(UiRuntime runtime, T initial)`、`TryGet()`、`TrySet(T value)`、`TryClose()`、`Dispose()`。生成対象の`T`は`bool`と`long`に限定し、それ以外をnativeへ渡す前に`UI_UNSUPPORTED`で拒否する。
- `bool`はC ABI `ui_bool`の0/1 byteに明示変換し、nativeから0/1以外が返った場合は不正ABI結果として扱う。`long`はC# `long`とABI `int64_t`を損失なく往復する。
- create/setはC ABIのerror tokenから既存のimmutable `UiErrorSnapshot`を作り、get失敗はraw statusを保持する。全native callを`CallLease`で囲み、Property close/Dispose競合とrelease retryにRuntime実装と同じ規則を適用する。
- 失敗したsetを成功扱いにせず、値保持・同値判定・通知の意味論はRust Coreと既存ABIの正本に委ねる。wrapperは独自cacheを値の正本にしない。
- UTF-8、F64、Decimal64の変換規則やAPIをこのunitで先取りしない。

## 試験提案

実装承認後、source変更前にtest designへcaseを追加し、`tools/units.yaml`へ`UNIT-WRAP-CS-PROP-BOOL-I64-001`を登録する。

- fake native dispatch focused testでBOOL/I64 create/get/set、0/1境界、I64 min/max、wrong-threadと未知raw status、create/set error snapshot、failed close retry、active CallLease中のDisposeを検査する。fakeはnative call数・token release回数・渡されたraw値を記録する。
- 別process consumerから実C ABIへ接続し、Runtime上でBOOL/I64をcreate/get/set/closeし、設定値が往復することを確認する。UI loop bridgeのないhostでも通常のProperty操作が完了することを確かめる。
- C# focused tests、consumer、全体`tools/dev.py check`、Rust minimum gate、文書リンク検査を同じimplementation revisionで実行する。Windows/Linux CI成功後にtest reportとrevisionを本計画・unit台帳・memoryへ記録する。

## 実施順

1. [x] 利用者が本計画のscope、BOOL/I64単位、公開API案をレビューし承認した（2026-10-24、「ok」）。
2. [x] 承認計画を`a6473fc`（「C# BOOL I64 Property wrapper計画を承認記録」）でcommit/pushした。
3. [x] 実装開始承認後、C# Property focused/consumer試験設計と`UNIT-WRAP-CS-PROP-BOOL-I64-001` mappingをsource変更前に登録した。
4. [x] Property facadeとfake ABI focused testsを実装した。
5. [x] 実C ABI consumer roundtripを追加した。Windows local `tools/dev.py check`、Rust `tools/dev.py minimum`、文書・Ruff・diff検査が成功した。
6. [x] Windows/Linux CI成功後、unit台帳・計画・`.docs/01_memory/`を完了更新した。implementation commitと検証記録commitを日本語でcommit/pushした。

## レビュー時の決定点

- 初回をBOOL/I64に絞る案でよいか。UTF-8/F64/Decimal64を一つの大きなProperty unitにまとめず、別々の小さな実装単位とする。
- static factoryを`UiProperty<T>.TryCreate(runtime, initial)`に置く案でよいか。承認済みDDは`UiProperty<T>` facadeと`TrySet/Get`を定めるが、create APIの具体的な配置を明記していないため、この配置は提案としてレビュー対象にする。
- generic facadeの初回対応型を`bool`/`long`のみに制限し、未対応型をunsupportedで明示拒否する。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | 計画scope・公開API案 | 利用者 | 「ok」によりBOOL/I64 scopeと`UiProperty<T>.TryCreate`案を承認 |
| 2026-10-24 | 本計画に沿う実装開始 | 利用者 | 「実装も進めて」により実装を承認 |

## 検証結果

- 実装commit: `0330ed2`（`C# BOOL I64 Property wrapperを実装`）。
- ローカル: `tools/dev.py csharp-property-test`、`tools/dev.py csharp-property-consumer-test`、Windows `tools/dev.py check`、`tools/dev.py minimum`、`tools/check_docs.py`、Ruff、`git diff --check`が成功。
- GitHub Actions: [Development bootstrap run 40049830264](https://github.com/edamame-edame/freemvvm/actions/runs/40049830264)でWindows/Linux両job成功。
- 確認範囲: BOOL 0/1とI64端値の往復、失敗status/error snapshot、未対応型拒否、close retry、active lease中のDispose競合、実C ABI consumer。
