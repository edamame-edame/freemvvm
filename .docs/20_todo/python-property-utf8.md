# Python UTF-8 Property wrapper

- 計画版: 0.1
- 状態: 完了。実装・Windows local gates・Windows/Linux CI成功
- 作成日: 2026-10-25
- 対象: 既存Python `Property` facadeへ`str`のUTF-8 create/get/setを追加する独立unit
- 作業branch案: `impl-python/runtime-wrapper`を継続する。Python RuntimeとBOOL/I64 Propertyに依存し、Core/C ABI変更や他言語wrapper差分は含めない

## 背景と次unit選定

[Python BOOL/I64 Property unit](python-property-bool-i64.md)は実装・Windows local gates・Windows/Linux CIまで完了した。次のPython Property値型としてUTF-8を追加すれば、既存C ABIのbyte-length APIをPython `str`から利用できる。F64/DECIMALや通知・Bindingとは独立して検証できる。

本unitは他言語のUTF-8 Property実装をそのままPython仕様へ転用しない。Python固有のstr encode/decode失敗、buffer再試行、公開例外境界をPython DDで提案し、文書単位の承認を得る。

## 上位根拠と未確定点

- 要求: [RQ-02 Property](../11_requirements/RQ-02_core-features.md)（RQ-02-001、RQ-02-002、RQ-02-017）、[RQ-04 ABI](../11_requirements/RQ-04_abi-boundary.md)（RQ-04-003、RQ-04-020、RQ-04-021）、[ownership](../11_requirements/RQ-05_ownership-callbacks.md)、[threading](../11_requirements/RQ-06_threading-dispatch.md)。要求本文・承認履歴を実装前に再確認する。
- Python詳細設計: [Python wrapper DD v1.4](../13_detail_design/wrappers/python.md)全文承認済み。`Property`のowner/call lease/API基盤にUTF-8 API・例外・strict conversionを追加した。
- 共通ABI: [共通C ABI DD](../13_detail_design/runtime-objects/abi.md)のUTF-8 Property create/get/set、byte length、caller-buffer query/readと`UI_BUFFER_TOO_SMALL`規則を再利用する。Core/C ABIを変更しない。
- 値・ownership: [共通object/value DD](../13_detail_design/runtime-objects/objects-values.md)のUTF-8借用・copy・buffer規則を参照する。
- 試験: [wrapper unit-test design](../32_unit_test/wrappers.md)、[unit-test strategy](../32_unit_test/unit-test-strategy.md)、[release test matrix](../30_release_test/requirement-test-matrix.md)。既存Rust/C#/C++試験はcoverage参考に限り、Pythonの試験承認を代替しない。
- コーディング規則: [Python coding rule](../00_rule/coding_rule_python.md)。
- 実装知見: [Python UTF-8 Property実装memory](../01_memory/python-property-utf8.md)。

## 提案scope

- 既存`Property.create(runtime, initial)`へbuilt-in `str`を追加し、専用UTF-8 ABI関数によるcreate/get/set/closeを提供する。
- `str`はstrict UTF-8でbyte列化し、Unicode normalization・replacement・NUL終端変換を行わず、埋め込みNULと空文字列を保持する。
- isolated surrogateなどUTF-8へstrict encodeできないPython文字列をnative call前に拒否する。公開例外/status写像はDD改訂案で提案し、承認前に確定しない。
- getterはbyte長を照会し、必要なら`UI_BUFFER_TOO_SMALL`を受けて容量を再照会・再試行し、部分文字列を公開しない。不正なnative UTF-8をreplacement decodeせず拒否する。具体例外写像はDDで確定する。
- Runtime anchor、UI-thread制約、call lease、native error snapshot、release失敗時のtoken retryはBOOL/I64 unitの契約を維持する。

対象外: F64/DECIMAL、Subscription、Binding、Command、UI Control、loader/package/PyPI変更、Core/C ABI変更、Unicode normalizationや文字列変換converter。

## 実施順と承認境界

1. [x] Python BOOL/I64 Property unit完了と現在のbranch状態を確認した。
2. [x] 本計画0.1のscope・branch・レビューgateについて利用者の「ok」による承認を得た（2026-10-25）。
3. [x] Python DD v1.4でPython `str` mapping、strict encode/decode、invalid input/output、buffer retry、error/ownership flowを具体化し、利用者の「ok」による全文承認を得た（2026-10-25）。
4. [x] DD承認後にwrapper test design 0.6へfake ABI/consumer casesを追加し、利用者の「ok」による全文承認を得た（2026-10-25）。`UNIT-WRAP-PY-PROP-UTF8-001`をunit台帳へ`ready`で登録し、実装開始は別承認とする。
5. [x] 利用者の実装開始指示後、Python ctypes bindings、Property facade、focused fake ABI tests、独立real C ABI consumer、`tools/dev.py` command/check gatesを同unitに実装した。
6. [x] Windows local focused tests/consumer/check/minimum/document validation、Windows/Linux CIを実装SHAで確認し、unit/plan/test/memoryを完了記録して日本語でcommit/pushした。

## 検証と完了条件

- Fake ABI: empty/ASCII/Japanese/supplementary/NUL文字列のbyte-length mapping、strict encode拒否、不正native UTF-8拒否、容量照会後の成長・再試行、短buffer非変更、create/get/set status/error、wrong-thread/closed Runtime、failed release retry、active call/close競合をDD承認caseの範囲で確認する。
- Real C ABI consumer: 別processから空・ASCII・日本語・補助平面・埋め込みNULをcreate/get/set/closeし、長さと値が往復することを確認する。
- UTF-8以外の値種別やSubscription/Bindingの意味論を検証済みと主張しない。
- local gates: dedicated fake test、real consumer、`tools/dev.py check`、`tools/dev.py minimum`、document link validation。
- 実装commit後にpushし、Windows/Linux CIが同じ実装SHAで成功したことを記録する。

## 計画履歴

| 日付 | 対象 | 承認者 | 結論 |
| --- | --- | --- | --- |
| 2026-10-25 | 計画0.1 / Python UTF-8 Property wrapper | — | 次unit候補の計画案。利用者レビュー待ち。DD改訂・試験設計・実装の承認を含まない |
| 2026-10-25 | 計画0.1全文 | 利用者 | 「ok」によりscope・branch・段階別レビューgateを承認。DD改訂・試験設計・実装開始は別承認 |
| 2026-10-25 | Python wrapper DD v1.4全文 | 利用者 | 「ok」によりUTF-8 Property API・変換・buffer retry・失敗境界を含む文書全体を承認。試験設計・実装開始は別判断 |
| 2026-10-25 | wrapper unit-test design 0.6全文 | 利用者 | 「ok」によりPython UTF-8 Propertyのfake ABI/real C ABI consumer試験設計を承認。実装開始・試験合格は別判断 |
| 2026-10-25 | 実装開始 | 利用者 | 「ok」により承認済みDD v1.4・試験設計0.6に基づく実装開始を指示 |
| 2026-10-25 | Windows local実行 | — | fake ABI 14件、実C ABI consumer、全体`check`、`minimum`、document-link validationがsuccess |
| 2026-10-25 | implementation SHA `c3fd6ca6d1fa01613756a1e0b835253c50675e95`のCI | — | [Windows/Linux CI run 40200059368](https://github.com/edamame-edame/freemvvm/actions/runs/40200059368) success |
