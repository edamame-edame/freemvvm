# Python wrapper / Runtime facade / lifecycle

- Python Runtime wrapper unitは、公開Runtime owner、UI-thread context、shutdown request/receiptを一体で扱い、Property/UI facadeは別unitに分ける。
- ctypesの宣言とABI library loadingは`freemvvm._native`に閉じ、fake ABIの注入点はprivate seamにする。製品loaderへtest用path overrideを追加しない。
- `UI_PENDING`はshutdown要求の受付であり完了ではない。receiptのraw stateを保持して未知状態を誤分類せず、明示的なRuntime closeとshutdown要求を分離する。
- shutdown bridge未登録のreal C ABI consumerでは`UI_UNSUPPORTED`になり得る。UI event loopがない試験でshutdown受理・完了を期待しない。
- native close失敗時はowner/tokenを保全してretry可能にし、in-flight native callとcloseをcall leaseで直列化する。finalizerはblocking native cleanupを避け、owner cleanup ticketを安全な入口で再試行する。
- このunitの確認コマンドは`.venv/Scripts/python.exe tools/dev.py python-runtime-test`、`python-runtime-consumer-test`、全体`check`、`minimum`。結果は実行revision/CIと分けて記録する。
- 2026-10-25のcommit `bdaf48e`でfocused 9件、real C ABI consumer、全体check/minimum、およびWindows/Linux CI run 40190078806が成功。CI記録は`.docs/20_todo/python-runtime-wrapper.md`と`.docs/32_unit_test/wrappers.md`を参照する。

## Scope: Python Window background RGB facade

- `Window.background_rgb`はCore/C ABI getter/setterへ委譲する3-float tuple。resetは`Window.reset_background_rgb()`。Python側にRGB state cacheやrange/finite validationを持たせず、native status/errorを既存`UiError`へ渡す。
- `_native.CtypesNativeApi`のRGB outputsは`ctypes.c_float`で受ける。consumer比較はABI float32丸めを考慮し`math.isclose`を使う。
- fake ABIのWindow focused suite入口は`.venv/Scripts/python.exe tools/dev.py python-window-test`、real library consumerは`python-window-consumer-test`。実peer表示は別のPlatform確認である。
