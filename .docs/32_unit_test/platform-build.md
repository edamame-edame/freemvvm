# 単体試験：Platform接点・artifact検証・package選択

- 文書版: 0.1
- 状態: 0.1全文承認済み（利用者、2026-10-17）。製品testは未実装・未実行。接続gapは別管理
- 更新日: 2026-10-17

候補単位はsnapshot生成、surface generation、ActionRequest、ArtifactSelector/SecurityVerifier、PackageAssembler/NativeLoader。fake OS/FS/clock/networkで局所契約を検査し、実GPU・IME・UIA/AT-SPI・実loaderの保証はplatform/package実行へ残す。

上位根拠: [単体試験方針](unit-test-strategy.md)、[実装unit台帳契約](implementation-test-inventory.md)、[共通fixture・実行記録](README.md)。

設計根拠: [platform/abstraction.md](../13_detail_design/platform/abstraction.md) 1.1「CoreとOS実装の境界」、[platform/internal-layers.md](../13_detail_design/platform/internal-layers.md) 1.1「surface / OS同期action」、[build/toolchain-packages.md](../13_detail_design/build/toolchain-packages.md) 1.1「artifact検証 / package / loader」、[manifest verifier](../14_secure_design/06-manifest-wire-verifier.md) 1.0「実現設計」、[trust/staging](../14_secure_design/07-trust-key-staging.md) 1.0「型・状態と所有」。

## OS境界の局所契約

Controlを更新した後のScene/AxSnapshotを保持する。surface generation、submit/present完了、ActionRequest開始を別制御点にし、2秒待機既定はfake clockで進める。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-platform-001"></a>UT-PLATFORM-001 | Scene生成後にControl変更とsurface喪失、旧generation frame完了を与える | 旧Sceneの値は不変、Core確定値を保ち、古い完了やsubmit失敗をPresentedとしない。暗黙software fallbackなし | [RQ-11-008](../11_requirements/RQ-11_platform.md#rq-11-008)、[RQ-11-020](../11_requirements/RQ-11_platform.md#rq-11-020)、[RQ-11-022](../11_requirements/RQ-11_platform.md#rq-11-022)<br/>[TC-PLAT-01](../30_release_test/requirement-test-matrix.md#case-tc-plat-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-SUPPORT-01](../30_release_test/requirement-test-matrix.md#case-tc-support-01) |
| <a id="ut-ax-001"></a>UT-AX-001 | name未設定/空と表示labelを別設定し、Logical/Visual内部部品をsnapshotへ変換 | nameを表示値から合成せず、Logical階層を公開。queryでFocusを変更せず終了nodeを新nodeへ誤解決しない | [RQ-10-011](../11_requirements/RQ-10_accessibility.md#rq-10-011)、[RQ-10-012](../11_requirements/RQ-10_accessibility.md#rq-10-012)、[RQ-10-013](../11_requirements/RQ-10_accessibility.md#rq-10-013)<br/>[TC-ACC-01](../30_release_test/requirement-test-matrix.md#case-tc-acc-01)、[TC-ACC-02](../30_release_test/requirement-test-matrix.md#case-tc-acc-02)、[TC-ACC-03](../30_release_test/requirement-test-matrix.md#case-tc-acc-03)、[TC-ACC-04](../30_release_test/requirement-test-matrix.md#case-tc-acc-04)、[TC-ACC-05](../30_release_test/requirement-test-matrix.md#case-tc-acc-05)、[TC-ACC-06](../30_release_test/requirement-test-matrix.md#case-tc-acc-06)、[TC-ACC-07](../30_release_test/requirement-test-matrix.md#case-tc-acc-07)、[TC-ACC-08](../30_release_test/requirement-test-matrix.md#case-tc-acc-08)、[TC-ACC-PLAT-01](../30_release_test/requirement-test-matrix.md#case-tc-acc-plat-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-ax-002"></a>UT-AX-002 | 同期actionを未開始でtimeout、開始後timeout、通常完了に分岐する | 既定2秒と設定値を識別。未開始取消と開始済み結果不明を区別し、timeoutをrollback成功としない | [RQ-10-014](../11_requirements/RQ-10_accessibility.md#rq-10-014)<br/>[TC-ACC-PLAT-01](../30_release_test/requirement-test-matrix.md#case-tc-acc-plat-01) |
## 検証からloadまで

テスト専用鍵で署名済みmetadataとnative bytesを作り、fake clock/trust sequence/private stagingを用意する。実運用の鍵・認証情報はfixtureに使わない。各ケースは独立cacheで実施し一時dir/handleを終了後に回収する。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-artifact-001"></a>UT-ARTIFACT-001 | target/profile/ABI/featureを一項目ずつ不適合にし、適合artifactと比較する | 不適合はload前に拒否し、別targetへ流用しない。VerifiedArtifactへ未検証値を昇格しない | [RQ-12-001](../11_requirements/RQ-12_build-distribution.md#rq-12-001)、[RQ-12-011](../11_requirements/RQ-12_build-distribution.md#rq-12-011)<br/>[TC-ARTIFACT-01](../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)、[TC-TARGET-A64-01](../30_release_test/requirement-test-matrix.md#case-tc-target-a64-01)、[TC-TARGET-ARMV7-01](../30_release_test/requirement-test-matrix.md#case-tc-target-armv7-01)、[TC-TARGET-BOARD-01](../30_release_test/requirement-test-matrix.md#case-tc-target-board-01)、[TC-TARGET-LATER-01](../30_release_test/requirement-test-matrix.md#case-tc-target-later-01)、[TC-TARGET-LNX-01](../30_release_test/requirement-test-matrix.md#case-tc-target-lnx-01)、[TC-TARGET-WIN-01](../30_release_test/requirement-test-matrix.md#case-tc-target-win-01)、[TC-PKG-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-artifact-002"></a>UT-ARTIFACT-002 | 重複key・未知major・未知必須・未知optionalを署名vectorで与える | 曖昧/未知必須は拒否。optional無視は署名検証後のみ。選択や信頼に必要な情報をoptional扱いしない | [RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015)<br/>[TC-ARTIFACT-01](../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-artifact-003"></a>UT-ARTIFACT-003 | 未信頼鍵・失効・期限切れ・rollback・時計逆行を個別に注入しoffline検証する | 拒否を明示し自動trust/期限延長しない。未取得の最新失効を検知済みとは主張しない | [RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-023](../11_requirements/RQ-12_build-distribution.md#rq-12-023)<br/>[TC-ARTIFACT-01](../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-artifact-004"></a>UT-ARTIFACT-004 | manifest上限の各dimensionについて上限-1/上限/上限+1を合法構造で生成する | bytes/depth/member/string/signature制限を個別に確認。超過拒否でstagingをpublishしない | [RQ-12-022](../11_requirements/RQ-12_build-distribution.md#rq-12-022)<br/>[TC-ARTIFACT-01](../30_release_test/requirement-test-matrix.md#case-tc-artifact-01) |
| <a id="ut-artifact-005"></a>UT-ARTIFACT-005 | 検証後cache差替え、依存binary改ざん、archiveの.. / absolute / link entryを試す | 同一検証objectへの束縛と依存digestを維持。不正entryを拒否しprivate staging外へ書かない | [RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015)<br/>[TC-ARTIFACT-01](../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
| <a id="ut-package-001"></a>UT-PACKAGE-001 | 一native binaryから4言語packageを組み、import/loadをnetwork禁止shimで行う | native identity/digestを共有しCore再compile/複製なし。import時に暗黙取得・更新しない | [RQ-12-003](../11_requirements/RQ-12_build-distribution.md#rq-12-003)、[RQ-12-012](../11_requirements/RQ-12_build-distribution.md#rq-12-012)、[RQ-12-013](../11_requirements/RQ-12_build-distribution.md#rq-12-013)、[RQ-12-025](../11_requirements/RQ-12_build-distribution.md#rq-12-025)<br/>[TC-ARTIFACT-01](../30_release_test/requirement-test-matrix.md#case-tc-artifact-01)、[TC-PKG-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cpp-01)、[TC-PKG-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-cs-01)、[TC-PKG-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-py-01)、[TC-PKG-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-pkg-rs-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-ARCH-01](../30_release_test/requirement-test-matrix.md#case-tc-arch-01) |

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |
| 2026-10-17 | 0.1全文 | 利用者 | 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存試験案を承認し環境整備を開始。製品test実装・合格を意味しない |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
