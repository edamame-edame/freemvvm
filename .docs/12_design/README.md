# 基本設計

- 文書版: 1.0
- 状態: 承認済み（利用者、2026-10-17）
- 更新日: 2026-10-17

要求を実現する全体構成・層の責務・主要data/ownership flow・stateをまとめる。設計独自IDで管理せず、要求IDと設計項目を対応付け、文書版全体で承認する。

上位根拠: [要求仕様](../11_requirements/reqirements-specification.md)、[ADR](../04_approval/adr-0001-rust-core.md)。

| 分野 | 文書 |
| --- | --- |
| 全体構成 | [全体アーキテクチャと分野の責務](architecture.md) |
| イベントハンドリング | [イベントハンドリング：通知・入力・Command](events/notifications-commands.md) |
| ランタイム/オブジェクト管理 | [ランタイム：実行context・所有・終了](runtime-objects/runtime.md) |
| ランタイム/オブジェクト管理 | [オブジェクト管理：値・モデル・更新整合性](runtime-objects/model-values.md) |
| ランタイム/オブジェクト管理 | [共通ABI：言語非依存境界と互換性](runtime-objects/abi.md) |
| 言語ラッパー | [言語ラッパー：共通facade・所有変換・解析](wrappers/language-wrappers.md) |
| UIクラス | [UIクラス：Tree・Layout・Focus・Control](ui/tree-layout-controls.md) |
| UIクラス | [UIクラス：TextBox・編集・Validation](ui/text-editing-validation.md) |
| プラットフォーム | [プラットフォーム：抽象化・描画・text・OS接続](platform/abstraction.md) |
| ビルド・配布 | [ビルド環境：toolchain・検証・support](build/toolchain-support.md) |
| ビルド・配布 | [ビルド・配布：package・artifact・信頼境界](build/packages-artifacts.md) |

## 適用範囲と文書運用

既承認の設計内容を分野別に再編集した。再編集版の全文承認は、旧版の方式承認や詳細設計の承認と別に記録する。初期採用・次段階・未採用候補を要求どおり区別し、文書再編で採用範囲を変えない。旧本文・承認履歴・作業記録はローカルarchiveに保存し、Git対象から除外する。

## 承認履歴

| 日付 | 対象文書・版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 入口と掲載11本文の各1.0全文 | 利用者 | 「OK、承認します。」により承認。実装・試験合格を意味しない |
