# RQ-15 / 22旧IDの意味単位移行案

- 文書版: 0.2
- 状態: 全案反映承認済み・正本適用済み（2026-10-17）。具体化未了は適用結果を参照
- 作成日: 2026-10-17
- 正本: [RQ-15](../../11_requirements/RQ-15_requirement-decisions.md)

旧IDは廃止・統合後も再利用しない。各行で移管済みを確認するまではRQ-15本文を削除しない。既存RQへの統合だけで条件を落とさず、足りない文は仮候補から新しい正式RQへ分割する。RQ-15の移行案内は同層のRQのみをリンクし、BD/DD/testへの参照を要求仕様へ追加しない。

| 旧RQ ID | 統合・照合先 | 分割補足候補 | 意味を保持する移管内容 |
|---|---|---|---|
| [RQ-15-001](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-001) | [RQ-12-010](../../11_requirements/RQ-12_build-distribution.md#rq-12-010)、[RQ-11-003](../../11_requirements/RQ-11_platform.md#rq-11-003)、[RQ-11-004](../../11_requirements/RQ-11_platform.md#rq-11-004) | 既存要求へ統合。方式/履歴は別成果物 | 対応・検証の優先順は既存RQ-12-010へ統合し、公開supportと開発順を分ける。cross-buildのsource共有方式はBD-12/16へ。 |
| [RQ-15-002](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-002) | [RQ-01-002](../../11_requirements/RQ-01_purpose-architecture.md#rq-01-002)、[RQ-11-008](../../11_requirements/RQ-11_platform.md#rq-11-008) | 既存要求へ統合。方式/履歴は別成果物 | 共通のControl状態正本という制約をRQへ保持。独自共通Rendererでnative controlを正本にしない実現方式はBD-12へ残す。 |
| [RQ-15-003](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-003) | [RQ-11-008](../../11_requirements/RQ-11_platform.md#rq-11-008)、[RQ-11-009](../../11_requirements/RQ-11_platform.md#rq-11-009) | 既存要求へ統合。方式/履歴は別成果物 | CoreとRenderer/Platform分離の既存制約へ統合。Visual snapshotとboundsの受渡しはBD/DD。 |
| [RQ-15-004](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-004) | [RQ-09-002](../../11_requirements/RQ-09_ui-scope.md#rq-09-002)、[RQ-09-003](../../11_requirements/RQ-09_ui-scope.md#rq-09-003) | 既存要求へ統合。方式/履歴は別成果物 | Widget/Containerの利用上の分類と対象を既存RQへ統合。継承class/trait構成はDD。旧Elementの読み替えは履歴へ。 |
| [RQ-15-005](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-005) | [RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005) | [SEP-RQ-058 text coverageと編集位置](rq-candidates.md#candidate-058) | Latin/日本語の表示・編集、targetごとのfont依存の識別をRQ-09の新しい意味単位へ。font discovery/shaping/rasterizationの方式とcorpusはBD/test。 |
| [RQ-15-006](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-006) | [RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005) | [SEP-RQ-026 TextBox入力三値の結果](rq-candidates.md#candidate-026)、[SEP-RQ-028 IME cancel後の入力理由](rq-candidates.md#candidate-028)、[SEP-RQ-030 外部更新と編集中の値](rq-candidates.md#candidate-030) | Focus離脱でpreedit/位置を保持、明示cancelで開始前へ復元、commit/FocusLostの受信順と終了済みsessionの遅延event拒否を別条件として新RQへ。session ID表現はDD。 |
| [RQ-15-007](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-007) | [RQ-07-008](../../11_requirements/RQ-07_binding.md#rq-07-008) | [SEP-RQ-022 長期保留の診断設定](rq-candidates.md#candidate-022)、[SEP-RQ-023 非収束診断と回復](rq-candidates.md#candidate-023)、[SEP-RQ-048 診断・解析の能力と限界](rq-candidates.md#candidate-048) | 静的に解決可能なControl/Propertyアクセス警告はRQ-03診断能力へ、有限作業と保留/診断/非収束連鎖停止はRQ-07へ分ける。128/hash/解析方式はDD。 |
| [RQ-15-008](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-008) | [RQ-07-010](../../11_requirements/RQ-07_binding.md#rq-07-010)、[RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005) | [SEP-RQ-024 Validator登録条件](rq-candidates.md#candidate-024)、[SEP-RQ-025 判定callback中の禁止操作](rq-candidates.md#candidate-025) | 短時間同期判定と初期Core async pending非提供を機能範囲として保持。ViewModelのAsyncCommand構成とrevision照合方式はBD-25。 |
| [RQ-15-009](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-009) | [RQ-06-004](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-004) | [SEP-RQ-031 Command実行受付と結果](rq-candidates.md#candidate-031)、[SEP-RQ-032 取消・close・handle破棄の差](rq-candidates.md#candidate-032) | 進捗は最新値集約可、終端は独立一回、過去実行・終端後通知は適用しない結果を分割する。operation IDとslot/wakeはBD/DD。 |
| [RQ-15-010](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-010) | [RQ-07-005](../../11_requirements/RQ-07_binding.md#rq-07-005)、[RQ-07-006](../../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-07-009](../../11_requirements/RQ-07_binding.md#rq-07-009) | 既存要求へ統合。方式/履歴は別成果物 | 即時値確定/後続配信/独立setter非rollbackと一般transaction非提供へ統合。外側scopeとqueueの方式はBD-02/DD-07。 |
| [RQ-15-011](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-011) | [RQ-04-006](../../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](../../11_requirements/RQ-04_abi-boundary.md#rq-04-007) | [SEP-RQ-017 error詳細を生成できない失敗](rq-candidates.md#candidate-017)、[SEP-RQ-018 利用者定義エラーの自動採番](rq-candidates.md#candidate-018) | 機械識別、説明message、不変snapshot、cause/diagnosticの意味と利用期間を分割要求にする。handle/domain配置はDD。 |
| [RQ-15-012](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-012) | [RQ-04-009](../../11_requirements/RQ-04_abi-boundary.md#rq-04-009) | [SEP-RQ-019 機能照会と互換性](rq-candidates.md#candidate-019) | stable versionと同major後方互換、wrapper minimum宣言を新しい互換条件へ。symbol prefix/signature管理の方式はDD-06。 |
| [RQ-15-013](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-013) | [RQ-12-007](../../11_requirements/RQ-12_build-distribution.md#rq-12-007) | [SEP-RQ-048 診断・解析の能力と限界](rq-candidates.md#candidate-048) | ctypes/extension非必須をRQ-12へ統合。lint/type checkとMCP/CLI/CIで同一診断を利用できる能力をRQ-03へ。Temp cache増分方式はBD-15。 |
| [RQ-15-014](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-014) | [RQ-12-004](../../11_requirements/RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](../../11_requirements/RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](../../11_requirements/RQ-12_build-distribution.md#rq-12-006)、[RQ-12-008](../../11_requirements/RQ-12_build-distribution.md#rq-12-008) | [SEP-RQ-053 license/noticeの継承](rq-candidates.md#candidate-053) | license/再配布/依存noticeの独立要求をRQ-12へ新設。package別の収録実装と検査はDD/release-test。 |
| [RQ-15-015](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-015) | [RQ-11-001](../../11_requirements/RQ-11_platform.md#rq-11-001)、[RQ-11-002](../../11_requirements/RQ-11_platform.md#rq-11-002) | [SEP-RQ-049 検証状態と正式support](rq-candidates.md#candidate-049)、[SEP-RQ-054 support lifecycleとEOL記録](rq-candidates.md#candidate-054) | support状態・24か月限定保守・原則12か月告知・security例外・EOL記録をRQ-12の独立要求へ。状態表更新は運用。 |
| [RQ-15-016](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-016) | [RQ-12-004](../../11_requirements/RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](../../11_requirements/RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](../../11_requirements/RQ-12_build-distribution.md#rq-12-006)、[RQ-12-008](../../11_requirements/RQ-12_build-distribution.md#rq-12-008) | [SEP-RQ-050 artifact選択・identity・取得](rq-candidates.md#candidate-050)、[SEP-RQ-051 artifactの信頼と検証失敗](rq-candidates.md#candidate-051)、[SEP-RQ-052 manifestの互換・曖昧性](rq-candidates.md#candidate-052) | 各言語配布境界の重複は既存RQへ。identity/適合/明示取得/信頼/失効/offline/schema互換を別要求へ分割。catalog/cache/envelopeはDD/SD。 |
| [RQ-15-017](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-017) | [RQ-12-010](../../11_requirements/RQ-12_build-distribution.md#rq-12-010) | 既存要求へ統合。方式/履歴は別成果物 | 初期CI順・ARM両target cross-build・実機verification区分はRQ-12-010へ統合。具体job/gateはtest計画へ。 |
| [RQ-15-018](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-018) | [RQ-10-001](../../11_requirements/RQ-10_accessibility.md#rq-10-001)、[RQ-10-002](../../11_requirements/RQ-10_accessibility.md#rq-10-002)、[RQ-10-003](../../11_requirements/RQ-10_accessibility.md#rq-10-003)、[RQ-10-004](../../11_requirements/RQ-10_accessibility.md#rq-10-004)、[RQ-10-005](../../11_requirements/RQ-10_accessibility.md#rq-10-005)、[RQ-10-006](../../11_requirements/RQ-10_accessibility.md#rq-10-006)、[RQ-10-007](../../11_requirements/RQ-10_accessibility.md#rq-10-007)、[RQ-10-008](../../11_requirements/RQ-10_accessibility.md#rq-10-008)、[RQ-10-009](../../11_requirements/RQ-10_accessibility.md#rq-10-009) | [SEP-RQ-040 Accessibility identityと操作境界](rq-candidates.md#candidate-040)、[SEP-RQ-041 Accessibility nameの空欄](rq-candidates.md#candidate-041) | 共通意味とLogical階層を既存RQ-10へ統合し、Visual内部部品非公開の利用上の結果を補足。snapshot/OS providerはDD。 |
| [RQ-15-019](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-019) | [RQ-14-002](../../11_requirements/RQ-14_delivery-sequence.md#rq-14-002) | 既存要求へ統合。方式/履歴は別成果物 | 所有/callback/threadの優先確認という実施要求をRQ-14の既存工程へ。検証手順はTC-TRACE/DEVへ。 |
| [RQ-15-020](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-020) | [RQ-14-003](../../11_requirements/RQ-14_delivery-sequence.md#rq-14-003)、[RQ-14-005](../../11_requirements/RQ-14_delivery-sequence.md#rq-14-005) | 既存要求へ統合。方式/履歴は別成果物 | Binding更新意味論を実装・smokeに先行確認する工程条件へ統合。実際のcaseはTC-BINDへ。 |
| [RQ-15-021](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-021) | [RQ-14-010](../../11_requirements/RQ-14_delivery-sequence.md#rq-14-010)、[RQ-14-012](../../11_requirements/RQ-14_delivery-sequence.md#rq-14-012) | 既存要求へ統合。方式/履歴は別成果物 | 初期OS/embedded描画・実機の確認順を実施要求へ統合。具体runner/boardはtest計画へ。 |
| [RQ-15-022](../../11_requirements/RQ-15_requirement-decisions.md#rq-15-022) | [RQ-14-007](../../11_requirements/RQ-14_delivery-sequence.md#rq-14-007)、[RQ-14-008](../../11_requirements/RQ-14_delivery-sequence.md#rq-14-008) | 既存要求へ統合。方式/履歴は別成果物 | Text/IME/Accessibility範囲の確認を各工程へ対応付ける。具体corpus/AT/操作は要求testへ。 |

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-17 | RQ-15意味単位移行 | — | 一括作成指示に基づくレビュー案。採用・正本適用は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 状態 |
|---|---|---|---|
| 0.1 | 2026-10-17 | 現行文書から責務分離・改訂文案・影響を作成 | レビュー待ち |

## 2026-10-17 適用結果

正本は [適用結果](../requirements-design-separation-application-result.md) と [正式要求ID移行](../../11_requirements/requirements-separation-id-migration.md) に記録した。以下の比較・引用・選択肢は作成時点の案として保持する。採用した選択は推奨案、未提示の実装詳細は後続課題として扱った。

### 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全レビュー案の反映 | 利用者 | 「すべて反映して下さい。」により採用・正本適用を承認 |

### 改訂履歴

| 版 | 日付 | 内容 |
| --- | --- | --- |
| 0.2 | 2026-10-17 | 承認と正本適用結果を追記。作成時点の比較・引用を保持 |
