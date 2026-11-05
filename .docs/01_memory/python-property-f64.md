# [Python wrapper / F64 Property]

- `Property.create`はexact typeでbool/int/float/strのkindを選ぶ。bool/int/strは有効な別kindなので、F64 createの入力拒否に使わない。F64 `set`では`type(value) is float`のみ受け付け、bool/int/subclassをnative call前に拒否する。
- C ABIのF64 create/get/setは`ctypes.c_double`のargtypes/out valueを使う。Python built-in floatをそのまま渡し、wrapperでfinite制限、演算、比較、NaN正規化を加えない。
- F64 getはstatusだけを返す既存ABI契約を保ち、失敗をraw `UiError`にする。create/setはerror tokenを既存error snapshot経路へ渡す。owner cell・Runtime anchor・UI-thread check・close retry・call leaseは共有Property経路を再利用する。
- fake API試験は値mapping、入力型境界、status/error、wrong-thread/closed、failed set、partial create cleanup、release retry、in-flight call/closeを確認する。実C ABI consumerはfinite/NaN/±infinity/±zeroのcreate/getとsetter往復を別processで確認する。Core同値・notificationはfake consumerで主張しない。
- Python Propertyの型集合を増やすときは、旧BOOL/I64のinvalid-create testsが新しい有効型を誤って拒否対象に含んでいないか確認する。focused testsとconsumerを`tools/dev.py`の個別commandおよび`check`へ登録する。
- 実装・検証: [Python F64 Property計画](../20_todo/python-property-f64.md)、[wrapper test design](../32_unit_test/wrappers.md)、`UNIT-WRAP-PY-PROP-F64-001`。Windows local focused 16件、実C ABI consumer、`tools/dev.py check`、`minimum` success。実装SHA `1c38de364c1824793db53f758cebf7e160463865`で[Windows/Linux CI run 40202740672](https://github.com/edamame-edame/freemvvm/actions/runs/40202740672) success。
