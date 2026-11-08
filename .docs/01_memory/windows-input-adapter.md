# freemvvm: Windows Platform Adapter / native input bridge

- カテゴリ: 実装境界 / Windows Platform Adapter / Input bridge
- 対象: `crates/platform-windows/src/input.rs`、`crates/platform-windows/src/win32.rs`

## 学んだこと

- Win32 native messageはWindows worker上でCore-neutral owned eventへ写し、同一Window generationのinput sinkへpostする。WndProcはCore InputRouterやuser callbackを直接呼ばない。
- Keyboard identityはscan codeとextended bitからHID Keyboard/Keypad usageへ写し、Unicode textはmessage pumpが生成する`WM_CHAR`をUTF-16 surrogate pair処理後にUTF-8化する。system keyのnative default processingは維持する。
- Mouseのsigned client pixelとTouch/PenのScreenToClient後pixelは、Window open時に得たgeneration DeviceScaleで一度だけDIPへ変換する。PT_MOUSEをpointer pathへ重複計上しない。
- Windows target build/testはWin32 APIの型・featureとplatform-gated codeを検証する。OS IME、入力機器、実画面の挙動は別途実機確認が必要。

参照: [Windows入力bridge計画](../20_todo/windows-platform-input-bridge.md)、[Platform内部レイヤー設計](../13_detail_design/platform/internal-layers.md)、[UI単体試験](../32_unit_test/ui.md)。
