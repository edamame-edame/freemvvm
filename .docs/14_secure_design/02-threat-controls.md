# SD-02 脅威と対策の対応

- 文書版: 0.2
- 状態: 責務分離・防御範囲承認済み（2026-10-17）。詳細parametersはSD-06/07へ適用済み、実環境値は未取得
- 上位根拠: [RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)、[RQ-16-004](../11_requirements/RQ-16_document-governance.md#rq-16-004)
- 基本設計: [ビルド・配布：package・artifact・信頼境界](../12_design/build/packages-artifacts.md)

<a id="sd-02"></a>

| 脅威 | 防御の責務 | 実現箇所 |
| --- | --- | --- |
| manifest/native改ざん | 署名対象payloadとnative digestを検証成功recordへ束縛する | SecurityVerifier / staging publisher |
| 鍵差替え・失効無視 | 独立trust rootから認証済みkeysetを解決し、失効状態を確認する | TrustResolver |
| target/profile/ABI混同 | consumer要求とmanifest適合条件を照合してからlink/loadする | ArtifactSelector |
| 曖昧parse・未知必須項目 | 重複key等を拒否し、一意payloadを構成、署名後にschemaを評価する | ManifestParser |
| 所有競合・再入悪用 | validation pin、Active/in-flight、call lease、token/ticket handoff | DD registry/closure/wrapper owner |
| 解析結果の取り違え | workspace/config/input generation照合、世代単位の結果公開 | DiagnosticsStore |
| 資源枯渇・秘密混入 | SD-06のparser上限、DD-27のdiagnostic budget/opt-in、SD-07のprivate stagingとrole権限で制限する | 方式適用済み・実装/運用証拠は未取得 |

対策の内部上限を利用者の新しい拒否条件として公開する場合はRQ変更として扱う。

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1 / SD-02の責務・防御範囲 | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | SD-02の責務・防御範囲 | Codex |

## 具体化したセキュア実現

[署名envelope・schema・parser](06-manifest-wire-verifier.md)、[鍵管理・失効・offline・staging](07-trust-key-staging.md)を個別方式の正本とする。解析器のMCP/Clang入力境界は、登録workspace/readonly SDKの限定・動的plugin拒否・networkなし解析process・資源budgetで実現する。実鍵、KMS環境、digest、監査/試験結果は未取得として運用台帳へ記録する。

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.2 / SD-06/07への個別方式と解析入力境界の具体化 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.2 | 2026-10-17 | SD-06/07への個別方式と解析入力境界の具体化を正本へ適用し、旧判断との対応を保持 | Codex |
