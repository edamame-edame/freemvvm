# SD-04 防御条件の検証境界

- 文書版: 0.2
- 状態: 責務分離・防御範囲承認済み（2026-10-17）。詳細parametersはSD-06/07へ適用済み、実環境値は未取得
- 上位根拠: [RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)、[RQ-16-004](../11_requirements/RQ-16_document-governance.md#rq-16-004)
- 基本設計: [ビルド・配布：package・artifact・信頼境界](../12_design/build/packages-artifacts.md)

<a id="sd-04"></a>

対策ごとに機能要求の観測と内部防御の不変条件を分ける。これは検証を設計するための境界対応で、実行済み証拠ではない。

| 防御 | 外部要求の検証 | 局所unitの検証 |
| --- | --- | --- |
| 署名/trust/digest | 改ざん・未知鍵・失効鍵・不一致を拒否してlink/loadしない | 署名対象byteの同一性、keyset世代照合、成功型を不正に構成できないこと |
| schema/parser | 未知major/critical・duplicate・曖昧manifestの拒否 | raw parseと意味処理の分離、optional field保持、境界値/資源limit |
| cache/staging | 異target/異digestのartifactを利用しない | 未検証/verified領域隔離とatomic publish、open objectの同一性 |
| 所有/context | 解除後不再invoke、破棄失敗後に参照を失わない | pin/lease/in-flight線形化、destroy一回、lock外の言語値drop |

実caseは要求test台帳、fault injectionとcommand/SHAはunit-test台帳に記す。expiry/staleはRQ-12-023とSD-07の承認済み期間をoracleとし、実際の鍵/時刻/失効配布証拠を未取得のままpassにしない。

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1 / SD-04の責務・防御範囲 | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | SD-04の責務・防御範囲 | Codex |

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
