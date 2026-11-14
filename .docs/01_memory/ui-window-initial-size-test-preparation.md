# freemvvm: UI Window初期client sizeの試験設計

- カテゴリ: 試験設計 / Core・C ABI / 0.0.1
- 対象: RQ-09-076、Window初期希望寸法、Wayland/Core接続

## 学んだこと

- 初期size testでは、Windowが保持するDIP希望値と、OS/compositorが返す実client areaを別々に観測する。既定希望値800×600、正の有限値、invalid時の生成拒否とfailure atomicityをCore/C ABI oracleにする。
- Core unitとABI unitを分ける。wrapper parity、画面表示、Wayland configure/resizeはこれらのpassに含めず、wrapper/Platform release testで別途確認する。
- 要求test case/assertion文書とunit catalogはsource実装前にレビュー承認を取り、未承認のcaseを既存承認case coverageへ数えない。Core初期サイズunitはfake peerで希望値のopen受渡し、OS実寸のLayout利用、close/reopen再利用、invalid値のhandle非公開を確認する。
- Core側のPlatform peer API拡張では既存`open(generation)`を維持し、default `open_with_initial_client_size`を追加すると既存peerを壊さずに新しい希望サイズ契約へ段階対応できる。

## 実装記録

- 2026-11-14、`UNIT-CORE-095`を実装し、`cargo test -p freemvvm-core initial_client_size --locked`、Core全206 tests、Linux/Windows platform peer tests、workspace all-target checkを通した。Wayland peerは後続unitでサイズを実際のconfigure/初回surfaceへ適用する。

参照: [UI unit-test設計](../32_unit_test/ui.md)、[要求test対応表](../30_release_test/requirement-test-matrix.md)、[requirements assertions](../30_release_test/requirements-test-assertions.md)、[Core/C ABI unit catalog](../../tools/units/core-ui.yaml)、[ABI unit catalog](../../tools/units/abi.yaml)。
