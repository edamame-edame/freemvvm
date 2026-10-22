# 要求・設計分離 / 全案適用結果

- 文書版: 1.0
- 状態: 正本適用済み（2026-10-17、利用者の全案反映指示）

## 適用範囲

| 対象 | 結果 |
| --- | --- |
| 要求 | 15章を改訂。249現行ID＝165保持＋84追加。RQ-15の22旧IDを統合廃止 |
| BD/DD | 42 BDと23既存DDを更新。62残件案＋既適用DD00～02の要求根拠を反映。DD23は要求testへ移管済みのまま |
| API | C宣言をDD-06分冊へ移管。各機能・言語DDのAPI/所有表と図を保持。独立API工程は設けない |
| SD | SD00～05に資産・信頼境界・脅威/防御・検証境界・DD整合の分離範囲を配置 |
| 要求test | 249 RQを90論理caseへ対応付け、要求別assertionとcase逆引きを更新。未実装・未実行は維持 |
| 承認 | 166レビュー対象の採用範囲・日付・適用先を承認台帳へ記録 |
| 履歴 | 適用前65設計文書を日付付き作業記録へ保存。承認・改訂履歴を追記 |

## 採用した選択と残る境界

| 論点 | 採用・扱い |
| --- | --- |
| 停止UI loop | host停止通知で同期waitを未完了として解除。任意timeout値やworker UI破棄は追加しない |
| F32 | 現行ABI要求を維持し、利用APIを設計gapとして計画化 |
| TextBox拒否 | 案B：編集候補・理由を保持し確定値/VMは旧値 |
| IME Cancel | 開始時editing/caret/selectionにinput state/reasonも復元 |
| IME準備失敗 | 回復可否・再試行を識別。Core退避かAdapter再送かはprofile別未設計 |
| Button/既定値 | Commandなしは表示・無操作。配置/enabled/focusable既定とvisibility/直接Click範囲をRQへ明記 |
| Python | 管理host登録を前提に通常終了を集約。個別ownerの回収と全Runtime終了を分離 |
| Security | 既承認algorithmをSDへ保持。schema/envelope/期限/鍵管理等は未設計 |

## 正本への入口

- [要求仕様](../11_requirements/reqirements-specification.md)
- [正式要求ID移行](../11_requirements/requirements-separation-id-migration.md)
- [基本設計](../12_design/design-overview.md)
- [DD ABI宣言分冊](../13_detail_design/06-abi-declarations.md)
- [SD索引](../14_secure_design/00-security-design-map.md)
- [要求-test対応](../30_release_test/requirement-test-matrix.md)
- [要求別assertion](../30_release_test/requirements-test-assertions.md)
- [166件の承認台帳](requirements-design-separation-review/approval-register.md)
- [未設計の後続課題](requirements-design-separation-follow-up.md)

## 検査結果

対象273文書の相対リンク14,969件、Markdown表1,131個、fenceの対応を検査し、エラー0件だった。現行249 RQと90 logical caseの対応で未対応要求0件・孤立case0件・未定義case参照0件を確認した。全既存の判断IDも本文または移管先に保持した。`git diff --check`は成功した。Mermaidは構文fence/参照を確認し、画像としての描画は未実施。製品の実装・試験実行は今回の作業対象ではない。

## 2026-10-17 適用結果

要求・設計・試験・運用の所管移管を正本へ適用した。未設計事項は明示して引き継いだ。

### 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全レビュー案の反映 | 利用者 | 「すべて反映して下さい。」により採用・正本適用を承認 |

### 改訂履歴

| 版 | 日付 | 内容 |
| --- | --- | --- |
| 1.0 | 2026-10-17 | 承認と正本適用結果を追記。作成時点の比較・引用を保持 |

## 追加照合の差異処理

A20の旧本文はValidator内shutdownをUI変更と一括禁止していたが、BD-36のcallback内終了要求許可と整合しなかった。RQ-05の判定callback条件に終了要求受付の例外を明記し、DD-04/09とABI分冊は「受付record登録 → callback終了 → UI cleanup」へ統一した。直接のUI変更・自己解除は禁止を維持する。今回承認されたSEP-RQ-008/009/025の整合として記録する。

ABI分冊のget_versionは既承認DD-ABI-02のMAJOR/MINOR/PATCH署名へ整合し、UI_PENDING=14を既承認catalogから収録した。元の2出力署名は適用前履歴に保持した。
