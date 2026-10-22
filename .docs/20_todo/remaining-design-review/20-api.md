# 未設計レビュー / 共通API表・feature・status優先順・互換移行

- 文書版: 0.2
- 状態: 全判断承認済み・正本適用済み（2026-10-17）
- 判断ID: REM-API-01～07
- 入口: [全件索引](index.md)
- 根拠RQ: [RQ-01-002](../../11_requirements/RQ-01_purpose-architecture.md#rq-01-002)、[RQ-04-006](../../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-021](../../11_requirements/RQ-04_abi-boundary.md#rq-04-021)、[RQ-04-024](../../11_requirements/RQ-04_abi-boundary.md#rq-04-024)、[RQ-04-028](../../11_requirements/RQ-04_abi-boundary.md#rq-04-028)、[RQ-05-001](../../11_requirements/RQ-05_ownership-callbacks.md#rq-05-001)、[RQ-16-004](../../11_requirements/RQ-16_document-governance.md#rq-16-004)
- 具体化するBD: [BD-01](../../12_design/BD-01_value-types-upper-design.md)、[BD-27](../../12_design/BD-27_error-object-upper-design.md)、[BD-28](../../12_design/BD-28_abi-release-version-upper-design.md)、[BD-36](../../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md)
- 適用対象DD/SD: [DD-01](../../13_detail_design/01-handle-registry.md)、[DD-05](../../13_detail_design/05-boundary-values-errors.md)、[DD-06](../../13_detail_design/06-abi-surface.md)、[DD-07](../../13_detail_design/07-property-notification.md)、[DD-18](../../13_detail_design/18-wrapper-cpp.md)、[DD-19](../../13_detail_design/19-wrapper-csharp.md)、[DD-20](../../13_detail_design/20-wrapper-rust.md)、[DD-21](../../13_detail_design/21-wrapper-python.md)

本書は承認したレビュー記録である。推奨案と代替理由は提案時点の比較として保持する。現行の方式・正式判断IDは[DD-40: 共通API表・feature・status優先順・互換移行](../../13_detail_design/40-api-inventory.md)を正本とする。採用済みでも本文に将来候補と記した範囲は初期必須へ昇格しない。

## 判断案

| 判断ID | 推奨案・具体的な選択 | 代替と選択理由 | 承認区分 |
| --- | --- | --- | --- |
| REM-API-01 | DD内API表を共通schemaで統合し、入力/出力、owned/borrowed、thread/context、commit、failure ownership、feature/versionを必須fieldにする。 | 独立API設計工程は復活させない。 | DD管理 |
| REM-API-02 | 失敗優先順はrequired output妥当性→token→kind→Runtime→thread/context→lifecycle→値/容量→資源準備。API固有例外は表に列挙する。 | 状態検査順が違うと同じ不正入力で言語間のstatusが変わる。 | DD |
| REM-API-03 | feature候補IDは0 invalid、1 core.mvvm.v1、2 values.f64_decimal.v1、3 numeric.f32_bridge.v1、4 host.shutdown_receipt.v1、5 collection.v1、6 textbox.snapshot.v1。 | 既存数値の割当がないことを適用時に再照合し、公開済み値を置換しない。 | DD |
| REM-API-04 | has_featureは機能群の全symbolと外部契約が提供されるときだけtrue。build flagの存在だけで有効としない。 | 未知IDは既承認のUI_OK+falseを保持する。 | DD |
| REM-API-05 | global snapshot getterはowned immutable値、UI object getterはUI専用。bufferの2-call readはlive getterなら同snapshot保証なし、snapshot getterなら固定。 | 容量照会後の内容変化を曖昧にしない。 | DD |
| REM-API-06 | borrowed callback引数の保存にはretain/copyが必要。新owned outを受け取るwrapperはticketを先に準備する。 | 自動cleanupが失敗する経路でも新ownerを管理不能にしない。 | DD |
| REM-API-07 | 既存ABI1は追加symbolで拡張し、旧2出力get_version等の履歴と既承認3出力版を移行記録で区別する。初回公開前の宣言統合gateを設ける。 | まだ配布していないAPIのreview版を実装済み互換基準と誤認しない。 | DD/Release管理 |

<a id="rem-api-01"></a>

<a id="rem-api-02"></a>

<a id="rem-api-03"></a>

<a id="rem-api-04"></a>

<a id="rem-api-05"></a>

<a id="rem-api-06"></a>

<a id="rem-api-07"></a>

## 実現設計案

### API inventoryの記録例

| 操作群 | 入力 / 出力 | thread/context | commit / 失敗時所有 |
|---|---|---|---|
| create/retain | Runtime/token borrow → owned token | createはUI、retainは任意 | registry公開が成功点。失敗で元token不変 |
| release | owned token | 任意 | 成功時消費。失敗はcaller/ticket保持 |
| Property get/set | Property borrow + typed scalar/UTF8 | UI、Validator中setter禁止 | copy/validate後commit、out初期化 |
| subscribe/cancel | closure borrow→成功時Coreへ移転 | UI | Active登録/停止が成功点、failed registerはcaller所有 |
| Binding/Validator | object borrow、callback closure | UI、判定callback再入制約 | prepare後登録、releaseとcancelを分離 |
| Command execute/complete | Command borrow、operation ID、error values | execute/completeはUI、report橋渡しは任意 | 受付と業務終端を別に記録 |
| snapshots | UI object borrow→owned immutable snapshot | UI取得、任意thread読取 | 失敗out=0、snapshotは元ownerを延命しない |
| shutdown request | Runtime borrow、mode→owned receipt | 任意、UI callbackは待機しない | 拡張APIは受付済みUI_PENDINGとreceiptを返す。失敗out=0、Closedだけ完了 |
| Tree/Layout/edit | typed UI対象と候補値 | UI | 全準備後state commit、禁止contextで無変更 |

`api-inventory.json`案は`symbol, abi_major, since_minor, feature_id, owner_dd_id, rq_ids, inputs, outputs, thread, forbidden_contexts, commit_point, failure_outputs, ownership_transfer`。C header、ctypes prototype、Rust -sys、P/Invoke宣言をこのschemaから生成し、C++ facadeは型/例外ポリシーを別の生成adapterで結ぶ。実装generationを採用した場合も公開headerが正本設計の宣言と一致するgateを置く。

Callback matrixはPost/Notification/Validator/InputValidator/CommandStart/Destroy/BridgeWakeごとに書く。終了要求の受付はValidator禁止UI変更の例外として扱い、UI cleanupをそのcallback stack内で実行しない。BridgeWakeはscheduleだけ、Destroyはroot/ticket回収だけとし、通常UI APIへ再入しない。

必須out pointerが無効な場合はmemoryの妥当性を安全に検証できるとはせず、NULLの検査とcallerの有効領域契約を区別する。エラーを返す前に読み取ったout以外を部分初期化しないよう、出力shape検査後に規定値をまとめて書く。新featureの数値候補は未公開なので適用時の衝突検査を必須とする。

## 適用結果

全判断を[DD-40](../../13_detail_design/40-api-inventory.md)へ正式IDで適用した。関連要求の変更もRQ本文へ反映した。実装・試験実行は未完了。[全200件の適用表](../remaining-design-application-result.md)で旧新対応を確認できる。

## 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 本書の案作成 | 利用者 | 一括生成指示。方式採用・正本適用は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 現行の未設計箇所を実現案・代替・interfaceへ具体化 | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.2 / 本書の全推奨案と補足の採用・正本適用 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.2 | 2026-10-17 | 本書の全推奨案と補足の採用・正本適用を正本へ適用し、旧判断との対応を保持 | Codex |
