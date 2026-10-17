# SD-05 DDとの防御interface対応

- 文書版: 0.2
- 状態: 責務分離・防御範囲承認済み（2026-10-17）。詳細parametersはSD-06/07へ適用済み、実環境値は未取得
- 上位根拠: [RQ-12-014](../11_requirements/RQ-12_build-distribution.md#rq-12-014)、[RQ-12-015](../11_requirements/RQ-12_build-distribution.md#rq-12-015)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-05-006](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-006)、[RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)、[RQ-16-004](../11_requirements/RQ-16_document-governance.md#rq-16-004)
- 基本設計: [ビルド・配布：package・artifact・信頼境界](../12_design/build/packages-artifacts.md)

<a id="sd-05"></a>

| DD側interface | SD側の防御入力/出力 | 保全する所有・state |
| --- | --- | --- |
| Handle registry API | kind/Runtime/context検証済みpin | 検証後call終了まで対象を保持し、失敗でtokenを消費しない |
| Closure invoke/cancel/destroy | Active/in-flightと禁止contextのguard | 新規entry停止後にdrain、userdata一回回収 |
| Wrapper owner/reaper | 事前確保ticketとcall lease | native release失敗でもtokenをticketへ移して管理を継続 |
| Platform Accessibility | 不変snapshotとstale node判定 | queryでFocus変更せず、actionのみUI dispatch |
| Package loader | VerifiedArtifactのみ受け入れる | 検査済みbytesとidentity/target/trust generationを束縛 |
| Analyzer result store | workspace/input/config世代の照合 | 解析途中・旧世代を最新recordとして公開しない |

防御の追加で外部結果が変わる場合はRQ-16-004の差異と変更対象を記録する。DDの機能lifecycleとSDの信頼判断の両方を満たすinterfaceを具体化する。

## 承認履歴（今回）

| 日付 | 版・対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1 / SD-05の責務・防御範囲 | 利用者 | 全案反映指示により承認。実装・実行証跡のない項目は未実装・未実行を維持 |

## 改訂履歴（今回）

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | SD-05の責務・防御範囲 | Codex |

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
