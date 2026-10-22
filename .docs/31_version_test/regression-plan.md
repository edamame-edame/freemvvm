# Version試験：変更影響・互換性・回帰選択

- 文書版: 0.1
- 状態: 0.1全文承認済み（利用者、2026-10-17）。製品testは未実装・未実行。接続gapは別管理
- 更新日: 2026-10-17

上位規約は[version-test](version-test.md)、[共通manifest](../30_release_test/requirements-test-manifest.md)、[実装unit台帳](../32_unit_test/implementation-test-inventory.md) 1.1。要求oracleは[RQ-04の互換契約](../11_requirements/RQ-04_abi-boundary.md)、[RQ-12のartifact/信頼契約](../11_requirements/RQ-12_build-distribution.md)。設計oracleは[ABI](../13_detail_design/runtime-objects/abi.md) 1.1と[build/package](../13_detail_design/build/toolchain-packages.md) 1.1、関連SDとする。

## 前提と選択手順

変更前baseline version/SHAと候補version/SHA、両方のunits manifest、変更RQ/設計文書・対象版・節を固定する。sourceとtestは候補の同一checkout SHAを使い、過去releaseの証跡を今回の実行結果で上書きしない。新旧互換consumerは個別のtoolchainとartifact digestを記録する。

1. renameは旧新path、deletionは旧manifest、共有header/config/generated inputは関連全unitへ対応付ける。
2. 直接変更unitと依存逆閉包、変更RQのTC、該当release縦断scenarioを選ぶ。test文書だけの変更は文書検査であり、実装unitの動作証明にしない。
3. 未mapping pathは定義済みwide fallbackへ広げる。fallback未定義ならmapping failure/blockedとし、success skipにしない。
4. 新機能の対象stage・feature/profileと非対応構成を分け、未提供機能を旧consumerへ必須化しない。
5. 実行後、各testとTC/RQのassertion、選択理由・除外理由・追加回帰、SHA/profile/reportを保存する。

## 変更対象からの回帰範囲

| 変更対象 | 最低限選ぶ試験設計 | 上位回帰 |
| --- | --- | --- |
| handle・値・ABI表現 | [Core/runtime](../32_unit_test/core-runtime.md)、[4wrapper](../32_unit_test/wrappers.md)、consumer互換 | artifactを共有する全wrapperと[REL-PACKAGE-001](../30_release_test/system-scenarios.md#rel-package-001) |
| Runtime・callback・通知 | [Core/runtime](../32_unit_test/core-runtime.md)、[events](../32_unit_test/events.md)、全cleanup owner | [REL-LIFE-001](../30_release_test/system-scenarios.md#rel-life-001)、Button/Binding/TextBoxを含む縦断 |
| Binding・Command・Validation | [events](../32_unit_test/events.md)、[UI編集](../32_unit_test/ui.md)、関連wrapper | [REL-UI-002](../30_release_test/system-scenarios.md#rel-ui-002)、[REL-UI-003](../30_release_test/system-scenarios.md#rel-ui-003) |
| Tree・Layout・Input・IME | [UI](../32_unit_test/ui.md)、snapshotとplatform generation | 対象OSの[REL-UI-004](../30_release_test/system-scenarios.md#rel-ui-004)、Accessibility、frame確認 |
| Collection・Selection | [Collection](../32_unit_test/collections.md)、参照する次段階Control | 提供stageのsnapshot/選択縦断。初期版へ一律必須化しない |
| Platform/backend/font | [Platform](../32_unit_test/platform-build.md)、Core snapshot契約 | 対象profileの実描画・入力・IME・Accessibility。未検証architectureへ成功を流用しない |
| manifest/trust/loader/package | [artifact局所検証](../32_unit_test/platform-build.md)、関連SD vector | [REL-ARTIFACT-001](../30_release_test/system-scenarios.md#rel-artifact-001)、全該当package consumer |
| analyzer/frontend | [解析器](../32_unit_test/wrappers.md)、同じ入力で全入口の診断比較 | lint/type/MCP/CLI/CI parity。解析対象ソースを実行しない |

## 互換・選択の具体ケース

各行は独立したcandidate/baseline環境を前提とする。予定test IDを実command/symbolへ登録するのは実装時とし、現在は全件not-runである。

| 予定test ID / 論理case | 操作 | 期待結果・観測 |
| --- | --- | --- |
| <a id="ver-selection-001"></a>VER-SELECTION-001 / [TC-DEV-01](../30_release_test/requirement-test-matrix.md#case-tc-dev-01) | fixture units manifestで変更、rename、deletion、shared header変更を与える | 旧新mappingと依存逆閉包どおりのsuite集合。未mappingを黙って捨てずfallbackまたはfailure。証拠は選択入力と集合 |
| <a id="ver-abi-001"></a>VER-ABI-001 / [TC-ABI-COMP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-01) | 同majorの旧clientを新Runtimeへ接続。新wrapperを最低条件未満Runtimeへ接続するnegativeも試す | 公表した後方互換を維持し、不適合は利用前に検知。minor/patchで公開status/featureの意味を再利用しない。[RQ-04-024](../11_requirements/RQ-04_abi-boundary.md#rq-04-024)、[RQ-04-028](../11_requirements/RQ-04_abi-boundary.md#rq-04-028) |
| <a id="ver-feature-001"></a>VER-FEATURE-001 / [TC-ABI-COMP-02](../30_release_test/requirement-test-matrix.md#case-tc-abi-comp-02) | known supported/unsupportedと未知featureを照会。optional新機能なしの旧Runtimeでconsumerを起動 | 未知は未対応。必須feature不足の拒否と、非必須featureを使わない利用を区別。新Runtimeで作った証跡を旧Runtimeへ流用しない。[RQ-04-010](../11_requirements/RQ-04_abi-boundary.md#rq-04-010)、[RQ-04-024](../11_requirements/RQ-04_abi-boundary.md#rq-04-024) |
| <a id="ver-package-001"></a>VER-PACKAGE-001 / [TC-ARTIFACT-01](../30_release_test/requirement-test-matrix.md#case-tc-artifact-01) | 新旧packageを分離cacheへ導入し、明示更新前後とoffline再起動を観測 | 固定版の暗黙更新なし。wrapperが公表したRuntime/ABI条件に従いidentityを選択。不適合bytesを同名だからとloadしない。[RQ-12-011](../11_requirements/RQ-12_build-distribution.md#rq-12-011)、[RQ-12-013](../11_requirements/RQ-12_build-distribution.md#rq-12-013) |
| <a id="ver-trust-001"></a>VER-TRUST-001 / [TC-ARTIFACT-01](../30_release_test/requirement-test-matrix.md#case-tc-artifact-01) | 正当root rotationと旧root/失効sequenceへのrollback、期限切れbundleを別fixtureで投入 | 旧新閾値認証による更新を検査しrollback/期限切れ拒否。製品versionを戻すこととtrust metadata rollbackを混同しない。[RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-023](../11_requirements/RQ-12_build-distribution.md#rq-12-023)、[SD trust](../14_secure_design/07-trust-key-staging.md) 1.0 |
| <a id="ver-behavior-001"></a>VER-BEHAVIOR-001 / [TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) | 変更RQに対応するTC/assertionを差分比較し、独立fixtureで候補の回帰を実行 | 意図した変更と従来保証維持をRQ別に記録。baselineの実装バグを正しいoracleとせず、承認済み要求/設計へ照合。[RQ-16-003](../11_requirements/RQ-16_document-governance.md#rq-16-003) |

## 後始末とreport

新旧consumer processを終了し、残owner/callback root/workerを回収する。テスト用trust store/cacheは個別に破棄し、baselineの配布物・metadata・過去証跡を変更しない。結果は変更前後version/SHA、選択manifest版、実行SHA、TC/RQ別assertion、target/profile、artifact digest、toolchain/command/logを保持する。

全件未実装のため、現時点のsuite選択は計画でありCI実行結果ではない。[接続gap](../32_unit_test/README.md)が影響するcaseはblocked理由を添え、変更影響が不明なままversion完了としない。releaseとして公開する場合はversion回帰に加えてrelease規約のgateを満たす必要がある。

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |
| 2026-10-17 | 0.1全文 | 利用者 | 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存試験案を承認し環境整備を開始。製品test実装・合格を意味しない |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
