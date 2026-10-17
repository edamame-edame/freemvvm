# 適用前記録 / 01-handle-registry.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [01-handle-registry.md](../../13_detail_design/01-handle-registry.md)

---

# DD-01 Handle登録と検証

- 文書版: 0.2
- 状態: DD-HND-01～06承認済み（2026-09-27）。要求・設計・テスト境界の改訂を利用者承認済み（2026-10-16）
- 作成日: 2026-09-27
- 更新日: 2026-10-16
- 対象: 非ゼロhandle tokenの発行、登録、型・Runtime検証、`retain` / `release`
- 上位出典: [Runtime契約 R1～R4](runtime-contract.md)、[ABI仕様 A1～A32](abi-spec.md)、[ADR-0001](../../04_approval/adr-0001-rust-core.md)

本書の[DD-HND-01](#dd-hnd-01)、[DD-HND-02](#dd-hnd-02)、[DD-HND-03](#dd-hnd-03)、[DD-HND-04](#dd-hnd-04)、[DD-HND-05](#dd-hnd-05)、[DD-HND-06](#dd-hnd-06)は2026-09-27に利用者承認済みである。承認は本書の詳細設計判断に限る。C宣言や擬似コードは配布用ヘッダ、実装済みAPIを意味しない。

## 1. 範囲

DD-01では、公開handle tokenがどのobjectと種類を指すかを登録し、API入口で検証する処理を定める。object本体の内部参照、Runtime終了時の破棄、UI threadへの破棄dispatchは本書の対象外とする。本書は[Runtime契約 R1～R4](runtime-contract.md)と[ABI契約 A1～A32](abi-spec.md)を具体化する。

対象要求: [RQ-02-001](../../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-04-004](../../11_requirements/RQ-04_abi-boundary.md#rq-04-004)、[RQ-04-005](../../11_requirements/RQ-04_abi-boundary.md#rq-04-005)、[RQ-05-001](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-001)、[RQ-05-002](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)、[RQ-05-003](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-003)、[RQ-06-003](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)。

| 項目 | DD-01で扱うこと | 後続へ送ること |
|---|---|---|
| token | 発行の一意性、無効値、枯渇時 | ABI全体のversionや追加handle型 |
| 登録情報 | handleの種類、Runtime所属、object識別 | object内部参照の構造、破棄順序 |
| 検証 | 無効・異種・別Runtimeの判定 | `WRONG_THREAD` / `CLOSED`との全API共通優先順位 |
| retain/release | 公開tokenごとの独立性、同時呼出しの意味 | 最終参照後の実破棄とshutdown連携 |

## 2. 承認済み判断

以下のDD-HND-01～06は2026-09-27承認済み。

| ID | 承認内容 | 根拠・補足 | 状態 |
|---|---|---|---|
| <a id="dd-hnd-01"></a>DD-HND-01 | 発行器はプロセス内で共有する単調増加の64-bit token番号を使う。0は発行しない。成功済みtokenはprocess終了まで再利用せず、カウンタを周回させない。 | [Runtime契約 R1](runtime-contract.md)の非ゼロ・非再利用・枯渇時失敗を具体化する。番号の内部表現はABIから観測できない。 | 承認済み（2026-09-27） |
| <a id="dd-hnd-02"></a>DD-HND-02 | 登録表はtokenから、handle kind、Runtime identity（Runtimeを持たないError等はnone）、対象objectの内部identityおよび公開参照レコードを引けるようにする。C ABIへ内部pointerを露出しない。 | [RQ-04-004](../../11_requirements/RQ-04_abi-boundary.md#rq-04-004)のopaque handleと[Runtime契約 R1](runtime-contract.md)を登録表で具体化する。 | 承認済み（2026-09-27） |
| <a id="dd-hnd-03"></a>DD-HND-03 | Runtimeを指定するAPIは、入力tokenの有効性を検証した後、期待kindを照合し、kind一致時にRuntime所属を照合する。Runtimeを引数に取らないAPIは所属照合を行わない。 | 検証はtokenの登録状態、kind、Runtime所属の順に行う。status/errorの外部分類と数値割当は本書では規定しない。関連要求は[RQ-04-006](../../11_requirements/RQ-04_abi-boundary.md#rq-04-006)と[RQ-04-007](../../11_requirements/RQ-04_abi-boundary.md#rq-04-007)。 | 承認済み（2026-09-27） |
| <a id="dd-hnd-04"></a>DD-HND-04 | `retain`と`release`は登録表上で線形化できる操作とする。retainが先に成功した場合、新tokenは元tokenと独立して有効。releaseが先に成功した場合、そのtokenへの後続retainは失敗する。同一tokenの競合releaseは最大一回だけ成功し、別tokenを無効化しない。 | [Runtime契約 R1](runtime-contract.md)、[ABI仕様 2.2](abi-spec.md#22-所有権を伴う関数)、[RQ-05-002](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-002)の所有参照契約を同時呼出しにも具体化する。 | 承認済み（2026-09-27） |
| <a id="dd-hnd-05"></a>DD-HND-05 | 表の同期区間ではtokenと登録レコードの整合だけを更新し、object破棄、callback、UI操作、ユーザーコードを実行しない。必要な内部参照を登録表外で確保してから同期を解く。 | [RQ-06-003](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)に対応し、競合中の再入やlock保持中の破棄を避ける。 | 承認済み（2026-09-27） |
| <a id="dd-hnd-06"></a>DD-HND-06 | token番号枯渇時はtokenを発行せず失敗する。登録表の割当失敗時は部分登録を残さない。 | [Runtime契約 R1](runtime-contract.md)の枯渇時失敗を発行処理に適用する。status/error codeの割当は本書では規定しない。 | 承認済み（2026-09-27） |

## 3. 操作別契約

| 操作 | 成功時 | 失敗時・境界 |
|---|---|---|
| object登録 | 新しいtokenと登録レコードを一体で公開し、所有tokenを返す | tokenまたは表の割当失敗時に部分登録を残さない。出力handleは0 |
| handle検証 | live tokenのkindと必要ならRuntime identityを返す | 失敗時に対象objectの操作へ進まない |
| `handle_retain` | 同じobjectへの別tokenを登録し、入力tokenは引き続き有効 | 出力tokenは0。入力tokenを消費しない |
| `handle_release` | 入力tokenの公開参照レコードを一度だけ消費 | 無効・解放済みtokenは失敗する。別tokenや別objectの公開参照は変化しない |

ここでいう「公開」はABI呼び出しから観測可能になることを指す。object登録、retain、releaseに伴う内部参照の増減とobjectの実破棄は本書の対象外とする。

## 4. 検証の不変条件

1. 0は常に無効であり、live tokenとして登録されない。
2. 同じprocess内で二つの所有参照に同じtoken値を割り当てない。`retain`は必ず異なるtokenを返す。
3. 解放済みtoken値を別objectまたは別Runtimeへ割り当て直さない。
4. handle値だけからkind、Runtime、object addressを推定できない。
5. 入力検証に失敗したAPIは、対象objectの状態を変更しない。
6. kind検証を必要とするAPIは、kind確認前に型固有のobjectを参照しない。

## 5. 承認記録

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-09-27 | DD-HND-01～06 | 利用者 | 「内容を承認します」と確認。各詳細設計判断を承認 |
| 2026-10-16 | DD-01の要求・設計・テスト境界整理 | 利用者 | 改訂案を承認。DD-01文書と関連計画へ適用 |

## 6. 改訂履歴

| 版 | 日付 | 変更 | 承認状態 |
|---|---|---|---|
| 0.1 | 2026-09-27 | DD-HND-01～06を作成し、利用者承認 | 承認済み |
| 0.2 | 2026-10-16 | 要求根拠の直接リンク、設計内容と要求/API結果の分離、確認caseの移管、下流参照の除去 | 利用者承認済み |
