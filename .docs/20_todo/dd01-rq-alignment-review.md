# D01-RQ / DD-01由来の要求境界レビュー案

- 文書版: 0.3
- 状態: 全案反映承認済み・責務分離の正本適用完了（2026-10-17）。未設計課題は別計画へ移管
- 作成日: 2026-10-16
- 対象: [DD-01 Handle登録と検証](../13_detail_design/01-handle-registry.md)
- 作業計画: [D01 / DD-01境界整理](dd01-requirements-design-separation.md)
- 関連要求: [RQ-02](../11_requirements/RQ-02_core-features.md)、[RQ-04](../11_requirements/RQ-04_abi-boundary.md)、[RQ-05](../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06](../11_requirements/RQ-06_threading-dispatch.md)
- 一括レビューへの対応: [全案件索引](requirements-design-separation-review/index.md)、[要求文案SEP-RQ-001～004](requirements-design-separation-review/rq-candidates.md#candidate-001)

2026-10-17の一括作成で、下記4論点の具体的な要求本文案とtest影響を上記へ集約した。本書は個別調査の記録として保持する。候補の採用・RQ本文適用は引き続き未承認。

## 現在の適用結果

[全案適用結果](requirements-design-separation-application-result.md)と[未設計の後続課題](requirements-design-separation-follow-up.md)を参照する。以下の走査・案作成時点の差異や未承認表示は当時の記録であり、現在の承認状態は本節と承認台帳を正本とする。

## 目的と承認境界

DD-01の改訂中に見つかった、利用者が観測するhandleの結果を要求として明記する必要性をレビューする。要求本文は編集せず、新しいRQ IDも割り当てない。候補の採否・RQ本文の改訂・承認履歴の変更は、本レビュー案への承認後に別途適用する。

## 既存要求・上位判断との照合

| 論点 | 現行記載 | 分類候補 |
|---|---|---|
| 参照の独立取得・破棄 | [RQ-05-002](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)は複数の所有参照を独立して破棄できることを要求する | 既存要求で充足。新しい要求は追加しない |
| 解放済み参照の利用 | [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)と[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)は寿命終了後の参照を無効として扱う結果を要求する。[Runtime契約](../12_design/runtime-contract.md)はtokenを再利用しない | 異なるobjectへの誤った再解決を防ぐ外部結果は既存要求の適用範囲内として扱う案。tokenの非再利用方式自体は設計に残す |
| worker threadからのretain/release | [Runtime契約 §3](../12_design/runtime-contract.md)は`retain` / `release`を任意threadとする。[RQ-05-003](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)はthread条件を利用者が判断できることを求めるが、許可threadを列挙していない | 任意threadからの呼出し可否と、最終破棄が呼出し完了より遅れる場合の扱いをRQ-05-003に明記する候補 |
| 同じtokenの二重release | [Runtime契約 §8](../12_design/runtime-contract.md)は二重releaseを検出し、別所有参照へ影響させない条件を示す。[RQ-05-002/003](../11_requirements/RQ-05_ownership-callbacks.md)は複数参照の独立性と破棄後の無効化を定める | 「同一tokenへの重複破棄は失敗し、他の有効参照を変化させない」という利用者可視結果をRQ-05-002/003へ明記する候補 |
| token枯渇時 | [Runtime契約](../12_design/runtime-contract.md)は枯渇時に生成を失敗させる。[RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)は寿命終了後に無効参照となることを求める | 安全に新しい参照を発行できない場合は発行失敗となる外部結果をRQ-04-005またはRQ-05-002へ明記する候補。64-bit幅やstatus/error codeは要求へ追加しない |
| status/errorの区別と割当 | [RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006) / [RQ-04-007](../11_requirements/RQ-04_abi-boundary.md#rq-04-007)はstatusとerror objectを要求する。[DD-06](../13_detail_design/06-abi-surface.md)はhandle error statusと`HANDLE_TOKEN_EXHAUSTED`の割当を定める | error種類の区別を製品保証として求めるかRQで判断する。数値status・domain/codeの割当はAPI詳細設計に残し、本案では変更しない |

## レビューで決める事項

1. worker threadからの`retain` / `release`を利用者向け保証としてRQ-05-003に明記するか。
2. 同じtokenの二重release結果をRQ-05-002/003に明記するか。
3. token枯渇時に新しい参照を発行しないことを要求に明記するか。
4. invalid handle・wrong type・wrong Runtimeの区別を利用者向け保証としてRQ-04-006/007に明記するか。

## 変更しない事項

- RQ-02/04/05/06本文、各文書版、承認履歴
- DD-06の既承認status/error catalog
- 要求test対応表とunit-test方針。要求変更が承認された場合に限り別の更新案を作る
- 単調64-bit発行器、登録表、lock、線形化などの内部方式

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-16 | D01-RQ / DD-01由来の要求境界レビュー案 | — | 提案作成。候補の採否・RQ改訂は未承認 |

## 改訂履歴

| 版 | 日付 | 変更 | 承認状態 |
|---|---|---|---|
| 0.1 | 2026-10-16 | DD-01改訂で分離した外部結果候補を既存RQ・上位判断と照合 | レビュー案 |
| 0.2 | 2026-10-17 | 全案件レビューの要求文案・test影響への入口を追加 | 採用状態は変更なし |

## 2026-10-17 適用結果

全66台帳項目の分離を適用した。D23は要求testへの既存移管を維持し、廃止文書を再作成していない。

### 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全レビュー案の反映 | 利用者 | 「すべて反映して下さい。」により採用・正本適用を承認 |

### 改訂履歴

| 版 | 日付 | 内容 |
| --- | --- | --- |
| 0.3 | 2026-10-17 | 承認と正本適用結果を追記。作成時点の比較・引用を保持 |
