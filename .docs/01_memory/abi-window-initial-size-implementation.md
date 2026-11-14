# freemvvm: Window初期client size C ABI実装

- カテゴリ: 実装知識 / ABI / Window / 0.0.1
- 対象: UNIT-ABI-024、feature 10、公開C header・C/C++ probes

## 学んだこと

- ABI 1の互換追加は既存symbolを維持し、新symbolと新feature IDを加える。Window初期size APIでは`ui_abi_1_window_create_with_initial_client_size`、feature 10、runtime version 1.1.0を追加する。
- optional backendの`NULL/0`はartifact default選択として有効。pointer/length片側指定と空identityはinvalid。backend未指定と明示backendのどちらでも有限・正のDIP希望寸法を渡す。
- 寸法はCoreで検証し、C ABIもout windowを先に0初期化する。無効寸法、UTF-8、backend identityの失敗でpartial handleを返さない。featureはsymbolと契約が同revisionで有効になってからtrueを返す。
- CMake presetが要求するNinjaがworktree環境にない場合、同じC/C++ probesは`MinGW Makefiles` generatorと利用可能なMinGW compilerで実行できる。ただし全CTestには今回のABI変更とは独立したC++ stderr-capture failureが出ることがある。ABI smokeだけの結果と全suiteの結果を分けて記録する。

## 実装記録

- 2026-11-14、`UNIT-ABI-024`を実装。`cargo test -p freemvvm-c-abi --locked` 43 tests、C ABI Clippy、C/C++ header compile probes、`abi.runtime-smoke`、`abi.runtime-smoke-cpp`がpass。MinGW CTestの全15件ではABI関連3件を含む14件がpassし、既存`wrapper.cpp.property.subscription`のstderr capture testが1件failした。

参照: [共通C ABI DD](../13_detail_design/runtime-objects/abi.md)、[UI unit-test設計](../32_unit_test/ui.md)、[ABI unit catalog](../../tools/units/abi.yaml)。
