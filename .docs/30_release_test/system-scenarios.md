# Release試験：共通UI縦断・profile・配布物

- 文書版: 0.1
- 状態: レビュー案・未承認。全scenario planned / 未実装 / not-run
- 更新日: 2026-10-17

上位規約は[release-test](release-test.md)、[共通manifest](requirements-test-manifest.md)、[要求対応表](requirement-test-matrix.md)。実現根拠は[BD全体構成](../12_design/architecture.md) 1.0、[DD入口](../13_detail_design/README.md)と各分野の1.1。単体のfake成功を実OS・配布物の成功へ読み替えない。下記IDは予定実test識別子であり、既存TCを置き換えない。

## 共通前提・入力・証跡

候補releaseのSHAとnative artifact identity/digestを固定し、4wrapperが同一target向けの同じnative binaryを使う。各profileのbuild条件・OS/driver/font/IME/provider・ABI/feature・wrapper版を記録する。source/testは同じcheckout SHA、installed packageも当該SHAから生成したidentityであることを確認する。別版のpreinstalled native libraryを拾った実行は無効とする。

共通UIはWindow→StackPanel→TextBox/Text/Button、ViewModelのUTF8 PropertyとCommandを持つ。TextBox↔ViewModelをTwoWay、ViewModel→TextをOneWay、Button→Commandを接続し、表示labelとAccessibility nameは別値にする。代表入力は「初期値」→「変更値」と日本語preedit。これらは目視とbyte一致を区別しやすいfixture値で、製品制限ではない。数値pixel一致や日本語以外の全文字対応をこの入力で証明しない。

各行を独立した初期状態で実施する。状態・値・通知順・operation ID・Focus・session・公開snapshotを記録し、観測できない内部条件をpass推定しない。失敗系はtest adapterで注入した結果と実OS障害を区別する。

## シナリオ

| 予定test ID / 論理case | 操作・観測 | 期待結果・根拠 |
| --- | --- | --- |
| <a id="rel-ui-001"></a>REL-UI-001 / [TC-VERT-01](requirement-test-matrix.md#case-tc-vert-01)、[TC-VERT-02](requirement-test-matrix.md#case-tc-vert-02) | 配布packageから共通UIを構築しopen。初期Property、Tree、layout、描画済みframeを別々に観測 | 同じ機能・値へ4言語から到達。単なるsubmitを表示成功とせず、Coreと実描画の証跡を分ける。[RQ-01-002](../11_requirements/RQ-01_purpose-architecture.md#rq-01-002)、[RQ-09-001](../11_requirements/RQ-09_ui-scope.md#rq-09-001)、[RQ-11-008](../11_requirements/RQ-11_platform.md#rq-11-008) |
| <a id="rel-ui-002"></a>REL-UI-002 / [TC-VERT-03](requirement-test-matrix.md#case-tc-vert-03)、[TC-VERT-04](requirement-test-matrix.md#case-tc-vert-04) | TextBoxへ受理値を入力しViewModel/Textをread。次にVMから別値を設定。拒否候補も入力 | 受理時だけ確定値を伝播、表示側の同一確定値を確認。拒否時は本文の編集保持規則に従いVMを変更しない。[RQ-07-002](../11_requirements/RQ-07_binding.md#rq-07-002)、[RQ-07-003](../11_requirements/RQ-07_binding.md#rq-07-003)、[RQ-09-038](../11_requirements/RQ-09_ui-scope.md#rq-09-038) |
| <a id="rel-ui-003"></a>REL-UI-003 / [TC-VERT-05](requirement-test-matrix.md#case-tc-vert-05)、[TC-CMD-01](requirement-test-matrix.md#case-tc-cmd-01) | ButtonでCommand開始→実行不可を観測→worker成功/失敗/取消を個別報告→再実行。旧ID報告を遅延投入 | UIを長時間占有せず、取消要求と終端を区別。再登録なしの再実行は新ID。旧結果で上書きしない。[RQ-06-004](../11_requirements/RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](../11_requirements/RQ-06_threading-dispatch.md#rq-06-008)、[RQ-06-009](../11_requirements/RQ-06_threading-dispatch.md#rq-06-009)。専用進捗channelは[接続gap](../32_unit_test/README.md)の解消後に追加 |
| <a id="rel-ui-004"></a>REL-UI-004 / [TC-VERT-06](requirement-test-matrix.md#case-tc-vert-06)、[TC-IME-01](requirement-test-matrix.md#case-tc-ime-01) | 実OS入力でFocus移動、日本語preedit、Focus離脱、明示cancel、commit、終了session遅延eventを別fixtureで観測 | Focus離脱は未確定編集保持、cancelは開始状態復元、commitは受理結果だけ確定、stale event不適用。特定Tab/shortcutは要求しない。[RQ-09-040](../11_requirements/RQ-09_ui-scope.md#rq-09-040)、[RQ-09-055](../11_requirements/RQ-09_ui-scope.md#rq-09-055)、[RQ-09-056](../11_requirements/RQ-09_ui-scope.md#rq-09-056) |
| <a id="rel-ui-005"></a>REL-UI-005 / [TC-VERT-07](requirement-test-matrix.md#case-tc-vert-07)、[TC-ACC-PLAT-01](requirement-test-matrix.md#case-tc-acc-plat-01) | OS providerからname/role/階層をquery。Button actionとTextBox編集actionを実施し、未開始/開始済みtimeoutも制御 | queryだけでFocusを変えず通常操作と同じCore結果。空nameをlabelから補わない。timeoutの未開始取消と結果不明を区別。[RQ-10-011](../11_requirements/RQ-10_accessibility.md#rq-10-011)、[RQ-10-012](../11_requirements/RQ-10_accessibility.md#rq-10-012)、[RQ-10-014](../11_requirements/RQ-10_accessibility.md#rq-10-014) |
| <a id="rel-life-001"></a>REL-LIFE-001 / [TC-VERT-08](requirement-test-matrix.md#case-tc-vert-08)、[TC-LIFE-01](requirement-test-matrix.md#case-tc-life-01) | owner releaseと明示cancel/closeを区別し、Window close後再open、callback中Runtime終了、残owner解放を順に観測 | Window closeだけではRuntime終了せずCore値保持。callback完了後に終了優先、後始末一回。未完了を完了表示しない。[RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-06-005](../11_requirements/RQ-06_threading-dispatch.md#rq-06-005)、[RQ-09-071](../11_requirements/RQ-09_ui-scope.md#rq-09-071) |
| <a id="rel-package-001"></a>REL-PACKAGE-001 / [TC-PKG-CPP-01](requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-RS-01](requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-PKG-PY-01](requirement-test-matrix.md#case-tc-pkg-py-01) | 開発treeの探索pathを外したconsumer環境で各packageを導入し、共通UIを起動。network遮断下でもimport/load | CMake/NuGet/Cargo/Pythonの配布境界に従って同identity/digestをload。import時に暗黙fetchや更新なし。未検証ARM wheelを適合表示しない。[RQ-12-003](../11_requirements/RQ-12_build-distribution.md#rq-12-003)、[RQ-12-013](../11_requirements/RQ-12_build-distribution.md#rq-12-013)、[RQ-12-025](../11_requirements/RQ-12_build-distribution.md#rq-12-025) |
| <a id="rel-artifact-001"></a>REL-ARTIFACT-001 / [TC-ARTIFACT-01](requirement-test-matrix.md#case-tc-artifact-01) | テスト署名artifactの実FS上取込、offline再利用、改ざん、失効/期限切れ、依存binary差替えを別環境で試す | 検証した同一bytesのみ利用し、未信頼/失効/不適合を拒否。固定版を勝手に更新しない。[RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-023](../11_requirements/RQ-12_build-distribution.md#rq-12-023)、[SD staging](../14_secure_design/07-trust-key-staging.md) 1.0 |
| <a id="rel-profile-001"></a>REL-PROFILE-001 / [TC-TARGET-WIN-01](requirement-test-matrix.md#case-tc-target-win-01)、[TC-TARGET-LNX-01](requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TARGET-BOARD-01](requirement-test-matrix.md#case-tc-target-board-01) | 候補releaseのprofileごとに上記適用scenarioを実行。代表boardでdevice/surface喪失と再開も観測 | Windows11 x64、Debian12 x64、代表RPi4 AArch64の対象範囲を個別記録。ARMv7は初期build検証と実機動作を区別。[RQ-11-021](../11_requirements/RQ-11_platform.md#rq-11-021)、[RQ-11-024](../11_requirements/RQ-11_platform.md#rq-11-024) |

## 後始末と判定

workerを協調終了させ、登録を明示解除、Window/Commandを明示closeし、Runtime終了後に残owner・snapshotを解放する。native handle、callback root、cleanup ticket、一時consumer環境とcacheを回収し、終了失敗もreportへ残す。Pythonは登録済みmanaged hostの正常return・例外・SystemExit・loop終了を追加parameterにし、即時process killをcleanup完遂ケースへ含めない。[RQ-03-009](../11_requirements/RQ-03_language-support.md#rq-03-009)、[RQ-03-010](../11_requirements/RQ-03_language-support.md#rq-03-010)、[RQ-03-015](../11_requirements/RQ-03_language-support.md#rq-03-015)を適用する。

reportは予定test ID、TC/RQ別assertion、SHA、profile、wrapper/ABI/feature、artifact digest、コマンド、観測ログ、pass/fail/blocked/not-runを保持する。複数RQ共有scenarioでも一項目の成功を全RQの成功へ集計しない。profile×languageの正式support表と実commandは実装開始時に確定し、未登録の必須構成はblockedとしてrelease規約へ返す。全profileの正式supportを推定して宣言しない。

本書は最初の縦断scenario設計であり、release規約の必須gate、license/support監査、全要求testの代替ではない。Collectionと次段階Controlは採用stageに達したreleaseで専用scenarioを追加する。性能の合否閾値をこの文書で追加しない。

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
