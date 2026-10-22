# C++ Property wrapper UTF-8 value

- 更新日: 2026-10-23
- 状態: 計画案・利用者レビュー待ち。未承認・未commit
- 対象: `freemvvm::Property<std::string>` のcreate/get/setとUTF-8 byte境界

## 背景と承認済み根拠

BOOL/I64の [C++ Property実装](cpp-property-wrapper.md) は完了した。次の独立単位として、承認済み[C++ wrapper DD v1.2](../13_detail_design/wrappers/cpp.md)のUTF8 mappingと[共通C ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)のcaller-buffer getterを実装する。

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreとversioned C ABIを正本とし、C++は薄いwrapper。
- 基本設計: [値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)。Propertyの確定値と変更通知を分ける。
- C++ DD: UTF8は指定byte lengthを保つ`std::string` value。入力はcall中借用し、Coreが保持時にcopyする。埋め込みNULを許し、終端NULに依存しない。
- ABI DD: `(nullptr, 0)`の容量照会、`UI_BUFFER_TOO_SMALL`で出力bufferを書き換えず必要byte数を返す規則、再照会・再試行を使う。Getterはerror tokenを返さずraw statusを返す。

## 実装案

```cpp
auto property = freemvvm::Property<std::string>::create(runtime, initial_utf8);
auto value = property.value()->get();
auto updated = property.value()->set(next_utf8);
```

既存`Property<T>`のowner、cleanup node、move、close、supervisor経路を再利用し、`property_create_utf8/get_utf8/set_utf8`へ接続する。C++側の値はUTF-8 byte列を保持する`std::string`とし、Unicode正規化・置換・NUL終端変換を加えない。不正UTF-8はCoreへ渡してABIの失敗status/error snapshotを保持する。

`get()`はUI thread上で容量照会し、必要長だけ`std::string`領域を確保してcopyする。間に値が変わって`UI_BUFFER_TOO_SMALL`となった場合は、返された現在長をもとに容量照会からやり直す。各C ABI callの間にイベントループを回さず、buffer不足時は部分出力を利用者へ返さない。wrapper自身のmemory確保失敗は`UI_OUT_OF_MEMORY`として返し、native error snapshotを捏造しない。

## 試験案

- `UNIT-WRAP-CPP-003` / `UT-WRAP-CPP-PROP-UTF8-001` をsource変更前に`.docs/32_unit_test/wrappers.md`と`tools/units.yaml`へ登録する。
- fake ABIで空文字列、ASCII、複数byte日本語、埋め込みNUL、create/get/set、move/closeを確認する。
- invalid UTF-8のcreate/setでraw statusとnative error snapshotを保持し、失敗setが既存値を変えないことを確認する。
- getterの容量照会後に必要長が増えるケースを注入し、`UI_BUFFER_TOO_SMALL`後に再照会・再試行して完全な値を返すこと、およびbuffer不足時に部分値を公開しないことを確認する。
- wrong-thread status、explicit close失敗後のretry、destructor release失敗のsupervisor handoffを確認する。
- `UT-PKG-CPP-003`を既存のrelocated CMake package consumerへ追加し、外部consumerで`Property<std::string>`が使えることを確認する。
- focused test、既存C++ Runtime/Property試験、package consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書検査とWindows/Linux CIを同じsource revisionで確認する。

## 実施手順

1. 利用者レビューと承認を受け、この計画を日本語commitで先にcommit/pushする。
2. unit-test設計と実装unit mappingをsource変更前に登録する。
3. UTF8 create/get/setを実装し、fake ABIとrelocated package consumerを追加する。
4. focusedおよび全体検証を行い、Windows/Linux CI evidenceを本書へ記録する。
5. C++ wrapperのUTF-8 byte・buffer retry規則を`.docs/01_memory/`へ追記し、完了結果を記録する。

## 承認境界

この文書はレビュー案であり、承認前は計画のcommit/push、source変更、test台帳登録を行わない。承認対象は`Property<std::string>` API、invalid UTF-8をCoreへ渡す方針、容量不足時の再照会・再試行、上記test範囲である。
