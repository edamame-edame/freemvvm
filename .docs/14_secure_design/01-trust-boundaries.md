# SD-01 信頼境界

- 文書版: 0.2
- 状態: 責務分離・防御範囲承認済み（2026-10-17）。詳細parametersはSD-06/07へ適用済み、実環境値は未取得
- 上位根拠: [RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)、[RQ-16-004](../11_requirements/RQ-16_document-governance.md#rq-16-004)
- 基本設計: [ビルド・配布：package・artifact・信頼境界](../12_design/build/packages-artifacts.md)

<a id="sd-01"></a>

外部入力を次の境界で検査する。検査済みの型/recordのみを次の層へ渡し、受領bytesや生pointerを信頼済みの意味へ昇格させない。

| 入力側 → 処理側 | 検査・所有の切替 |
| --- | --- |
| artifact/catalog → parser/verifier | catalogのkeyは候補情報。独立trust bundleでkey identityを解決する |
| ABI caller → Core | length/kind/Runtime/contextを検査し、保持用copyとpin取得後にtyped操作へ渡す |
| analyzer request → workspace/cache | workspace identityと入力/config generationを付け、別workspaceや旧世代のresultを混用しない |
| OS provider → UI snapshot/action | queryは不変snapshot。actionはUI dispatchしnode identityを再検査する |

同一processの任意pointerを安全にdereferenceできるという保証は追加しない。有効なmemory領域を渡すcaller契約と、値/shape検査を分ける。

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1 / SD-01の責務・防御範囲 | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | SD-01の責務・防御範囲 | Codex |

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
