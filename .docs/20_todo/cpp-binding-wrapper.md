# C++ Binding wrapper

- 更新日: 2026-10-24
- 状態: 完了。Windows localのfocused test、relocated package consumer、全体check、Rust 1.85 minimum、および実装revisionのWindows/Linux CIが成功
- 対象: 承認済みC++ wrapper、Binding要求、共通C ABIに基づき、C++からBindingの作成・明示解除・遅延エラー観測を使えるようにする。

## 背景

C++ Runtime、Propertyの全5値型、Property通知購読とcallback failure handlerは実装済みで、最新のProperty subscription revisionはWindows/Linux CIで成功した。次のC++ wrapper機能として、Core/C ABIに存在するBinding APIを公開する。

Bindingの外部動作は承認済み[RQ-07 Binding](../11_requirements/RQ-07_binding.md)で定義されている。共通ABIはOneTime/OneWay/TwoWay作成、Binding取消、遅延エラーsnapshot、エラーSubscriptionを定める。C++ DDはowned token、明示deactivate/close、callback寿命の共通規則を定めるが、Binding固有の公開C++型・メソッド名はまだ確定していない。本書のAPIはレビュー案であり、承認前に実装済み・確定仕様として扱わない。

## 承認済み根拠

- 要求: [RQ-07-001〜RQ-07-011](../11_requirements/RQ-07_binding.md)、[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreを動作の正本とし、C++ wrapperはversioned C ABIを呼ぶ。
- 詳細設計: [C++ wrapper DD v1.3](../13_detail_design/wrappers/cpp.md)、[共通C ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)、[通知とBinding伝播 DD v1.1](../13_detail_design/events/notifications-binding.md)
- 試験: [wrapper unit-test設計](../32_unit_test/wrappers.md)、[unit-test strategy](../32_unit_test/unit-test-strategy.md)、[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)

## レビューするC++ API案

```cpp
enum class BindingMode { kOnce, kOneWay, kTwoWay };
enum class BindingDirection { kSourceToTarget, kTargetToSource };

auto result = source.bind_to(target, BindingMode::kTwoWay);
// Result<std::optional<Binding<T>>>:
// kOnce成功なら空、kOneWay/kTwoWay成功ならmove-only Bindingを返す。
result.value()->cancel();  // Core接続を明示解除
result.value()->close();   // 所有handleだけをrelease
```

- `Property<T>::bind_to(Property<T>& target, BindingMode)` は同一C++値型のProperty間だけを受け付け、source/targetを呼出し中だけ借用する。異なるProperty型同士の変換やconverterは含めない。
- OneTime成功はBinding handleを持たないため、`Result<std::optional<Binding>>`で成功と接続なしを表す。失敗は通常の`Result` errorで表し、OneTime成功と混同させない。
- `Binding`はmove-only。`cancel()`はC ABI cancel、`close()`とdestructorはhandle releaseのみを行う。releaseやdestructorを解除操作として扱わない。cancel成功後もBinding handleを別途closeできる。
- `Binding::error_snapshot()`は`Result<std::optional<BindingError<T>>>`を返す。snapshotは不変の`UiError`、拒否されたPropertyの新しい所有wrapper、伝播方向を保持する。errorなしは空値とする。
- `Binding::subscribe_error(std::function<void()>)`は既存Subscription/callback handler機構を再利用し、初回callbackなしとする。callback中に`error_snapshot()`を呼び現在状態を読む。Binding解除・Runtime shutdown時のcallback state破棄はCore契約に委ねる。
- 初期同期失敗、型/Runtime/thread/mode/graph違反はC ABI status/errorを保って返し、C++側で値や接続状態を補正しない。遅延伝播のvalidation失敗はBinding errorとして取得・購読できるようにする。
- Propertyの全5値型を対象とする。新しいC ABI、異種型変換、throwing convenience APIは追加しない。

## 試験範囲案

- fake C ABI focused testで、3 modeの呼出しと戻り値、各5値型でのBinding生成、OneTime成功時の空optional、初期同期失敗と失敗時token cleanupを確認する。
- Binding ownership testで、move、close、cancelとcloseの独立、release単独で接続を維持、cancel後の伝播停止、close/release失敗時の再試行、destructor cleanupを確認する。
- Error testで、エラーなしsnapshot、遅延validation error snapshotの値・拒否Property・方向、取得済snapshotの不変性、回復後の空snapshot、初期同期拒否が状態を変えないことを確認する。
- Error subscription testで、初回通知なし、状態変化通知、明示cancel、callback中のsnapshot取得、in-flight callback中の解除と一度だけのdestroyを確認する。Coreの伝播順・coalescingそのものはCore testsを正とする。
- relocated CMake package consumerで、OneTime/OneWay/TwoWay、値伝播、cancel/closeを使い、公開headerとexport targetだけで実Coreへ接続できることを確認する。C++ wrapperのBinding error snapshot/Subscriptionはfake ABIで状態を制御して検証し、Core/C ABIの遅延validation errorは既存suiteを正とする。validator登録を公開するC++ APIは本unitの範囲外。
- 実装開始前に`.docs/32_unit_test/wrappers.md`へ詳細test設計を追加し、`tools/units.yaml`へ実装path・依存・要求・case・focused test・package testを登録する。候補IDは`UNIT-WRAP-CPP-007`、`UT-WRAP-CPP-BIND-001`、`UT-PKG-CPP-007`。

## 実施順案

1. [x] 利用者がこの計画と公開API案をレビュー・承認した（2026-10-24）。
2. [x] 承認済み計画を日本語commit messageで記録する。unit-test設計とunit mappingは実装開始後、source変更前に登録する。
3. [x] C++ DD v1.3へBinding型、OneTimeの戻り方、cancel/release分離、error snapshotとsubscriptionの公開面を反映した。
4. [x] Property全値型でBinding wrapperとfake ABI focused testsを実装した。
5. [x] relocated CMake package consumerへmode・伝播・解除経路を追加した。focused tests、全体check、minimum compiler gateはWindows localで成功した。
6. [x] 実装と同じrevisionのWindows/Linux CI結果を確認し、unit台帳・本計画・`.docs/01_memory/`に記録した。run [39989485319](https://github.com/edamame-edame/freemvvm/actions/runs/39989485319) はWindows/Linux両runnerで全bootstrap gates成功。

## レビュー時に確認する選択

この計画は、同型Property間の`bind_to`、OneTime成功時の空`optional`、Binding error snapshotとerror Subscriptionを同じ公開機能としてまとめる案である。API名・戻り値の形・範囲に異論があれば、計画承認前に修正する。計画の承認と実装開始の承認は別に扱う。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | 本計画と公開API案 | 利用者 | 「ok」により計画・API案を承認。実装開始は別途承認待ち |
| 2026-10-24 | 実装開始 | 利用者 | 「進めてください」により、本計画に沿う実装を承認 |

## Windows local検証

- Visual Studio 2022 Build Tools (MSVC 19.44)で`tools/dev.py cpp-property-binding-test`成功。fake ABIで全5値型の3 mode、status/error ownership、Binding move/cancel/close、error snapshotとerror subscription callback kindを確認。
- `tools/dev.py cpp-package-test`成功。移設・別prefixのconsumerでOneTime copy、OneWay/TwoWay伝播、cancel後の停止を確認。
- `tools/dev.py check`成功。CTest 11件を含む全体gateが成功し、`wrapper.cpp.property.binding`がpass。
- `tools/dev.py minimum`成功（Rust 1.85.0）。`tools/check_docs.py`はactive documents 76、local links 6402を検査し、`git diff --check`も成功。
- package consumerは公開C++ validator APIを持たないため、遅延error state生成そのものはfake ABIで検証した。Core/C ABIの遅延validationは既存Core/ABI testsが担当する。
- CI: 実装commit `9e89dd5026ea053acd441bc6a5d30147ea992ae3` のrun [39989485319](https://github.com/edamame-edame/freemvvm/actions/runs/39989485319) はWindows/Linux両runnerで成功。
