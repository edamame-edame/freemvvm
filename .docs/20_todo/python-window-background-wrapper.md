# Python Window背景RGB wrapper実装計画

- 計画版: 0.1
- 状態: 計画・Python DD 2.1・試験設計1.3承認済み。実装中
- 上位計画: [Window背景色の規定値・利用者設定](window-background-config.md)
- 対象: Python `Window.background_rgb` getter/setter、`reset_background_rgb()`、fake ABI・real C ABI consumer
- 対象branch: `impl-rust/core-apis-window-background`を先頭親とするPython wrapper branch
- 上位設計: [Python wrapper DD](../13_detail_design/wrappers/python.md) 2.1（承認済み）
- 試験設計: [wrapper単体試験](../32_unit_test/wrappers.md) 1.3（承認済み）
- 要求: [RQ-09-001](../11_requirements/RQ-09_ui-scope.md#rq-09-001)、[RQ-09-074](../11_requirements/RQ-09_ui-scope.md#rq-09-074)

## 目的

承認済みPython wrapper設計2.1に従い、Core/C ABIが保持するWindow background RGBをtupleとして公開する。Window/Runtime ownership、status/error変換、Window peer lifecycleは既存Python wrapper契約を再利用し、新たな状態cacheやRGB validation規則を追加しない。

## Git分岐と既存Python wrapperの取り込み案

既存の`impl-python/runtime-wrapper`は`impl-rust/core-apis-window-background`より前の共通祖先から分岐しており、Python RuntimeからWindow/Controls/Commandまでの実装・試験を保持する。別言語wrapperを親にしないという規則を守るため、次のbranchはCore API branchを先頭親として作成し、既存Python branchだけを統合する。C++/C# wrapper branchは統合しない。

統合対象はPython wrapper実装・そのPython-specific tests/examples/commands・Python unit registrationsとそれらに必要な文書に限定する。読み取り専用merge previewでは`.docs/13_detail_design/wrappers/python.md`と`tools/units.yaml`の2ファイルが競合し、`tools/dev.py`などは自動merge候補と確認した。競合箇所では両側の承認済み内容を保持し、Python設計1.2およびCore unit登録を落とさず、Window background試験設計を追記する。不要な履歴・ファイルを一括採用しない。branch候補名は`impl-python-wrapper/window-background-config`。

## 実装・検証範囲

1. `Window.background_rgb`を`tuple[float, float, float]`で読み書きし、`Window.reset_background_rgb()`を追加する。既存`_ui_call`、Window token lease、native error mappingを利用し、Core/C ABIへ委譲する。
2. fake ABI testでopaque white default、指定値get/set、reset、invalid tupleのCoreへの転送とnative status/error mappingを確認する。既存`wrappers/python/tests/test_window.py`のfake APIを拡張する。
3. real C ABI consumerでWindow create、default/set/get/reset、明示closeを行う。既存`wrappers/python/examples/window_consumer.py`と`tools/dev.py python-window-consumer-test`を拡張する。
4. wrapper test designに`UNIT-WRAP-PY-WINDOW-BG-001`、`UT-WRAP-PY-WINDOW-BG-001`、`UT-CONSUMER-PY-WINDOW-BG-001`を登録し、requirements/caseをRQ-09-001・RQ-09-074・TC-WINDOW-BG-01へ結ぶ。`tools/units.yaml`には既存Window unitとCore Window背景色unitへの依存を登録する。
5. `python-window-test`、`python-window-consumer-test`、`tools/dev.py check`、`tools/dev.py minimum`、docs link・manifest・`git diff --check`を同revisionで実行する。
6. 実装unit試験が通ったら、日本語commitと対象Python branchへのpushを同じ応答内で行う。CI結果は別途記録する。window-background-config全Todo完了前にmain向けPRは作成しない。

## 完了条件と対象外

- Python公開surfaceからdefault/set/resetを実C ABIで観測できる。
- fake ABIでnative error statusを保持し、Python wrapper側がCoreのRGB validationを置換しない。
- unit台帳・test design・実行記録・Python memory noteが実装SHAと整合する。
- Platform peerのopen/close、実機画面表示、C++/C# API変更は対象外。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-31 | 計画0.1 | 承認済み | 利用者の「ok」により計画を承認。実装開始 |
