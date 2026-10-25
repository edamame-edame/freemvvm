# Python F64 Property wrapper

- 計画版: 0.1
- 状態: 実装済み。Windows local gates成功。Windows/Linux CI確認待ち
- 作成日: 2026-10-25
- 対象: Python `Property` facadeへbuilt-in `float`のF64 create/get/setを追加する独立unit
- 作業branch案: `impl-python/runtime-wrapper`を継続。Python Runtime、BOOL/I64、UTF-8 Propertyを基盤とし、Core/C ABIや他言語wrapperは変更しない

## 背景と次unit選定

[Python UTF-8 Property unit](python-property-utf8.md)が実装・local gates・Windows/Linux CIまで完了した。次の値型候補は、承認済みRQ-02の初期Property型で未対応のF64とDECIMALである。F64は既存C ABIに`double`のcreate/get/setがあり、数値変換を追加せずPython binary64へ写す小さなscopeとして先行できる。DECIMALは独自canonical encodingとPython側の型/API選定があるため独立unitに分ける。

## 上位根拠と現状

- 要求: [RQ-02 Property](../11_requirements/RQ-02_core-features.md)（RQ-02-001、RQ-02-002、RQ-02-017、RQ-02-018）、[ownership](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[threading](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)。要求本文・承認履歴をDD改訂前に確認する。
- 上位設計: [共通C ABI DD](../13_detail_design/runtime-objects/abi.md) v1.3と[共通object/value DD](../13_detail_design/runtime-objects/objects-values.md) v1.3は承認済み。F64はC `double`で受渡しし、NaN・infinityを許可する。NaN同士、同符号infinity、正負zeroはCore上同値であり、同値再設定は既存格納表現を保ち通知しない。
- ABI実装: `abi.h`とC ABIに`property_create_f64` / `property_get_f64` / `property_set_f64`が既にある。本unitではCore/C ABIを変更しない。
- Python: [Python wrapper DD v1.4](../13_detail_design/wrappers/python.md)は承認済みでBOOL/I64/UTF-8を定義するが、F64の公開型mappingは未記載。まずDD v1.5でPython `float`受入れ、binary64境界、失敗写像、所有/API flowを提案し、文書全体の承認を得る。
- 試験: [wrapper unit-test design](../32_unit_test/wrappers.md)、[unit-test strategy](../32_unit_test/unit-test-strategy.md)、[release test matrix](../30_release_test/requirement-test-matrix.md)。Pythonのfake API/consumer試験を別途設計し、文書全体の承認後に実装する。
- コーディング規則: [Python coding rule](../00_rule/coding_rule_python.md)。実装知見は完了時にPython F64用memoryへまとめる。

## 提案scope

- `Property.create(runtime, initial)`と`set(value)`へbuilt-in `float`を追加する。exact type判定と型間誤入力の拒否はPython DDで明記し、int/bool/strからfloatへの暗黙変換は追加しない。
- Python floatとC ABI `double`の間に丸め・finite-only制約・NaN/infinity正規化を加えず、値を直接渡す。NaN payloadやsigned zeroの保持に関するPython/ctypes境界の実際の保証をDD改訂で確認し、未保証事項を推測で断定しない。
- get/set/createの既存UiError/error snapshot、Runtime anchor、UI-thread検査、両owner call lease、close/retry契約を再利用する。wrapperに値cacheやF64同値判定を持たせない。
- Coreの同値判定・格納表現・通知結果は再実装せず、RQ/Core/C ABIと既存C ABI testsへ委譲する。

対象外: DECIMAL/Decimal64、Subscription、Binding、Validator、UI Control、Core/C ABI変更、format/parse/converter、package/loader変更。

## 実施順と承認境界

1. [x] Python UTF-8 Property unit完了、作業branch、要求/上位設計/C ABI現状を確認した。
2. [x] 本計画0.1のscope・順序について利用者の「ok」による承認を得た（2026-10-25）。
3. [x] 要求承認履歴とPython/ABI/value設計の範囲を再確認し、Python DD v1.5へF64 API・型境界・binary64・error/ownership flowを提案した。利用者の「ok」によりDD全文が承認された（2026-10-25）。
4. [x] DD承認後、wrapper unit-test design 0.7へfocused fake API・real C ABI consumer casesを提案し、利用者の「ok」により試験設計全文の承認を得た（2026-10-25）。
5. [x] Implementation unit `UNIT-WRAP-PY-PROP-F64-001`を依存関係付きで`tools/units.yaml`へ登録した。実装開始は別判断とする。
6. [x] 実装開始指示後、ctypes bindings、Property facade、focused fake tests、独立C ABI consumer、`tools/dev.py` command/check gatesを実装した。
7. [ ] Windows local focused tests 16件/consumer/check/minimum/document validationは成功。実装commitをpush後、同一SHAのWindows/Linux CIを確認してplan/test/memory/unit台帳を完了記録する。

## 検証と完了条件案

- Fake API: built-in floatのみ受入れ、bool/int/subclass拒否、finite・NaN・infinity・±0のmapping、create/get/set status/error snapshot、wrong-thread、closed、failed set、release retry、in-flight call/close競合を検証する。F64同値判定やCore通知をfake APIで検証したと主張しない。
- Real C ABI consumer: 別processでfinite値、NaN、±infinity、±0をcreate/get/set/closeする。NaN payloadやsigned-zero bit保持は承認されたPython DDとC ABI observable contractの範囲内で比較する。
- local gates: dedicated fake test、real consumer、`tools/dev.py check`、`tools/dev.py minimum`、document link validation。
- 実装commit後にpushし、Windows/Linux CIが同じ実装SHAで成功すること。

## 計画履歴

| 日付 | 対象 | 承認者 | 結論 |
| --- | --- | --- | --- |
| 2026-10-25 | 計画0.1 / Python F64 Property wrapper | — | 次unit提案。利用者レビュー待ち。DD改訂・試験設計・実装の承認を含まない |
| 2026-10-25 | 計画0.1全文 | 利用者 | 「ok」によりscope・順序を承認。DD改訂・試験設計・実装開始は別承認 |
| 2026-10-25 | Python wrapper DD v1.5全文 | 利用者 | 「ok」によりF64 Property API・型境界・Core委譲・error/ownership flowを含む全文を承認。試験設計・実装は別承認 |
| 2026-10-25 | Python wrapper試験設計 v0.7全文 | 利用者 | 「ok」によりfake API / real C ABI consumerのF64 Property試験設計を含む全文を承認。unit登録を実施。実装開始は別承認 |
| 2026-10-25 | Python F64 Property実装 | — | `UNIT-WRAP-PY-PROP-F64-001`実装とWindows local focused 16件・consumer・check・minimum成功。Windows/Linux CI待ち |
