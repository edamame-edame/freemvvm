# 詳細設計

- 文書版: 1.1
- 状態: 掲載19本文と本索引の1.1全文は承認済み（利用者、2026-10-17）。実装・試験状態は別管理
- 更新日: 2026-10-17

要求をソフトウェアへ反映するための設計正本。各文書に要求IDと上位基本設計を示し、class/interface・API・処理手順・所有・状態遷移をまとめる。文書名と版で承認し、設計独自IDによる項目追跡は行わない。

上位根拠: [要求仕様](../11_requirements/reqirements-specification.md)、[全体アーキテクチャと分野の責務](../12_design/architecture.md)、[ADR](../04_approval/adr-0001-rust-core.md)。セキュア設計は[専用文書](../14_secure_design/00-security-design-map.md)で管理する。

## イベントハンドリング設計

| 文書 | 範囲 |
| --- | --- |
| [イベントハンドリング：callbackと再入](events/callbacks.md) | callback実行と登録のライフサイクル |
| [イベントハンドリング：通知とBinding伝播](events/notifications-binding.md) | Property更新と通知配信、Binding graphと因果連鎖、診断・回復とerror catalog |
| [イベントハンドリング：非同期Command](events/commands.md) | Command実行・完了・取消し |

## ランタイム/オブジェクト管理設計

| 文書 | 範囲 |
| --- | --- |
| [ランタイム：実行基盤と終了](runtime-objects/runtime.md) | Runtimeと登録資源の所有、UI dispatcherとhost loop、Host境界・終了要求・完了観測 |
| [オブジェクト管理：handle・値・error](runtime-objects/objects-values.md) | handle登録と検証、値とerrorの境界表現、数値変換とConverter |
| [オブジェクト管理：CollectionとSelection](runtime-objects/collections.md) | Model・batch・snapshot・Selection |
| [共通C ABI：宣言・検査・互換性](runtime-objects/abi.md) | 共通型と操作宣言、共通status・UI loop・終了境界、API表・feature・検査順・互換性 |

## 言語ラッパーレイヤー設計

| 文書 | 範囲 |
| --- | --- |
| [言語ラッパー：C++](wrappers/cpp.md) | 所有型・callback・例外・thread、公開API・handler・cleanup同期 |
| [言語ラッパー：C#](wrappers/csharp.md) | interop・所有参照・Dispose、公開API・call lease・native load |
| [言語ラッパー：Rust](wrappers/rust.md) | 所有型・借用・Drop・panic境界、公開API・registry・Future |
| [言語ラッパー：Python](wrappers/python.md) | 所有参照・callback root・通常終了、公開API・cleanup・handler寿命 |
| [言語ラッパー：callback解析・lint・MCP](wrappers/analyzer.md) | 解析IR・診断・workspace境界 |

## UIクラス設計

| 文書 | 範囲 |
| --- | --- |
| [UIクラス：Tree・Layout・Geometry](ui/tree-layout.md) | UI要素の所属と寿命、Measure・Arrange・dirty処理、座標・Tree所属・操作API |
| [UIクラス：InputとFocus](ui/input-focus.md) | 入力配送・Focus状態・IME接続 |
| [UIクラス：TextBox・Validation・IME編集](ui/text-editing.md) | PropertyとView入力のValidation、TextBox編集とIME session、編集API・文字位置・失敗回復 |
| [UIクラス：Window・基本Control・次段階Control](ui/controls.md) | 基本Control、Windowの操作とclose request、次段階Control |
| [UIクラス：WebViewエンジンプラグイン詳細設計](ui/webview.md) | DOM Arena、Markdown/HTML/CSSパーサ、WebLayout、Script VM、MVVM/ABI接続 |

## プラットフォーム設計

| 文書 | 範囲 |
| --- | --- |
| [プラットフォーム：抽象化と共通protocol](platform/abstraction.md) | Host・Window・Rendererの抽象化、Accessibilityの共通Treeとaction |
| [プラットフォーム：OS・描画・text・IME内部レイヤー](platform/internal-layers.md) | Scene・Window peer・描画backend、Text engine・IME・OS Accessibility |

## ビルド・配布設計

| 文書 | 範囲 |
| --- | --- |
| [ビルド・配布：toolchain・profile・package](build/toolchain-packages.md) | ビルドと配布の責務・artifact検証、最低版・target profile・toolchain lock、package・loader・artifact取得 |

## 文書運用

旧文書はローカルarchiveへ退避し、Git対象から除外した。旧IDの互換入口・追跡台帳は設けない。今後の改訂は要求との対応と文書版全体でレビューし、実装・test実行状態とは別に記録する。

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論 |
| --- | --- | --- | --- |
| 2026-10-17 | 入口1.0と掲載19文書の各1.0全文 | 利用者 | 全部承認。実装・試験合格を意味しない |

## クラス設計の補足

図は設計の特定に必要な箇所へ選択して補強する。今回の1.1では、[Controlの機能委譲](ui/controls.md)、[入力・Focus・IMEの接続](ui/input-focus.md)、[CoreとOS実装の境界](platform/abstraction.md)へコンポーネント図を追加し、[Collectionの候補とsnapshot](runtime-objects/collections.md)、[Commandのworker報告](events/commands.md)、[TextBoxの編集値と確定値](ui/text-editing.md)へデータフロー図を追加した。クラス図は型と所有、コンポーネント図は利用・委譲先、データフロー図はデータの保持先と反映条件を示す。全文書に三種類の図を揃える運用にはしない。

掲載19本文に型の責務・保持データ・所有/寿命・主要メソッド、縦向きのclass図と呼出し/失敗境界を追加した。既承認1.0の本文を維持し、補足を含む各1.1全文は利用者承認済みである。CoreはRustのstruct/trait/composition、言語facadeは各言語の型、C ABIはhandle/固定幅値の境界として区別する。既存APIの呼出し規則は各本文を正本とし、補足だけから新しい公開APIを作らない。

## 1.1承認の記録

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 本索引と掲載19本文の1.1全文 | 利用者 | 「OK,テスト文書作りましょうか」により、クラス補足と選択的な図の補強を含む1.1を承認。テスト文書作成を開始する。試験合格を意味しない |
