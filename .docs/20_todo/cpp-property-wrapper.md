# C++ Property wrapper 初回実装（BOOL / I64）

- 更新日: 2026-10-22
- 状態: 計画承認済み（利用者、2026-10-22）。計画コミット後にtest登録・実装へ進む
- 対象: 承認済みC++ wrapper設計 v1.2 の `Property<T>` create/get/set の初回実装

## 背景

[C++ Runtime wrapper](cpp-wrapper-runtime-api.md) と relocatable CMake package は実装済みで、CI run [39796877396](https://github.com/edamame-edame/freemvvm/actions/runs/39796877396) はWindows/Linuxの全gateに成功した。次のC++機能として、承認済み[C++ wrapper詳細設計](../13_detail_design/wrappers/cpp.md) v1.2の `Property<T>::get/set` を実装する。

[RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)は、Coreが所有するPropertyの値・種別・変更観測を要求する。[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012) と [RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001) の所有失敗保持とUI-thread境界を維持する。Core/C ABI側にはBOOL/I64の生成・取得・設定APIがあり、この作業では新しいC ABI symbolを追加しない。

## 承認済み根拠

- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreが値の正本となり、C++はversioned C ABIを介する薄いwrapper。
- 基本設計: [値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)。Property値の確定と通知・Binding配信を区別する。
- 詳細設計: [C++ wrapper](../13_detail_design/wrappers/cpp.md) v1.2全文承認済み。`Property<T>::get/set`はC++17 `Result<T>`/`Result<void>`、UI-thread操作、失敗時にcommitしない。
- C ABI: [共通ABI](../13_detail_design/runtime-objects/abi.md) v1.2。BOOL/I64のcreate/get/set、status、owned error、kind/thread validation契約を使用する。
- テスト規約: [wrapper test設計](../32_unit_test/wrappers.md)、[unit test strategy](../32_unit_test/unit-test-strategy.md)、実装単位とtestを同じrevisionで検証する。

## この案でレビューするAPI境界

C++ DDは `Property<T>::get/set` と値型mappingを定めているが、PropertyをRuntimeから作るC++ factory名をまだ特定していない。この計画では次を提案する。

```cpp
auto property = freemvvm::Property<std::int64_t>::create(runtime, initial_value);
auto value = property.value()->get();
auto updated = property.value()->set(next_value);
```

`Property<T>::create(Runtime&, T)` はC ABIの `property_create_bool/i64` を呼び、返すPropertyはnative tokenを一つ所有する。PropertyはRuntimeのC++ objectをborrowしたままにせず、既存Runtime cleanup stateを共有して、Runtime objectの破棄後もtoken解放を管理できるものとする。通常操作は非throw `Result` 経路とし、boolはABIの`ui_bool`と明示変換、I64は`std::int64_t`とする。

この初回範囲はBOOL/I64に限定する。UTF8のbuffer容量照会、F64の特殊値境界、DECIMALの専用型/符号化は後続単位とし、OBJECT/Binding/Subscription/Property change callbackも含めない。

## 実施手順案

1. 利用者レビュー後、本計画を承認済みとして記録する。先行するC++ Runtime計画のworkflow結果記述も成功runへ揃える。
2. `.docs/32_unit_test/wrappers.md` にcreate/get/set、move/close、native error、wrong-thread、失敗時状態保持を検証する `UT-WRAP-CPP-PROP-001` を追加する。`tools/units.yaml` に `UNIT-WRAP-CPP-002` と実装source/test mappingを登録してからsourceを変更する。
3. 提案した `Property<T>::create(Runtime&, initial)`、`get()`、`set()`をBOOL/I64に実装する。token/nodeはnative call前に準備し、close失敗時は所有tokenを保持する。ABI/UI thread/status/error契約とRuntime cleanup supervisorを再実装しない。
4. fake C ABIによるfocused unit testとrelocated CMake package consumerにBOOL/I64 Propertyの一往復を追加する。
5. focused tests、workspace Rust tests、`tools/dev.py check`、`tools/dev.py minimum`、文書検査を行い、同じsource revisionのWindows/Linux CI成功を確認する。
6. 実装で分かったC++ wrapperの所有・検証方法を `.docs/01_memory/` の適切なscope別文書へ記録し、本計画へ結果を追記する。

## 承認境界

この計画と `Property<T>::create(Runtime&, T)` API案は2026-10-22に利用者が承認した。計画をcommit/pushした後にtest registrationと実装を開始する。
