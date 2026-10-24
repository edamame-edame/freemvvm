# C# Property notification Subscription

- 更新日: 2026-10-24
- 状態: 実装完了・focused/consumer/local check成功・CI確認待ち
- 対象: C# `UiProperty<T>`からnotification Subscriptionを登録し、明示cancel・handle close・managed callback root・callback例外通知を安全に扱う。

## 背景と範囲

C# Runtime ownerとBOOL/I64/F64/UTF-8/DECIMAL Property facadeが実装済み。既存C ABIのProperty subscriptionと共通Runtime callback handlerを利用し、C# callback lifetimeを追加する。Binding、Validator、Command、callback payloadを受け渡す新ABI、Core notification変更は含めない。

## 承認済み根拠

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)。
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。C# wrapperは既存versioned C ABIのcallback + userdata + destroy契約を使う。
- 詳細設計: [承認済みC# wrapper DD v1.2](../13_detail_design/wrappers/csharp.md)のCallbackRoot/exception境界/Runtime handler規則、[共通ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)のProperty subscription/cancel、[notification DD v1.1](../13_detail_design/events/notifications-binding.md)のin-flight callback/cancel/destroy規則。
- 参照実装・試験: [C++ Property subscription](../13_detail_design/wrappers/cpp.md)、[C++ subscription tests](../32_unit_test/wrappers.md#ut-wrap-cpp-prop-sub-001)。C++ wrapperは`subscribe`、明示`cancel`、handle `close`を分け、callback/handler例外をABI境界へ出さない。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[release-test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装範囲

- 公開APIを`UiProperty<T>.TrySubscribe(Action callback)`、owned `UiSubscription.TryCancel()`/`TryClose()`/`Dispose()`とする。notification callbackには値を重複して渡さず、利用者はcallback内で対応Propertyの`TryGet()`を呼んで最新値を取得する。登録時の即時callbackは行わない。
- handle `TryClose`/`Dispose`はsubscription owned referenceだけを解放し、notificationをcancelしない。明示`TryCancel`成功後は新しいcallbackが開始しない。close/releaseとcancelは独立操作であり、失敗したtokenは保持して再試行可能とする。
- C#はsubscriptionごとのstatic trampolineとGCHandle-rooted callback stateを使う。登録失敗時はmanaged側がrootを回収し、登録成功後はCore destroy callbackがin-flight invocation終了後に一度だけrootを解放する。trampolineはmanaged exceptionをcatchしてABIへ出さない。
- 公開例外通知APIを`UiRuntime.TrySetCallbackFailureHandler(Action<UiCallbackFailure> handler)`とする提案。戻り値はowned `UiCallbackHandlerRegistration`で、そのcloseは同じregistrationが現在もactiveの場合だけhandlerを解除する。新handler登録はRuntime内の一つのimmutable recordをatomic交換し、旧recordはin-flight handler call終了後に回収する。`UiCallbackFailure`はkindと捕捉した`Exception`を持つ。
- failure handler未設定時とfailure handler自体がthrowした場合は`System.Diagnostics.Trace.TraceError`へ一回記録する提案。ユーザーhandlerはcallback中に同期実行する。handlerの再入・UI API実行可能範囲はC# DD/RQの制約に従い、追加のdispatchやexception propagationをしない。
- Core/C ABIのcallback payload、cancel完了、queue order/coalescing、runtime shutdown時のcleanupは既存設計に委ねる。wrapperはそれらの意味論を再実装しない。

## 試験範囲

実装前に[wrapper test design](../32_unit_test/wrappers.md)へfake ABI testと別process consumer testを追加し、`tools/units.yaml`へunit mappingを登録した。

- fake ABIでsubscribe成功/失敗、同期callback、callback state transfer、destroyを制御し、登録失敗時のmanaged root回収と成功後のdestroy一回を確認する。
- Runtime callback handlerの登録、交換、旧registration closeによる新handler保持、解除、in-flight snapshot、handler例外を検証する。未設定/handler例外の診断がTrace listenerへ一度届くことを確認する。
- callbackから`TryGet`してProperty更新値を確認する。notification callback例外を発生させてもC ABI境界へ出ず、次callbackとRuntime利用を継続する。
- cancel成功後の新規callback停止、handle release単独ではcancelされないこと、callback in-flight中のcancel/destroy順、cancel/close失敗時のtoken保持と再試行を確認する。
- 別processのreal C ABI consumerでProperty変更によるcallback、callback内のgetter、明示cancel/close、Runtime closeを実行する。
- focused test、consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書リンク・diff検査を実装revisionで実行し、Windows/Linux CI結果を計画・unit台帳・`.docs/01_memory/`へ記録する。

## 実施順

1. [x] 利用者が公開API案、cancel/close分離、callback例外handlerと既定診断の案をレビューし、「ok，進めて」により計画を承認した。
2. [x] C# wrapper DD v1.2を改訂し、2026-10-24に利用者の「ok」で文書全体の承認を得た。
3. [x] 承認済み計画をcommit `a165368`で記録し、現在のC#作業ブランチへpushした。
4. [x] DD承認後、`UT-WRAP-CS-PROP-SUB-001` / `UT-CONSUMER-CS-PROP-SUB-001`の試験設計と`UNIT-WRAP-CS-PROP-SUB-001` mappingを製品source変更前に登録した。
5. [x] CallbackFailure/handler、static trampoline/root、Subscription facadeとfake ABI testsを実装した。`tools/dev.py csharp-property-test`成功。
6. [ ] real C ABI consumer、全体check、minimum toolchain check、文書リンク検査を通過。`tools/units.yaml`、wrapper test design、unit台帳、計画、`.docs/01_memory/`を更新済み。実装commit後のWindows/Linux CI成功を確認してcommit/pushする。

## 承認済みの決定

- `TrySubscribe(Action)`は通知callbackで値を渡さず、callback中の`TryGet()`で値を再読する。
- subscription handleの`TryClose`/`Dispose`をcancelから分離し、必要時にcancel成功後handleをcloseする。
- C# callback failure handlerはRuntime単位で交換可能とし、handler未設定/handler自身の例外を`Trace.TraceError`へ一度記録する。
- 上記はC# DD v1.2全文承認の対象に含まれる。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | C# Property notification Subscription計画 | 利用者 | 「ok，進めて」により計画を承認。DD v1.2全文と実装の承認は別途確認する |
| 2026-10-24 | C# wrapper DD v1.2 | 利用者 | 「ok」によりSubscription API、handler lifecycle、既定診断を含む文書全体を承認。 |

## 実装・検証記録

- fake ABI focused tests: `.venv/Scripts/python.exe tools/dev.py csharp-property-test` 成功。
- real C ABI consumer: `.venv/Scripts/python.exe tools/dev.py csharp-property-consumer-test` 成功。
- repository check: `.venv/Scripts/python.exe tools/dev.py check` 成功。document links、format/lint、Rust build/clippy、C/C++ probes/package、C# focused/consumer、Python package checksを含む。
- minimum toolchain: `.venv/Scripts/python.exe tools/dev.py minimum` 成功（Rust 1.85.0）。
- 検証環境: Windows 11、2026-10-24。GitHub Actionsは実装commit push後に確認する。
