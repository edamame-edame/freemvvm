# Memory category: Python Window presentation backend selector

- 更新: 2026-11-10
- スコープ: Python Windowの描画backend選択wrapper
- 参照先: [backend selector implementation plan](../20_todo/backend-selector-implementation.md)、[Python wrapper DD](../13_detail_design/wrappers/python.md)、[UI controls DD](../13_detail_design/ui/controls.md)
- API: `Window.create(runtime, *, presentation_backend=None)` は未指定なら従来の `window_create` を呼び、文字列指定なら厳密UTF-8 bytesを保持して同期的にC ABI selectorへ渡す。Python側に可変backend stateや独自loader/plugin方式は加えない。
- 検証: fake APIでUTF-8伝達、入力型/不正surrogateのnative call前拒否、native error mappingを確認する。real consumerはplatform IDの明示選択とunknown ID拒否を確認し、`Window.open()`は呼ばない。Windows/Ubuntu bootstrap CIでpytest、package wheel build/install/importを含む全体gateを確認する。
- 2026-11-10の実装は `41fff6e65b4899d3d39650413e7565110169e688`、ruff整形修正は `02b83cfa3dc3b8282ecfc3ade747486ee21b2d23`。CI run `42823301385` はWindows/Ubuntu両方成功。ローカルPython 3.15にはpytest/ruffがなく、ローカルfocused pytestは未実行。構文・文書リンク・catalog、Windows native DLL consumerはローカルで成功した。
