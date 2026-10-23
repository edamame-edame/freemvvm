# C++ Property wrapper F64 value

- 更新日: 2026-10-23
- 状態: 利用者承認済み。実装前
- 対象: `freemvvm::Property<double>` のcreate/get/set

## 背景と承認済み根拠

C++ Property の [BOOL/I64対応](cpp-property-wrapper.md) と [UTF-8対応](cpp-property-utf8.md) が完了した。次の独立実装単位として、承認済み[C++ wrapper DD v1.2](../13_detail_design/wrappers/cpp.md)のF64 mappingと、承認済み[共通C ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)のF64 Property関数をC++へ接続する。

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreを値の正本とし、C++はversioned C ABIを介する薄いwrapper。
- 基本設計: [値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)。F64のNaN・infinity・signed zeroの同値規則と、同値設定時の格納表現保持を規定する。
- C++ DD: F64はC++ binary64対応型へ明示mappingし、wrapperで値意味論を変えない。
- ABI DD: F64 Property create/get/setは`double`を値渡しし、getterは現在の確定値を返す。create/set失敗はABIのstatus/error handle契約に従い、getはraw statusを返す。

## 実装案

```cpp
auto property = freemvvm::Property<double>::create(runtime, initial_value);
auto value = property.value()->get();
auto updated = property.value()->set(next_value);
```

既存`Property<T>`のowner・cleanup node・move・close・supervisor経路を再利用し、`property_create_f64/get_f64/set_f64`を`double`で呼び出す。bool、I64、UTF-8の変換経路は維持し、型間の暗黙変換は追加しない。

WrapperはNaNやinfinityを拒否・正規化せず、signed zeroの符号を含めてC ABIへ渡す。Coreが規定する同値判定、同値時の格納表現保持、通知抑止はC++ wrapperで再実装しない。GetterはC ABIの戻り値を加工せず返し、getter errorにはerror snapshotを付けない。create/setのnative error snapshotとraw status、所有tokenのclose/retryは既存共通経路を維持する。

## 試験案

- `UNIT-WRAP-CPP-004` / `UT-WRAP-CPP-PROP-F64-001` をsource変更前に`.docs/32_unit_test/wrappers.md`と`tools/units.yaml`へ登録する。
- fake ABIでProperty<double>のcreate/get/set、有限値、正負infinity、複数のNaN bit pattern、`+0.0`と`-0.0`のbit-preserving引渡し・取得を検証する。NaN値自体の比較には`==`を使わず、bit patternまたは`std::isnan`で観測する。
- create/setのnative失敗とerror snapshot、getterのwrong-thread raw status、失敗set後も値が変化しないことを確認する。
- move、明示close失敗後のretry、destructor release失敗後のcleanup supervisor再試行を確認する。
- `UT-PKG-CPP-004`をrelocated CMake package consumerへ追加し、別consumerからProperty<double>を作成・取得・設定・closeする。
- focused tests、C++ Runtime/既存Property tests、package consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書検査とWindows/Linux CIを同じ実装revisionで確認する。

## 実施手順

1. 利用者レビュー・承認後、この計画を日本語commitで先にcommit/pushする。
2. unit-test設計と実装unit mappingをsource変更前に登録する。
3. F64 create/get/setのC++接続、fake ABI試験、relocated package consumerを実装する。
4. focusedおよび全体検証を行い、Windows/Linux CI evidenceを本書と試験台帳に記録する。
5. F64 wrapperのbit-preserving境界とCore/ABI責務分担を`.docs/01_memory/`へ記録する。

## 承認境界

利用者は2026-10-23に「ok」と承認した。承認対象は`Property<double>` API、F64値を変換・正規化せずC ABIへ渡す方針、上記test範囲である。計画を先にcommit/pushし、その後にtest台帳登録とsource変更を開始する。
