# [Python wrapper / DECIMAL Property]

- 承認済みPython設計は`.docs/13_detail_design/wrappers/python.md` 1.6「DECIMAL Property公開API」。公開値型は標準`decimal.Decimal`で、専用wrapper型・decimal arithmetic/context依存変換は追加しない。
- 入力はbuilt-in `Decimal` exact typeのみ。`as_tuple()`のsign/digits/exponentからbit63=sign、bits58..62=scale、bits0..57=coefficientへ変換する。有限値以外は拒否し、型違い/非有限/範囲外はnative call前にそれぞれ`TypeError`/`ValueError`/`OverflowError`とする設計。
- scale/係数が範囲外のときだけ、scaleが正で係数末尾が10の間、同時に係数とscaleを10で縮約する。これは数値を変えない。丸め・切捨て・quantizeは禁止。大きな正指数は桁数判定を先に行い巨大整数を作らない。
- getterはbitsから`Decimal((sign, digits, -scale))`を作り、格納scaleと負のzeroを保つ。数値同値判定・Validator・通知・同値更新時の表現保持はCoreへ委譲し、wrapperはsetterを短絡しない。
- DECIMAL Property試験設計v0.8は承認済み。`UNIT-WRAP-PY-PROP-DECIMAL-001`はplanned登録し、fake APIでPython bits mapping・型/範囲/例外・error/lifecycleを検証、別processのreal C ABI consumerで境界値と同値設定後のstored representationを検証する。Core意味論をfakeだけで検証したと主張しない。
- 設計根拠: [Python DECIMAL Property計画](../20_todo/python-property-decimal.md)、[Python wrapper DD](../13_detail_design/wrappers/python.md)、[Python wrapper試験設計](../32_unit_test/wrappers.md)。
