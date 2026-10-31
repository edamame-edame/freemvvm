# Python BOOL/I64 Property wrapper

- 計画版: 0.1
- 状態: 完了（実装・Windows local gates・Windows/Linux CI成功、2026-10-25）
- 作成日: 2026-10-25
- 対象: Runtime wrapperを基礎に、PythonからBOOLとsigned I64のPropertyをcreate/get/set/closeできる最初の値Property unit
- 作業branch案: `impl-python/runtime-wrapper`を継続する。現在のPython Runtime実装に依存し、Core/API変更や他言語wrapper差分は含まない。

## 背景

[Python Runtime wrapper unit](python-runtime-wrapper.md)がcommit `bdaf48e`で実装され、Windows/Linux CI run 40190078806まで成功している。Python packageには現在Property facadeがなく、利用者はRuntime上に値を作成・読み書きできない。0.0.1計画では4 wrapperから同じCoreのProperty/Bindingを使えることを目指すが、値型PropertyとBindingは独立して検証できる単位に分ける。

本計画はC#の[BOOL/I64 Property unit](csharp-property-bool-i64.md)を実装根拠として転用するものではない。Python公開surface、type mapping、例外、UI-contextの適用はPython DDで新たに提案し、利用者承認を得る。

## 承認済み上位根拠

- 要求: [RQ-02 Object/Property](../11_requirements/RQ-02_core-features.md)（RQ-02-001、RQ-02-002、RQ-02-017）、[ownership](../11_requirements/RQ-05_ownership-callbacks.md)、[threading](../11_requirements/RQ-06_threading-dispatch.md)。
- Python詳細設計: [Python wrapper DD v1.2](../13_detail_design/wrappers/python.md)（Runtime公開API、owner/call lease/cleanup契約）。本版はRuntime APIに限定し、Property公開signatureを定義していない。
- ABI根拠: [共通C ABI DD](../13_detail_design/runtime-objects/abi.md)のProperty create/get/setおよびhandle release。ABI側に不足・不整合があれば、差分を明示して実装を止め、Core変更を本unitへ混ぜない。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[release test matrix](../30_release_test/requirement-test-matrix.md)。

## 対象範囲案

- PythonのBOOLとsigned I64 Property create/get/set/明示close。
- Python `bool`と`int`の区別、I64範囲検査、ABI bool値0/1の変換規則をDD改訂案として定義する。
- Runtime wrapperのowner cell/call lease/error snapshot/cleanup retryを再利用し、native callとcloseの競合を安全に直列化する。
- Property操作で必要なUI thread/Runtime ownership checkを明示する。具体的にRuntime引数、UiContext、Runtime.ui_context()のどれを公開APIへ使うかはDD案で提示しレビュー対象とする。
- Fake NativeApi focused testと別process real C ABI consumerでBOOL/I64値を往復確認する。

対象外: UTF-8/F64/Decimal Property、Subscription、Binding、Command、UI Control、callback追加、package/loader変更、release/tag。本unitでは未対応値型を暗黙変換しないが、公開APIでの拒否方法は設計レビューで確定する。

## 公開API案（レビュー対象・未確定）

第一案はPythonらしいfactoryを用いる `Property.create(runtime, initial)`, `get()`, `set(value)`, `close()/try_close()` とcontext manager。初回対応値は`bool`とI64範囲内の`int`のみとし、`bool`が`int`の派生型である点を型判定で明示する。別案として型別factoryまたは`Property[bool]`型パラメータを採用するかをDD全体案に含め、利用者レビューで確定する。

API配置・例外class・unsupported typeの返し方・UI thread enforcementは、承認済みRQ/ABIの意味を変えずに提案する。Property wrapperに値cacheを置かず、Core Propertyの値と検証結果を正本とする。

## 試験案

計画とDD/test designが承認された後、ソース変更前に`.docs/32_unit_test/wrappers.md`全体の改訂案を提示して承認を得てからcaseを登録する。

- Fake ABI: create成功/失敗、bool 0/1と不正bool output、I64 min/max・範囲外、wrong-thread/closed Runtime、get/set raw statusとerror snapshot、failed set時にCore値を書き換えない委譲、failed close後のtoken保全/retry、active call中のclose待機。
- Real C ABI consumer: Runtime上で`False/True`とI64の0・端値をcreate/get/set/closeし、値を往復する。UI event loopがないhostで保証外のshutdown完了をassertしない。
- test designが要求していない通知・Binding・Property相互作用をこのunitで主張しない。
- local gates: focused test、real consumer、`tools/dev.py check`、`tools/dev.py minimum`、document link validation。実装revisionをpushした後、Windows/Linux CIが成功した同一SHAを記録する。

## 実施順と承認境界

1. [x] Runtime wrapper実装・CI成功と現在のPython branch状態を確認した。
2. [x] 本計画全文を提示し、scope/API案/branch案について利用者承認を得た（2026-10-25、「ok」）。
3. [x] Python wrapper DD v1.3全文にBOOL/I64 Property API・class/ownership/call flow/error/type mappingを提案し、利用者の「ok」（2026-10-25）で文書全体の承認を得た。実装・試験合格は別管理。
4. [x] wrapper unit-test文書0.5全文の利用者承認（2026-10-25、「ok」）を得た。`UNIT-WRAP-PY-PROP-BOOL-I64-001`を`tools/units.yaml`へ`ready`で登録した。試験実装・実行は未着手。
5. [x] 利用者の実装開始指示後、Python BOOL/I64 Property facade、ctypes ABI bindings、focused tests、real C ABI consumer、専用dev commandsを同じ実装unitへ追加した。承認済み契約から外れる未決事項なし。
6. [x] Windows local gatesとWindows/Linux CIが実装SHA `387eab6b9219a82693883b06ee8ef2da5fdf1fdd` で成功。unit/plan/test/memoryへ結果を記録する。

## 完了条件案

- Python DDとwrapper test designの全文が別々に承認済み。
- BOOLとI64の公開Property操作、所有、UI-thread、型/失敗境界をfake testで確認。
- consumerから実C ABIのBOOL/I64 Property roundtripとcloseを確認。
- 対応unit、全体check/minimum、Windows/Linux CIが同じ実装revisionで成功し、記録済み。
- UTF-8/F64/Decimal、Binding、Subscription、UI機能を実装済みと誤認させない。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-25 | 計画0.1 / Python BOOL-I64 Property | 利用者 | 「ok」によりscope/API案/branch案を承認。test文書の承認および実装開始を意味しない |
| 2026-10-25 | Python wrapper DD v1.3全文 | 利用者 | 「ok」によりBOOL/I64 Property公開API・型mapping・ownership・call flowを承認。実装・試験合格は別管理 |
| 2026-10-25 | wrapper unit-test文書0.5全文 | 利用者 | 「ok」によりfake ABI/real C ABI consumer試験設計を承認。実装開始・試験合格は別判断 |
