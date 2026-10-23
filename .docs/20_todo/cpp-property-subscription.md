# C++ Property 通知購読とcallback境界の計画

- 更新日: 2026-10-23
- 状態: 実装・Windowsローカル検証完了。Windows/Linux CI結果の記録待ち（2026-10-23）
- 対象: C++ wrapperからのProperty通知購読、明示cancel、callback stateの所有移転

## 背景と根拠

C++ PropertyのBOOL/I64/UTF-8/F64/DECIMAL値mappingは完了した。C ABIには`ui_abi_1_property_subscribe`と`ui_abi_1_subscription_cancel`があり、Coreは変更通知を登録順で配信するが、C++ wrapperにはSubscription型・callback trampoline・購読APIがまだない。

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-008](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-008)、[RQ-05-009](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-009)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreを通知の正本とし、C++はversioned C ABIの薄いwrapperとする。
- 基本設計: [値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)。Property commitと通知配信を分離する。
- 詳細設計: 承認済み[C++ wrapper DD v1.2](../13_detail_design/wrappers/cpp.md)はcallback state、noexcept trampoline、C++ exceptionの捕捉、Binding/Subscriptionのowned token・明示解除を定める。[callbackと再入DD v1.1](../13_detail_design/events/callbacks.md)は例外を診断経路へ記録して他subscriberを続ける規則を定める。[通知とBinding伝播DD v1.1](../13_detail_design/events/notifications-binding.md)は登録順、generation snapshot、cancel後のin-flight完了、Property teardown・Runtime shutdownとの関係を定める。

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
- C++ DDの`Runtime::set_callback_handler`を実装し、`CallbackFailure`を受けるhandler、userdata、destroy関数をRuntime単位で登録する。登録の交換では旧recordをin-flight callback終了後に一度だけdestroyする。`HandlerRegistration::close()`は自分が登録したrecordが現在もactiveな場合だけ解除する。
- trampolineはC++例外をC ABI境界から漏らさない。handlerがある場合は`CallbackFailure`を渡し、ない場合はstderrへ通知callbackの例外を記録する。`std::exception`は`what()`を記録し、non-standard exceptionは固定メッセージを記録する。handler自体の例外も捕捉し、stderrへ固定メッセージを一度記録する。通知例外は他subscriberの実行やcommit済みProperty値を取り消さない。

### 試験範囲案

- fake C ABIで各Property値型のsubscribe/cancel/releaseを確認する。登録失敗時のcallback state保持、成功時のownership transfer、token releaseとcancelの独立性、cancel失敗時の再試行、move、destructor cleanupを検証する。
- deterministic callback hookで通知callbackの呼出し、self-cancel、cancel中のin-flight lifetime、追加購読の次generation適用、C++ exception containment、handler交換中のin-flight保持、handler/closure destroy一回性、handler未登録時とhandler failure時のstderr診断を確認する。Coreの通知順序/coalescingそのものはCore testsの責務とする。
- relocated CMake package consumerで`Property<int64_t>`購読・変更・明示cancel・closeを確認する。
- 実装前に`UT-WRAP-CPP-PROP-SUB-001`と`UT-PKG-CPP-006`を`.docs/32_unit_test/wrappers.md`および`tools/units.yaml`へ登録し、実装unitを`UNIT-WRAP-CPP-006`とする。

## 実施順

1. [x] 公開API案・callback例外のstderr既定動作を利用者が承認済み。計画を日本語commitで記録した。
2. [x] C++コーディング規則に従ってテスト設計とunit mappingをsource変更より先に登録した。
3. [x] `Subscription` owner、`Property<T>::subscribe`、Runtime callback handler、noexcept trampoline/destroy thunkを追加した。C ABI/Coreの通知意味論は変更していない。
4. [x] fake ABI focused testとrelocated package consumerを実装し、focused CTestと全bootstrap gatesを実行した。
5. [ ] Windows/Linux CIで同一実装revisionを確認し、計画・unit台帳・`.docs/01_memory/`に結果を追記する。

## ローカル検証結果

- Windows: Visual Studio 2022 Build Tools環境で`tools/dev.py check`成功。CMake/CTest 10件すべてpassし、移設後CMake package consumerもpassした。`tools/dev.py minimum`もRust 1.85.0で成功した。
- `tools/check_docs.py`成功（active documents 76、local links 6363）。`git diff --check`成功。
- このPCの通常環境はVisual Studio 18 Build Toolsを優先するが、標準C++ヘッダーが未導入のため、検証はインストール済みのVisual Studio 2022 Build Toolsを明示して実行した。

## レビュー対象

利用者は2026-10-23に「ok」と返信し、Property subscription API案と試験範囲を承認した。既定例外診断に関する質問には「stderrへ」と回答し、handler未設定時のstderr記録を承認した。C++ DDに定義済みのRuntime callback handlerもこの実装単位へ含める。C ABI/Core通知意味論は変更しない。
