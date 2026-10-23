# C++ Property 通知購読とcallback境界の計画

- 更新日: 2026-10-23
- 状態: 提案。API案と実施範囲のレビュー待ち
- 対象: C++ wrapperからのProperty通知購読、明示cancel、callback stateの所有移転

## 背景と根拠

C++ PropertyのBOOL/I64/UTF-8/F64/DECIMAL値mappingは完了した。C ABIには`ui_abi_1_property_subscribe`と`ui_abi_1_subscription_cancel`があり、Coreは変更通知を登録順で配信するが、C++ wrapperにはSubscription型・callback trampoline・購読APIがまだない。

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreを通知の正本とし、C++はversioned C ABIの薄いwrapperとする。
- 基本設計: [値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)。Property commitと通知配信を分離する。
- 詳細設計: 承認済み[C++ wrapper DD v1.2](../13_detail_design/wrappers/cpp.md)はcallback state、noexcept trampoline、C++ exceptionの捕捉、Binding/Subscriptionのowned token・明示解除を定める。[通知とBinding伝播DD v1.2](../13_detail_design/events/notifications-binding.md)は登録順、generation snapshot、cancel後のin-flight完了、Property teardown・Runtime shutdownとの関係を定める。

## 実装提案

### 公開API案

```cpp
auto subscription = property.subscribe([] { /* current valueを必要に応じて読む */ });
subscription.value()->cancel();  // callback登録を無効化
subscription.value()->close();   // 所有handleをrelease
```

- `Property<T>::subscribe(std::function<void()>)`は`Result<Subscription>`を返す。購読時に初回callbackを同期実行せず、Coreが変更通知を配信するときにUI thread上で通知する。callback引数は持たず、必要な利用者状態はclosureで明示captureする。
- `Subscription`はmove-only。`cancel()`はC ABI `subscription_cancel`を呼び、成功後は新規callback開始を抑止する。`close()`はhandle tokenだけをreleaseする。失敗した明示操作ではtoken/stateを保持し再試行可能にする。
- `Subscription` destructorによるhandle releaseは通知登録を解除しない。RQ-05-002に従い、明示cancelまたはProperty teardown/Runtime shutdownまでCore registrationがcallback stateを保持する。
- callback stateはnative registration成功時だけCoreへ所有移転し、登録失敗時はC++側が解放する。destroy thunkはin-flight callbackの終了後に一度だけclosureを解放する。
- trampolineはC++例外をC ABI境界から漏らさない。通知例外は他subscriberの実行やcommit済みProperty値を取り消さない。利用者handlerが未設定の場合の既定diagnostic経路はDDで未確定のため、本実装が出力先や外部挙動を独自に追加しないよう、実装前に採用済み診断APIの有無を確認し、必要なら本計画を改訂して再レビューする。

### 試験範囲案

- fake C ABIで各Property値型のsubscribe/cancel/releaseを確認する。登録失敗時のcallback state保持、成功時のownership transfer、token releaseとcancelの独立性、cancel失敗時の再試行、move、destructor cleanupを検証する。
- deterministic callback hookで通知callbackの呼出し、self-cancel、cancel中のin-flight lifetime、追加購読の次generation適用、C++ exception containmentを確認する。Coreの通知順序/coalescingそのものはCore testsの責務とする。
- relocated CMake package consumerで`Property<int64_t>`購読・変更・明示cancel・closeを確認する。
- 実装前に`UT-WRAP-CPP-PROP-SUB-001`と`UT-PKG-CPP-006`を`.docs/32_unit_test/wrappers.md`および`tools/units.yaml`へ登録し、実装unitを`UNIT-WRAP-CPP-006`とする。

## 実施順

1. 本計画と公開API案・callback例外の扱いをレビューし、利用者承認後に日本語commitで計画を記録する。
2. C++コーディング規則に従ってテスト設計とunit mappingをsource変更より先に登録する。
3. `Subscription` owner、`Property<T>::subscribe`、noexcept trampoline/destroy thunkを追加する。C ABI/Coreの通知意味論は変更しない。
4. fake ABI focused testとrelocated package consumerを実装し、focused CTestと全bootstrap gatesを実行する。
5. Windows/Linux CIで同一実装revisionを確認し、計画・unit台帳・`.docs/01_memory/`に結果を追記する。

## レビュー対象

承認対象は、提案した公開API、明示cancelとhandle releaseの分離、callback stateの所有移転、試験範囲、および未定義の既定diagnosticを独自決定しない境界である。API案または範囲を修正する場合は本書を改訂して再提示する。提案どおりなら「承認します」と返信してください。承認前は本計画をcommitせず、source実装も開始しない。
