# カテゴリ: freemvvm Platform backend計画・実装前提

- 0.0.1初期Platform backendの計画は`.docs/20_todo/platform-first-backend.md`。対象は承認済みRQ-11のLinux DRM/KMS + EGL/OpenGL ES初期経路。別作業のRust inner WebEngine branchはmergeしない。
- `impl-rust/core-ui-controls-c-abi`には`WindowPeerAdapter`のprivate open/close seamとfake peer testsがあるが、production peer factory/Runtime接続はない。`window_install_test_peer`はtest-onlyで、peer未設定時の`window_open`はUnsupported。
- Platform DD 1.2（Linux Runtime/Window接続契約）とUI unit-test design 0.4は2026-10-26に文書全体承認済み。`UNIT-CORE-073`（factory/Window peer）、`UNIT-CORE-074`（Scene/frame）、`UNIT-PLATFORM-001`（Linux backend）が`tools/units.yaml`へplanned登録済みで、実装開始は別途指示を要する。
- 承認前のCore sourceにはSceneSnapshot/production present path、Runtime `UiLoopBridge`とのDRM event/frame completion統合、Runtime profile/factory/device失敗契約がなかった。承認済みprofile/backend選択を変えず、DD 1.2へ接続契約を具体化した。plannedのsource/crate/commandは実装開始時に現状と照合し更新する。
- RQ-11 v0.5の初期profileはWindows 11 x64、Debian 12 x64、ARMv7/AArch64 cross-build、代表RPi 4 AArch64。DRM/KMS + EGL/OpenGL ESはRQ-11-014が最初に検証する描画backend。build・CI・実機確認の証拠は分ける。
