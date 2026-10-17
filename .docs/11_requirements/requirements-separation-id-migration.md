# 要求分離の正式ID対応

- 文書版: 1.0
- 状態: 承認済み（2026-10-17）

## 補足要求の対応

仮IDは作業記録の識別用であり、要求の正本は正式IDへ移した。現行IDは穴詰めせず保持する。

| 仮ID | 項目 | 正式RQ ID |
| --- | --- | --- |
| SEP-RQ-001 | 任意threadの参照取得・破棄 | [RQ-05-010](RQ-05_ownership-callbacks.md#rq-05-010) |
| SEP-RQ-002 | 破棄済み参照への重複破棄 | [RQ-05-011](RQ-05_ownership-callbacks.md#rq-05-011) |
| SEP-RQ-003 | 安全な参照を発行できない場合 | [RQ-04-018](RQ-04_abi-boundary.md#rq-04-018) |
| SEP-RQ-004 | 参照の種類・所属の拒否結果 | [RQ-04-019](RQ-04_abi-boundary.md#rq-04-019) |
| SEP-RQ-005 | 破棄失敗時の参照保全 | [RQ-05-012](RQ-05_ownership-callbacks.md#rq-05-012) |
| SEP-RQ-006 | Runtime停止後の利用境界 | [RQ-02-015](RQ-02_core-features.md#rq-02-015) |
| SEP-RQ-007 | Runtime終了の再要求 | [RQ-02-016](RQ-02_core-features.md#rq-02-016) |
| SEP-RQ-008 | callback/event中の終了要求 | [RQ-06-005](RQ-06_threading-dispatch.md#rq-06-005) |
| SEP-RQ-009 | 終了要求の受付と完了 | [RQ-06-006](RQ-06_threading-dispatch.md#rq-06-006) |
| SEP-RQ-010 | 停止したUI loopへの同期要求 | [RQ-06-007](RQ-06_threading-dispatch.md#rq-06-007) |
| SEP-RQ-011 | ABI F32要求の未対応範囲 | [RQ-04-002](RQ-04_abi-boundary.md#rq-04-002) |
| SEP-RQ-012 | 初期Propertyの値型範囲 | [RQ-02-017](RQ-02_core-features.md#rq-02-017) |
| SEP-RQ-013 | F64の同値と保持結果 | [RQ-02-018](RQ-02_core-features.md#rq-02-018) |
| SEP-RQ-014 | 十進値の範囲と同値 | [RQ-02-019](RQ-02_core-features.md#rq-02-019) |
| SEP-RQ-015 | UTF-8文字列の受渡し結果 | [RQ-04-020](RQ-04_abi-boundary.md#rq-04-020) |
| SEP-RQ-016 | buffer不足時の結果 | [RQ-04-021](RQ-04_abi-boundary.md#rq-04-021) |
| SEP-RQ-017 | error詳細を生成できない失敗 | [RQ-04-022](RQ-04_abi-boundary.md#rq-04-022) |
| SEP-RQ-018 | 利用者定義エラーの自動採番 | [RQ-04-023](RQ-04_abi-boundary.md#rq-04-023) |
| SEP-RQ-019 | 機能照会と互換性 | [RQ-04-024](RQ-04_abi-boundary.md#rq-04-024) |
| SEP-RQ-020 | 購読の初期通知・順序 | [RQ-07-012](RQ-07_binding.md#rq-07-012) |
| SEP-RQ-021 | 同値設定時の検証 | [RQ-02-020](RQ-02_core-features.md#rq-02-020) |
| SEP-RQ-022 | 長期保留の診断設定 | [RQ-07-013](RQ-07_binding.md#rq-07-013) |
| SEP-RQ-023 | 非収束診断と回復 | [RQ-07-014](RQ-07_binding.md#rq-07-014) |
| SEP-RQ-024 | Validator登録条件 | [RQ-02-021](RQ-02_core-features.md#rq-02-021) |
| SEP-RQ-025 | 判定callback中の禁止操作 | [RQ-05-013](RQ-05_ownership-callbacks.md#rq-05-013) |
| SEP-RQ-026 | TextBox入力三値の結果 | [RQ-09-038](RQ-09_ui-scope.md#rq-09-038) |
| SEP-RQ-027 | TextBox自身のProperty検証拒否 | [RQ-09-039](RQ-09_ui-scope.md#rq-09-039) |
| SEP-RQ-028 | IME cancel後の入力理由 | [RQ-09-040](RQ-09_ui-scope.md#rq-09-040) |
| SEP-RQ-029 | IME commit準備失敗 | [RQ-09-041](RQ-09_ui-scope.md#rq-09-041) |
| SEP-RQ-030 | 外部更新と編集中の値 | [RQ-09-042](RQ-09_ui-scope.md#rq-09-042) |
| SEP-RQ-031 | Command実行受付と結果 | [RQ-06-008](RQ-06_threading-dispatch.md#rq-06-008) |
| SEP-RQ-032 | 取消・close・handle破棄の差 | [RQ-06-009](RQ-06_threading-dispatch.md#rq-06-009)、[RQ-06-010](RQ-06_threading-dispatch.md#rq-06-010) |
| SEP-RQ-033 | Detached subtreeの利用 | [RQ-09-043](RQ-09_ui-scope.md#rq-09-043) |
| SEP-RQ-034 | Tree変更の拒否結果 | [RQ-09-044](RQ-09_ui-scope.md#rq-09-044) |
| SEP-RQ-035 | StackPanel公開設定と既定 | [RQ-09-045](RQ-09_ui-scope.md#rq-09-045) |
| SEP-RQ-036 | 有効性とFocusの共通結果 | [RQ-09-046](RQ-09_ui-scope.md#rq-09-046) |
| SEP-RQ-037 | 初期機能の除外とButton未接続 | [RQ-09-047](RQ-09_ui-scope.md#rq-09-047)、[RQ-09-048](RQ-09_ui-scope.md#rq-09-048) |
| SEP-RQ-038 | Focus通知と入力route | [RQ-09-049](RQ-09_ui-scope.md#rq-09-049) |
| SEP-RQ-039 | Platform資源喪失と終了 | [RQ-11-020](RQ-11_platform.md#rq-11-020) |
| SEP-RQ-040 | Accessibility identityと操作境界 | [RQ-10-011](RQ-10_accessibility.md#rq-10-011) |
| SEP-RQ-041 | Accessibility nameの空欄 | [RQ-10-012](RQ-10_accessibility.md#rq-10-012) |
| SEP-RQ-042 | C++の例外無効buildとhandler | [RQ-03-005](RQ-03_language-support.md#rq-03-005) |
| SEP-RQ-043 | C#の明示破棄と競合 | [RQ-03-006](RQ-03_language-support.md#rq-03-006) |
| SEP-RQ-044 | Rust safe APIとpanic限界 | [RQ-03-007](RQ-03_language-support.md#rq-03-007) |
| SEP-RQ-045 | Python明示終了とowner回収 | [RQ-03-008](RQ-03_language-support.md#rq-03-008) |
| SEP-RQ-046 | Python通常終了・例外と保証外 | [RQ-03-009](RQ-03_language-support.md#rq-03-009)、[RQ-03-010](RQ-03_language-support.md#rq-03-010) |
| SEP-RQ-047 | GIL非依存とsupport区分 | [RQ-06-003](RQ-06_threading-dispatch.md#rq-06-003) |
| SEP-RQ-048 | 診断・解析の能力と限界 | [RQ-03-011](RQ-03_language-support.md#rq-03-011) |
| SEP-RQ-049 | 検証状態と正式support | [RQ-11-021](RQ-11_platform.md#rq-11-021) |
| SEP-RQ-050 | artifact選択・identity・取得 | [RQ-12-011](RQ-12_build-distribution.md#rq-12-011)、[RQ-12-012](RQ-12_build-distribution.md#rq-12-012)、[RQ-12-013](RQ-12_build-distribution.md#rq-12-013) |
| SEP-RQ-051 | artifactの信頼と検証失敗 | [RQ-12-014](RQ-12_build-distribution.md#rq-12-014) |
| SEP-RQ-052 | manifestの互換・曖昧性 | [RQ-12-015](RQ-12_build-distribution.md#rq-12-015) |
| SEP-RQ-053 | license/noticeの継承 | [RQ-12-016](RQ-12_build-distribution.md#rq-12-016) |
| SEP-RQ-054 | support lifecycleとEOL記録 | [RQ-12-017](RQ-12_build-distribution.md#rq-12-017)、[RQ-12-018](RQ-12_build-distribution.md#rq-12-018)、[RQ-12-019](RQ-12_build-distribution.md#rq-12-019)、[RQ-12-020](RQ-12_build-distribution.md#rq-12-020) |
| SEP-RQ-055 | Collection stable identity | [RQ-09-050](RQ-09_ui-scope.md#rq-09-050) |
| SEP-RQ-056 | Selection順序と拒否 | [RQ-09-051](RQ-09_ui-scope.md#rq-09-051) |
| SEP-RQ-057 | Collection batchの観測結果 | [RQ-09-052](RQ-09_ui-scope.md#rq-09-052) |
| SEP-RQ-058 | text coverageと編集位置 | [RQ-09-053](RQ-09_ui-scope.md#rq-09-053) |
| SEP-RQ-059 | 性能測定と閾値 | [RQ-13-011](RQ-13_nonfunctional-measurement.md#rq-13-011) |
| SEP-RQ-060 | 文書承認とID移行 | [RQ-16-003](RQ-16_document-governance.md#rq-16-003) |
| SEP-RQ-061 | 通知callbackの例外後の継続 | [RQ-05-014](RQ-05_ownership-callbacks.md#rq-05-014)、[RQ-05-015](RQ-05_ownership-callbacks.md#rq-05-015) |
| SEP-RQ-062 | 単一操作の準備失敗 | [RQ-04-025](RQ-04_abi-boundary.md#rq-04-025)、[RQ-04-026](RQ-04_abi-boundary.md#rq-04-026) |
| SEP-RQ-063 | 同じBindingの重複接続 | [RQ-07-015](RQ-07_binding.md#rq-07-015) |

## RQ-15の統合

| 旧ID（廃止・再利用禁止） | 現行ID |
| --- | --- |
| [RQ-15-001](RQ-15_requirement-decisions.md#rq-15-001) | [RQ-12-010](RQ-12_build-distribution.md#rq-12-010)、[RQ-11-003](RQ-11_platform.md#rq-11-003)、[RQ-11-004](RQ-11_platform.md#rq-11-004) |
| [RQ-15-002](RQ-15_requirement-decisions.md#rq-15-002) | [RQ-01-002](RQ-01_purpose-architecture.md#rq-01-002)、[RQ-11-008](RQ-11_platform.md#rq-11-008) |
| [RQ-15-003](RQ-15_requirement-decisions.md#rq-15-003) | [RQ-11-008](RQ-11_platform.md#rq-11-008)、[RQ-11-009](RQ-11_platform.md#rq-11-009) |
| [RQ-15-004](RQ-15_requirement-decisions.md#rq-15-004) | [RQ-09-002](RQ-09_ui-scope.md#rq-09-002)、[RQ-09-003](RQ-09_ui-scope.md#rq-09-003) |
| [RQ-15-005](RQ-15_requirement-decisions.md#rq-15-005) | [RQ-09-005](RQ-09_ui-scope.md#rq-09-005)、[RQ-09-053](RQ-09_ui-scope.md#rq-09-053)、[RQ-09-054](RQ-09_ui-scope.md#rq-09-054) |
| [RQ-15-006](RQ-15_requirement-decisions.md#rq-15-006) | [RQ-09-005](RQ-09_ui-scope.md#rq-09-005)、[RQ-09-038](RQ-09_ui-scope.md#rq-09-038)、[RQ-09-040](RQ-09_ui-scope.md#rq-09-040)、[RQ-09-042](RQ-09_ui-scope.md#rq-09-042)、[RQ-09-055](RQ-09_ui-scope.md#rq-09-055)、[RQ-09-056](RQ-09_ui-scope.md#rq-09-056) |
| [RQ-15-007](RQ-15_requirement-decisions.md#rq-15-007) | [RQ-07-008](RQ-07_binding.md#rq-07-008)、[RQ-07-013](RQ-07_binding.md#rq-07-013)、[RQ-07-014](RQ-07_binding.md#rq-07-014)、[RQ-03-011](RQ-03_language-support.md#rq-03-011) |
| [RQ-15-008](RQ-15_requirement-decisions.md#rq-15-008) | [RQ-07-010](RQ-07_binding.md#rq-07-010)、[RQ-09-005](RQ-09_ui-scope.md#rq-09-005)、[RQ-02-021](RQ-02_core-features.md#rq-02-021)、[RQ-05-013](RQ-05_ownership-callbacks.md#rq-05-013)、[RQ-07-016](RQ-07_binding.md#rq-07-016) |
| [RQ-15-009](RQ-15_requirement-decisions.md#rq-15-009) | [RQ-06-004](RQ-06_threading-dispatch.md#rq-06-004)、[RQ-06-008](RQ-06_threading-dispatch.md#rq-06-008)、[RQ-06-009](RQ-06_threading-dispatch.md#rq-06-009)、[RQ-06-010](RQ-06_threading-dispatch.md#rq-06-010)、[RQ-06-011](RQ-06_threading-dispatch.md#rq-06-011) |
| [RQ-15-010](RQ-15_requirement-decisions.md#rq-15-010) | [RQ-07-005](RQ-07_binding.md#rq-07-005)、[RQ-07-006](RQ-07_binding.md#rq-07-006)、[RQ-07-009](RQ-07_binding.md#rq-07-009) |
| [RQ-15-011](RQ-15_requirement-decisions.md#rq-15-011) | [RQ-04-006](RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-007](RQ-04_abi-boundary.md#rq-04-007)、[RQ-04-022](RQ-04_abi-boundary.md#rq-04-022)、[RQ-04-023](RQ-04_abi-boundary.md#rq-04-023)、[RQ-04-027](RQ-04_abi-boundary.md#rq-04-027) |
| [RQ-15-012](RQ-15_requirement-decisions.md#rq-15-012) | [RQ-04-009](RQ-04_abi-boundary.md#rq-04-009)、[RQ-04-024](RQ-04_abi-boundary.md#rq-04-024)、[RQ-04-028](RQ-04_abi-boundary.md#rq-04-028) |
| [RQ-15-013](RQ-15_requirement-decisions.md#rq-15-013) | [RQ-12-007](RQ-12_build-distribution.md#rq-12-007)、[RQ-03-011](RQ-03_language-support.md#rq-03-011)、[RQ-03-012](RQ-03_language-support.md#rq-03-012) |
| [RQ-15-014](RQ-15_requirement-decisions.md#rq-15-014) | [RQ-12-004](RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](RQ-12_build-distribution.md#rq-12-006)、[RQ-12-008](RQ-12_build-distribution.md#rq-12-008)、[RQ-12-016](RQ-12_build-distribution.md#rq-12-016)、[RQ-12-021](RQ-12_build-distribution.md#rq-12-021) |
| [RQ-15-015](RQ-15_requirement-decisions.md#rq-15-015) | [RQ-11-001](RQ-11_platform.md#rq-11-001)、[RQ-11-002](RQ-11_platform.md#rq-11-002)、[RQ-11-021](RQ-11_platform.md#rq-11-021)、[RQ-12-017](RQ-12_build-distribution.md#rq-12-017)、[RQ-12-018](RQ-12_build-distribution.md#rq-12-018)、[RQ-12-019](RQ-12_build-distribution.md#rq-12-019)、[RQ-12-020](RQ-12_build-distribution.md#rq-12-020) |
| [RQ-15-016](RQ-15_requirement-decisions.md#rq-15-016) | [RQ-12-004](RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](RQ-12_build-distribution.md#rq-12-006)、[RQ-12-008](RQ-12_build-distribution.md#rq-12-008)、[RQ-12-011](RQ-12_build-distribution.md#rq-12-011)、[RQ-12-012](RQ-12_build-distribution.md#rq-12-012)、[RQ-12-013](RQ-12_build-distribution.md#rq-12-013)、[RQ-12-014](RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](RQ-12_build-distribution.md#rq-12-015) |
| [RQ-15-017](RQ-15_requirement-decisions.md#rq-15-017) | [RQ-12-010](RQ-12_build-distribution.md#rq-12-010) |
| [RQ-15-018](RQ-15_requirement-decisions.md#rq-15-018) | [RQ-10-001](RQ-10_accessibility.md#rq-10-001)、[RQ-10-002](RQ-10_accessibility.md#rq-10-002)、[RQ-10-003](RQ-10_accessibility.md#rq-10-003)、[RQ-10-004](RQ-10_accessibility.md#rq-10-004)、[RQ-10-005](RQ-10_accessibility.md#rq-10-005)、[RQ-10-006](RQ-10_accessibility.md#rq-10-006)、[RQ-10-007](RQ-10_accessibility.md#rq-10-007)、[RQ-10-008](RQ-10_accessibility.md#rq-10-008)、[RQ-10-009](RQ-10_accessibility.md#rq-10-009)、[RQ-10-011](RQ-10_accessibility.md#rq-10-011)、[RQ-10-012](RQ-10_accessibility.md#rq-10-012)、[RQ-10-013](RQ-10_accessibility.md#rq-10-013) |
| [RQ-15-019](RQ-15_requirement-decisions.md#rq-15-019) | [RQ-14-002](RQ-14_delivery-sequence.md#rq-14-002) |
| [RQ-15-020](RQ-15_requirement-decisions.md#rq-15-020) | [RQ-14-003](RQ-14_delivery-sequence.md#rq-14-003)、[RQ-14-005](RQ-14_delivery-sequence.md#rq-14-005) |
| [RQ-15-021](RQ-15_requirement-decisions.md#rq-15-021) | [RQ-14-010](RQ-14_delivery-sequence.md#rq-14-010)、[RQ-14-012](RQ-14_delivery-sequence.md#rq-14-012) |
| [RQ-15-022](RQ-15_requirement-decisions.md#rq-15-022) | [RQ-14-007](RQ-14_delivery-sequence.md#rq-14-007)、[RQ-14-008](RQ-14_delivery-sequence.md#rq-14-008) |

## 公開既定値・範囲

| 項目 | 正式ID |
| --- | --- |
| 初期配置の省略値 | [RQ-09-057](RQ-09_ui-scope.md#rq-09-057) |
| 初期有効状態 | [RQ-09-058](RQ-09_ui-scope.md#rq-09-058) |
| 初期focusable値 | [RQ-09-059](RQ-09_ui-scope.md#rq-09-059) |
| 初期visibility範囲 | [RQ-09-060](RQ-09_ui-scope.md#rq-09-060) |
| 直接Click範囲 | [RQ-09-061](RQ-09_ui-scope.md#rq-09-061) |

## 既存IDの維持

RQ-15以外の165件はIDを維持した。RQ-08-001～008は従前どおり廃止済みで、再利用しない。旧本文のAPI・方式は対応する設計判断へ分離し、利用者可視の保証は上記IDに保持した。

## 今回の承認履歴

| 日付 | 対象版・範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.0 / 正式要求ID移行 | 利用者 | 全レビュー案の反映指示により承認。実装・試験実行の完了とは区別する |

## 今回の改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 |
| --- | --- | --- | --- |
| 1.0 | 2026-10-17 | 要求と実現手段・検証・作業記録を分離し、正式IDと参照を対応付けた | Codex |
