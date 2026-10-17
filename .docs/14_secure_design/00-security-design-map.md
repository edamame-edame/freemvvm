# セキュア設計の入口と資産対応

- 文書版: 0.4
- 状態: SD-00～07の方式・parameters承認済み（2026-10-17）。実装・採取値・実行証拠は未完了

## 要求根拠

[RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)、[RQ-16-004](../11_requirements/RQ-16_document-governance.md#rq-16-004)。

<a id="sd-00"></a>
## SD-00 資産と所管

| 資産 | 保護対象 | 所有・境界 |
| --- | --- | --- |
| native artifact | digest/identity/target/profile | 受領bytesはuntrusted staging、検証成功recordに束縛して公開 |
| manifest/trust metadata | 署名対象bytes・鍵世代・失効情報 | 構文検査と信頼判断を分離し、既存trust rootから解決 |
| ABI値/handle | 型・長さ・所属・有効期間 | 入口検査、validation pin、copy-before-commit |
| callback/root | userdata所有・in-flight・destroy責任 | 新規entry停止後にdrainし、言語値のdropをlock外へ送る |
| 解析workspace/cache | workspace範囲・入力世代・診断 | request scopeとgenerationで結果を隔離し、stale結果を最新へ公開しない |
| diagnostic/evidence | revision・profile・失敗分類 | 製品message、局所diagnostic、release証拠を別recordとして扱う |

## 章構成

| ID | 文書 |
| --- | --- |
| SD-01 | [信頼境界](01-trust-boundaries.md) |
| SD-02 | [脅威と対策の対応](02-threat-controls.md) |
| SD-03 | [Artifact検証の処理設計](03-artifact-verification.md) |
| SD-04 | [防御条件の検証境界](04-security-verification-boundary.md) |
| SD-05 | [DDとの防御interface対応](05-design-interface-alignment.md) |
| SD-06 | [Manifest wire・署名検証器](06-manifest-wire-verifier.md) |
| SD-07 | [鍵・失効・offline・staging](07-trust-key-staging.md) |

## 具体化境界

今回承認した範囲は資産・信頼境界・脅威/防御の対応とDDとの責務分離である。signature envelope/schema/parser上限はSD-06、鍵管理/expiry/revocation/stagingはSD-07、診断秘匿はSD-02およびDD-27で具体化済み。実鍵・provider環境・digest・試験証跡は未取得として管理する。SD全体の完成や対策実装を意味しない。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果・根拠 |
|---|---|---|---|
| — | secure-designの配置 / 入口0.1 | — | 成果物の追加は利用者指示に基づく。セキュリティ設計内容は未着手・未承認 |

## 改訂履歴

| 版 | 日付 | 変更内容 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 0.1 | 2026-10-12 | secure-designの入口と上位参照候補を作成 | Codex | 配置指示のみ。設計内容は未承認 |
| 0.2 | 2026-10-12 | SDの役割とDD内API具体化との並列関係を反映 | Codex | 利用者の役割指定。個別対策は未承認 |

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.3 / SD-00～05の分離範囲 | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.3 | 2026-10-17 | SD-00～05の分離範囲 | Codex |

## 具体化したセキュア実現

[署名envelope・schema・parser](06-manifest-wire-verifier.md)、[鍵管理・失効・offline・staging](07-trust-key-staging.md)を個別方式の正本とする。解析器のMCP/Clang入力境界は、登録workspace/readonly SDKの限定・動的plugin拒否・networkなし解析process・資源budgetで実現する。実鍵、KMS環境、digest、監査/試験結果は未取得として運用台帳へ記録する。

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.4 / SD-06/07への個別方式と解析入力境界の具体化 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.4 | 2026-10-17 | SD-06/07への個別方式と解析入力境界の具体化を正本へ適用し、旧判断との対応を保持 | Codex |
