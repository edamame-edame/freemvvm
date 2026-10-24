# C# Property Binding wrapper

- 更新日: 2026-10-24
- 状態: 計画案・利用者レビュー待ち（未承認・未commit）
- 対象: 同一Runtimeの同型`UiProperty<T>`間にOneTime/OneWay/TwoWay Bindingを作り、所有・明示解除・遅延Binding errorの読み取りと通知をC#から利用する。
- 作業ブランチ案: `impl-csharp/wrappers-runtime-api`

## 背景と範囲

C# Runtime、BOOL/I64/F64/UTF-8/DECIMAL Property、Property notification Subscriptionまでの各unitは実装済み。共通C ABIにはBinding create/cancel/error snapshot/error subscriptionがあり、C++ wrapperにもBinding APIとfake ABI試験、package consumerがある。一方、C# DD v1.2にはBinding具体APIがなく、C# unitも未登録である。

本unitは既存C ABIを使うC# facadeを追加する。異種型変換、validator登録API、新しいC ABI/Core動作、Binding graphや伝播意味論の変更、NuGet/package配布は含めない。

## 承認済み根拠

- 要求: [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)、[RQ-07-001](../11_requirements/RQ-07_binding.md#rq-07-001)～[RQ-07-011](../11_requirements/RQ-07_binding.md#rq-07-011)。RQ-07は文書版0.7で承認済み。
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。C# facadeはRust Coreの既存versioned C ABIを利用する。
- 詳細設計根拠: [共通ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)のBinding create/cancel/error snapshot/error subscription、および[notification DD v1.1](../13_detail_design/events/notifications-binding.md)の初期通知なし・in-flight callback・cancel/destroy規則。
- 参照実装: [承認済みC++ Binding DD v1.3](../13_detail_design/wrappers/cpp.md#property-binding-api)、[C++ Binding plan](cpp-binding-wrapper.md)、C++ `Binding`/`BindingError`/`Subscription` implementationとfake ABI tests。
- 先行C#実装: [C# Property Subscription plan](csharp-property-subscription.md)、[C# wrapper DD v1.2](../13_detail_design/wrappers/csharp.md)、[wrapper tests](../32_unit_test/wrappers.md)。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[implementation test inventory](../32_unit_test/implementation-test-inventory.md)、[release-test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装案（後続DDレビューの対象）

- 同じ`T`のProperty間だけを対象とし、`UiBindingMode`はOneTime/OneWay/TwoWayを持つ。OneTime成功はBinding handleを持たない成功結果として表し、OneWay/TwoWay成功はowned `UiBinding<T>`を返す案とする。
- `UiBinding<T>`は明示的な`TryCancel()`、`TryClose()`、`Dispose()`を提供する案とする。cancelはCore接続を解除し、close/disposeはBinding handleだけをreleaseする。release単独で伝播を解除しない。失敗したtokenは再試行可能な既存ownerへ残す。
- `TryGetErrorSnapshot()`はerrorなしを空値、errorありを不変のerror snapshot・拒否されたPropertyの新しいowned wrapper・伝播方向として返す案とする。両owned result token分のcleanup ticketをC ABI call前に準備する。
- `TrySubscribeError(Action)`は既存C# `UiSubscription`、callback trampoline/root、Runtime callback failure handlerを再利用する案とする。初期callbackは発生せず、callback中にsnapshotを再取得する。callback failure kindにBinding error notificationを追加する。
- C ABIのcreate/cancel/snapshot/subscription statusとerror snapshotを保ち、C#側で値、方向、グラフ状態を補正しない。初期同期失敗と遅延検証拒否を区別し、既取得snapshotの不変性を維持する。

## 試験案

- fake ABI focused testで全5値型と3 mode、OneTime成功時の空結果、OneWay/TwoWay token ownership、初期同期失敗とerror cleanup、cancel/close失敗retry、move/Dispose、error snapshotの各出力とtoken解放を検査する。
- fake ABIでerrorなし/ありsnapshot、拒否Property ownerと方向、取得済snapshotの不変性、error recovery、error Subscriptionの初期通知なし/cancel/root lifetime/destroyを制御する。遅延Validatorの公開登録APIは本unitで追加せず、Core/C ABIのValidator試験に委ねる。
- real C ABI consumerで全5型のOneTime/OneWay/TwoWayと初期同期・値伝播・cancel/close分離を検査する。notification配信の確認にはUI loop bridge登録とdispatch turnを使用する。
- `.docs/32_unit_test/wrappers.md`へ`UT-WRAP-CS-BIND-001`と`UT-CONSUMER-CS-BIND-001`を実装前に登録し、`tools/units.yaml`へ`UNIT-WRAP-CS-BIND-001`と依存・設計・要求・case・commandsを登録する。
- focused test、real consumer、`tools/dev.py check`、`tools/dev.py minimum`、document links、manifest parse、`git diff --check`を実装revisionで通し、Windows/Linux CI結果を計画・台帳・`.docs/01_memory/`へ記録する。

## 実施順と承認境界

1. [ ] 利用者がこの計画の範囲・単位・試験境界をレビューする。
2. [ ] 計画承認後、日本語commit messageで計画をcommit/pushする。
3. [ ] C# wrapper DD v1.3改訂案にBinding public API、OneTime結果、cancel/close、error snapshot/subscription、所有・失敗・thread規則を具体化し、文書全体の承認を得る。
4. [ ] DD承認後、test designとunit mappingを登録して計画を更新する。
5. [ ] 利用者から明示的な実装開始指示を受けた後、C# Binding facadeとfake ABI testsを実装する。
6. [ ] real C ABI consumer、local checks、Windows/Linux CIを通し、実装・検証記録をcommit/pushする。

計画承認、DD v1.3全文承認、実装開始承認は別々に確認する。本案の作成・承認だけではC# DDや公開APIを確定せず、実装開始を許可しない。

## 計画レビューで確認する点

- C++ Binding wrapperと同じ意味のgeneric C# Binding facadeを、全5 Property型に一括で提供する範囲でよいか。
- Binding error snapshotとerror Subscriptionはfake ABIでwrapper contractを検査し、real consumerはvalidator未公開のためmode・伝播・解除の実Core縦断に限定する境界でよいか。
- 次段階のC# DD v1.3提案へ進む前に、このunit計画だけを承認するか。
