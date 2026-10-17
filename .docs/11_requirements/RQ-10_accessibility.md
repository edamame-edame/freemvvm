# RQ-10: Accessibility

- 文書版: 0.5
- 状態: 承認済み（利用者、2026-10-17）
- 更新日: 2026-10-17
- 入口: [要求仕様索引](reqirements-specification.md)

## 概要

初期Controlのplatform中立なAccessibility意味情報を定義し、OSのAccessibility機構から利用可能にする。Core上の意味情報を基準とし、OS固有のAPI型や完全な規格適合は要求しない。

本章は提供機能・利用条件・外部から観測できる結果を定める。各IDの対象段階を区別し、将来候補の列挙を採用承認とはしない。

## 要求本文

| 要求ID | 項目 | 要求詳細 | 対象段階 |
| --- | --- | --- | --- |
| <a id="rq-10-001"></a>RQ-10-001 | 初期Control | Window、StackPanel、Text、TextBox、Buttonについて、Coreで定義する意味情報をPlatform経由で利用可能にする。Accessibilityは初期基盤に含める。 | 現行 |
| <a id="rq-10-002"></a>RQ-10-002 | role | 各ControlのroleはControl別の意味情報に示す意味を表す。OS側のroleが異なる語彙を使う場合も、利用者に同じControl種別を示す。 | 現行 |
| <a id="rq-10-003"></a>RQ-10-003 | name | Controlは設定されたAccessibility nameを保持し、対応する対象の意味情報から同じnameを取得できること。 | 現行 |
| <a id="rq-10-004"></a>RQ-10-004 | state | Controlの意味上のstateはControl別の意味情報に示す範囲で取得できる。TextBoxではvalidation状態を含める。 | 現行 |
| <a id="rq-10-005"></a>RQ-10-005 | value | valueが意味を持つControlではControl別の意味情報に示す値を公開する。Textは表示文字列、TextBoxは編集中の文字列を表す。ButtonやContainerに値を要求しない。 | 現行 |
| <a id="rq-10-006"></a>RQ-10-006 | focus | Accessibility上のfocus状態はCoreが管理するactive focus対象と一致する。focus対象でないControlをfocus中として公開しない。 | 現行 |
| <a id="rq-10-007"></a>RQ-10-007 | 操作 | Accessibility actionはControlが実際にサポートする操作に対応する。TextBoxの編集操作とButtonの起動は通常のControl処理経路を通る。特定キーとactionの対応表は要求しない。 | 現行 |
| <a id="rq-10-008"></a>RQ-10-008 | enabled / disabled | enabled / disabled状態を公開し、disabled Controlへの操作要求が実行されないことを反映する。 | 現行 |
| <a id="rq-10-009"></a>RQ-10-009 | OS公開 | PlatformはCoreのrole、name、state、value、focus、supported actionの意味を、対象OSのAccessibility機構へ意味を保って写像する。個別OS API型・属性名は本要求では固定しない。 | 現行 |
| <a id="rq-10-010"></a>RQ-10-010 | ARIA APG | W3C ARIA Authoring Practices Guideは参考資料として利用する。Web固有の慣習をdesktop・組み込みLinuxへ適用する範囲は本プロジェクトで判断し、完全なARIA適合を主張しない。 | 現行 |
| <a id="rq-10-011"></a>RQ-10-011 | Accessibility identityと操作境界 | 同じControlの生存中はAccessibility上の対象を安定して識別でき、終了済み対象へのquery/actionを新対象へ誤適用しないこと。参照するだけでkeyboard focusを変更せず、操作は対応する通常Control処理と同じ結果となること。 | 初期共通基盤 |
| <a id="rq-10-012"></a>RQ-10-012 | Accessibility nameの空欄 | Accessibility nameは表示文字列から独立し、未設定または空欄の場合は表示値等から自動合成しないこと。 | 初期共通基盤 |
| <a id="rq-10-013"></a>RQ-10-013 | Accessibility論理階層 | アプリが構成するLogical階層に従って初期Accessibilityの親子を公開し、template内のVisual部品は既定で独立対象として公開しないこと。 | 初期共通基盤 |
| <a id="rq-10-014"></a>RQ-10-014 | OS同期actionの待機境界 | OS同期編集actionの待機制限を設定・識別でき、初期既定を2秒とすること。未開始の取消と開始済み結果不明を区別し、timeoutをrollback成功として表示しないこと。 | 初期共通基盤 |


## Control別の意味情報

| Control | role | name | state / enabled | value | focus | supported action |
|---|---|---|---|---|---|---|
| Window | window | 設定された名前（任意） | 表示・active Window状態 | なし | active focus対象を含む | Windowの有効な操作 |
| StackPanel | group/container | 設定された名前（任意） | enabled状態 | なし | focus対象ではない | なし |
| Text | static text | 設定された名前（任意） | enabled状態 | 現在表示する文字列 | focus対象ではない | なし |
| TextBox | editable text | 設定された名前 | enabled、focus、validation状態 | 編集中の文字列 | Coreのactive focusと一致 | 入力・選択・caretに関する対応操作 |
| Button | button | 設定された名前 | enabled、focus状態 | なし | Coreのactive focusと一致 | 通常の起動。Command経由で実行可否に従う |

## 適用境界

承認は要求本文の範囲で完結する。API名・内部データ構造・処理方式の確定を前提にしない。既存ADRによるRust Core・versioned C ABI等の明示的な構成制約は維持する。

## 承認履歴

| 日付 | 対象版・ID / 範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-09-26 | 旧要求仕様の該当範囲 | 利用者 | Acceptedを継承。後日追記の承認日・未決範囲は旧版で確認 |
| 2026-10-12 | 要求承認管理・章分割・RQ ID化 | 利用者 | 今回の指示による管理方針。旧本文に記載済みの承認事項を要求側で管理する |
| 2026-10-12 | RQ-10全掲載ID / 再編集版0.3 | 利用者 | 要求詳細と受入確認を承認 |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 旧版 | 2026-09-25 | 旧要求仕様を作成。2026-09-26にAccepted | 旧文書参照 | 旧要求仕様の承認記録 |
| 0.1 | 2026-10-12 | 章分割、RQ ID、受入確認案・設計/testリンク、要求承認履歴の集約 | Codex | 管理方針の指示に基づく再編集。版全体のレビューは未完了 |
| 0.2 | 2026-10-12 | 下流参照リンクを除去し、参照方向を明記 | Codex | 要求本文・承認状態は変更なし |
| 0.3 | 2026-10-12 | Controlごとの意味情報と各要求IDの具体条件・受入確認を整理 | Codex | 再編集版を利用者が同日承認 |

## 今回の承認履歴

| 日付 | 対象版・範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.4 / 全レビュー案の要求補足・意味単位移管 | 利用者 | 全レビュー案の反映指示により承認。実装・試験実行の完了とは区別する |

## 今回の改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 |
| --- | --- | --- | --- |
| 0.4 | 2026-10-17 | 要求と実現手段・検証・作業記録を分離し、正式IDと参照を対応付けた | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.5 / 要求変更 RQ-10-014 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.5 | 2026-10-17 | 要求変更 RQ-10-014を正本へ適用し、旧判断との対応を保持 | Codex |
