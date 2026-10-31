# C++ wrapperの学び: Window safe ownerとCore背景RGB

- 対象: freemvvm C++ wrapperにおけるWindow handle owner、Window background RGB、C ABI接続、検証。
- 関連設計: [C++ wrapper DD](../13_detail_design/wrappers/cpp.md)、[UI unit-test設計](../32_unit_test/ui.md)。
- `Window`はC ABI Window tokenをmove-only RAII ownerとして保持し、`RuntimeCleanupState`共有参照とtoken受領前に確保した`PendingNode`を保持する。明示closeのrelease失敗はtokenをowner内に残し、destructor失敗は既存cleanup supervisorへ移す。
- 背景RGB値のvalidation/defaultはCoreへ委譲する。C++ wrapperは`RgbColor`を3つのfloatへ展開し、C ABIのraw statusを`Result<T>`/`UiError`へ保つ。wrapper側に色stateをcacheしない。
- Window handle生成・背景色APIはPlatform peerのopen/closeやOS表示を開始しない。
- fake ABI unitでは生成失敗、default/set/invalid/reset、get/set/reset status伝播、move、close retry、destructor release retryを検証する。移動したCMake packageの外部consumerでは実C ABI経由のcreate/get/set/reset/releaseを確認する。
- WindowsではVS2022 BuildTools/MSVC 19.44.35224の開発環境で`tools/dev.py check`と`minimum`がpassした。VS18 BuildTools 14.51 includeと19.44 compilerが混ざる既定のlatest環境ではMSVC標準ライブラリのversion mismatchが起こる場合があるため、診断時はcompiler/header toolchainの整合を確認する。
