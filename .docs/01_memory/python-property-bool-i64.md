# Python wrapper / BOOL-I64 Property implementation

- Python wrapperの値Property追加では、`bool`が`int`の派生型であるため`type(value) is bool`を先に判定し、`type(value) is int`でI64を判定してからsigned 64-bit範囲を確認する。暗黙変換やサブクラス受け入れは避ける。
- C ABIのBOOLはPython `bool`と`ctypes.c_uint8`の境界で0/1へ明示変換し、getterも0/1以外をABI内部エラーとして拒否する。
- Propertyは値をshadow cacheせず、get/setをCoreへ委譲する。RuntimeのUI-thread検査、Property owner call lease、Runtime owner lease、failed release token retryを組み合わせる。
- finalizer cleanup ticketがRuntimeをanchorとして保持するケースでは、想定外のdispose/release経路でもticketのanchorをdrop前に保存し、native tokenのcleanup経路が先に失われないようにする。
- focused fake API testsと独立Python consumerを分け、consumerは実C ABI libraryへ接続してBOOL/I64端値のroundtripと明示closeを確認する。dev.pyには専用test/consumer commandを登録し、check gateにも組み込む。
- 実装参考: `.docs/20_todo/python-property-bool-i64.md`、`.docs/32_unit_test/wrappers.md`、`UNIT-WRAP-PY-PROP-BOOL-I64-001`。2026-10-25 Windows local focused 7件、consumer、check、minimum、document-link validation成功。実装SHA `387eab6b9219a82693883b06ee8ef2da5fdf1fdd`の[Windows/Linux CI run 40197894655](https://github.com/edamame-edame/freemvvm/actions/runs/40197894655)も成功。
