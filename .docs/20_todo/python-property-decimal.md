# Python DECIMAL Property wrapper

- 計画版: 0.1
- 状態: 計画0.1承認済み（利用者、「ok」、2026-10-25）。DD改訂・試験設計・実装は未承認
- 作成日: 2026-10-25
- 対象: Python `Property` facadeへ初期DECIMAL値のcreate/get/setを追加する独立unit
- 作業branch案: `impl-python/runtime-wrapper`を継続。完了済みPython Runtime・BOOL/I64・UTF-8・F64 Propertyを基盤とし、Core/C ABIや他言語wrapperは変更しない

## 背景と次unit選定

[Python F64 Property unit](python-property-f64.md)が実装・local gates・Windows/Linux CIまで完了した。初期Property値型のうちDECIMALだけがPython facade未対応である。他wrapperと同じく、canonical uint64 value mappingとProperty create/get/setをPython向けに追加する単位とする。C ABIのDECIMAL関数、Coreの範囲・同値判定・格納表現は既に実装済みで、本unitでは変更しない。

## 上位根拠と現状

- 要求: [RQ-02 Property](../11_requirements/RQ-02_core-features.md)（RQ-02-001、RQ-02-002、RQ-02-017、RQ-02-019、RQ-02-020）、[ownership](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[threading](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)。要求本文と承認履歴をDD提案前に再確認する。
- 基本設計: [値・モデル・更新整合性](../12_design/runtime-objects/model-values.md) v1.0全文承認済み。値型の責務と同値更新の格納表現維持を継承する。
- 詳細設計: [イベントハンドリング：通知とBinding伝播](../13_detail_design/events/notifications-binding.md) v1.1承認済みのDECIMAL encodingは`bit63=sign`、`bits58..62=scale (0..31)`、`bits0..57=coefficient (0..2^58-1)`である。数値上同値のsetは格納済み表現を維持する。
- ABI: [共通C ABI](../13_detail_design/runtime-objects/abi.md)の`property_create_decimal` / `property_get_decimal` / `property_set_decimal`はcanonical `uint64_t` bitsを受渡しする。C headerにも該当関数がある。
- 他wrapper: C# `DecimalValue`とRust `Decimal64`はsign/scale/coefficientまたはcanonical bitsを型境界で検証する。ただし、それらの言語APIをPythonへそのまま移植しない。
- Python: [Python wrapper DD](../13_detail_design/wrappers/python.md) v1.5全文承認済み。DECIMALは別設計・別unitとして明示されている。次にv1.6でPython公開型・範囲変換・canonical bits mapping・error/ownership flowを提案する。
- unit-test: [wrapper試験設計](../32_unit_test/wrappers.md) v0.7全文承認済み。DECIMAL向けfake APIと独立real C ABI consumerの試験設計を別途提案し、文書全体の承認を得てから登録する。
- コーディング規則: [Python coding rule](../00_rule/coding_rule_python.md)。実装完了時にDECIMAL固有の知見を`.docs/01_memory/`へ記録する。

## 提案scope

- Python標準`decimal.Decimal`を利用する公開型候補として、`Property.create(runtime, initial)`、`get()`、`set(value)`にDECIMAL mappingを追加する案をPython DD v1.6で具体化する。別の`DecimalValue` facade型が必要か、または`decimal.Decimal`のみで十分かはDD改訂時に比較して明記する。
- Decimal tupleからsign/scale/coefficientへ厳密変換する。Python decimal contextを使った算術・量子化・丸め・binary float経由の変換を行わず、C ABIへcanonical uint64 bitsを渡す。
- RQ-02-019のscale/coefficient範囲外、非有限Decimal、型違いをどう拒否するか、末尾ゼロを除く正確な正規化が必要な場合の規則、負のzeroとscaleの返却方法をDDで明示する。実装都合だけで新しい外部保証・暗黙変換を決めない。
- BOOL/I64/F64/UTF-8と同じRuntime anchor、UI-thread check、owner/call lease、error snapshot、close/retry契約を再利用する。wrapperでdecimal equalityやnotification判定を再実装しない。

対象外: Core/C ABI変更、他言語wrapper変更、Property subscription通知の機能追加、Binding/Validator、文字列parse/format、TextBox入力・表示桁数、locale対応、arbitrary Python object、package/loader変更。

## 実施順と承認境界

1. [x] Python F64 unit完了後のbranch/worktree、要求、承認済みBD/DD、C ABI関数とCore/ABI DECIMAL unit完了状態を確認した。
2. [x] 本計画0.1のscope・順序について利用者の承認を得た（2026-10-25）。承認後に計画をcommitし、DD提案へ進む。
3. [x] 要求・既存DECIMAL設計・C#/Rust境界を照合し、Python wrapper DD v1.6へ公開Decimal型、厳密範囲写像、bits表現、失敗型、ownership/call flowを提案。利用者が文書全体を承認（2026-10-25）。
4. [ ] DD承認後、wrapper試験設計をv0.8へ改訂し、focused fake APIとreal C ABI consumerの試験ケースを提案して文書全体の承認を得る。
5. [ ] 承認済み設計・試験に基づき`UNIT-WRAP-PY-PROP-DECIMAL-001`を依存関係・要求・case・command付きで`tools/units.yaml`に登録する。登録後に実装開始を別途確認する。
6. [ ] 実装開始指示後、Python canonical Decimal mapping、facade create/get/set、focused fake tests、独立C ABI consumer、`tools/dev.py` command/check gatesを実装する。
7. [ ] Windows local focused tests/consumer/check/minimum/document validationと、実装SHAを確認するWindows/Linux CIを実行し、計画・試験記録・memory・unit台帳を完了記録して日本語commit/pushする。

## 検証と完了条件案

- Fake API: Decimal tuple to bits / bits to Decimal exact mapping、符号・scale・係数境界、型違い・non-finite・範囲外拒否、status/error snapshot、wrong-thread/closed、failed set、release retry、in-flight call/close競合を試験設計で具体化する。Core同値判定・notificationはfakeで検証したと主張しない。
- Real C ABI consumer: 別processで表現境界、負数、複数scale、正負zeroのcreate/get/set/closeを確認する。必要に応じて同値設定後のstored canonical bits維持を実Core/C ABI経由で確認する。
- local gates: dedicated fake test、real C ABI consumer、`tools/dev.py check`、`tools/dev.py minimum`、document-link validation。
- 完了条件: Windows local gates成功、実装とtestが同一revision、Windows/Linux CI成功、unit台帳state `complete`、memoryにPython固有のmapping知見を保存し日本語commit/push。

## 計画履歴

| 日付 | 対象 | 承認者 | 結論 |
| --- | --- | --- | --- |
| 2026-10-25 | 計画0.1 / Python DECIMAL Property wrapper | 利用者 | 「ok」により本計画のscope・順序を承認。DD改訂・試験設計・実装の承認を含まない |
