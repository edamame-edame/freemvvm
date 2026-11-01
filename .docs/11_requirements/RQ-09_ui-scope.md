# RQ-09: UI機能と段階的な対象範囲

- 文書版: 0.7
- 状態: 0.6全文承認済み（利用者、2026-10-30）。0.7全文承認済み（利用者、2026-10-31）。既存承認内容を維持
- 更新日: 2026-10-31
- 入口: [要求仕様索引](reqirements-specification.md)

## 概要

初期共通基盤、次段階、将来候補を区別する。製品機能として承認済みの範囲でも、個別Controlの未決API、既定操作、gesture、navigationはここで追加決定しない。

本章は提供機能・利用条件・外部から観測できる結果を定める。各IDの対象段階を区別し、将来候補の列挙を採用承認とはしない。

## 要求本文

| 要求ID | 項目 | 要求詳細 | 対象段階 |
| --- | --- | --- | --- |
| <a id="rq-09-001"></a>RQ-09-001 | Window | WindowはアプリのUIルートとなり、初期シナリオの内容を表示する一つのcontent rootを持つ。 | 現行 |
| <a id="rq-09-002"></a>RQ-09-002 | Widget | WidgetはUI要素の共通概念であり、初期Controlの共通状態と操作対象を表せる。 | 現行 |
| <a id="rq-09-003"></a>RQ-09-003 | Container | Containerは子Widgetを持ちうるWidgetの分類である。初期共通基盤ではWindowとStackPanelが該当する。 | 現行 |
| <a id="rq-09-004"></a>RQ-09-004 | Text | Textは読み取り専用の文字列表示Controlで、OneWay Bindingの表示先となる。 | 現行 |
| <a id="rq-09-005"></a>RQ-09-005 | TextBox | TextBoxは文字列入力Controlであり、入力文字列と選択・caret状態を保持できる。TwoWay Bindingとcommit時の検証を扱う。 | 現行 |
| <a id="rq-09-006"></a>RQ-09-006 | Button | ButtonはCommandを起動する。CanExecuteがfalseなら起動せず、同一Commandの実行中に再入させない。 | 現行 |
| <a id="rq-09-007"></a>RQ-09-007 | StackPanel | StackPanelは順序付きの子を配置し、配置方向と子の配置位置・伸長を設定できること。設定省略時の結果は本章の既定値要求に従うこと。 | 現行 |
| <a id="rq-09-008"></a>RQ-09-008 | 基本layout | 利用可能な表示領域と各Widgetの配置条件に従って表示位置・大きさを決定でき、親領域や配置条件の変化後も表示・入力対象の位置が一致すること。 | 現行 |
| <a id="rq-09-009"></a>RQ-09-009 | focus | active Windowと、0または1個のactive focus対象を識別できること。 | 現行 |
| <a id="rq-09-010"></a>RQ-09-010 | keyboard input | 対象Platformの入力を共通UI機能で利用でき、keyboard・文字入力が現在のactive focus対象へ届くこと。 | 現行 |
| <a id="rq-09-011"></a>RQ-09-011 | accessibility name | 初期ControlのAccessibility nameを設定・取得でき、対応するUI要素の意味情報として利用できること。空欄の扱いは本章およびRQ-10のname要求に従うこと。 | 現行 |
| <a id="rq-09-012"></a>RQ-09-012 | CheckBox | CheckBoxを次段階の対象に含める。 | 次段階 |
| <a id="rq-09-013"></a>RQ-09-013 | ToggleButton | ToggleButtonを次段階の対象に含める。 | 次段階 |
| <a id="rq-09-014"></a>RQ-09-014 | ScrollViewer | ScrollViewerを次段階の対象に含める。 | 次段階 |
| <a id="rq-09-015"></a>RQ-09-015 | Image | Imageを次段階の対象に含める。 | 次段階 |
| <a id="rq-09-016"></a>RQ-09-016 | Border | Borderを次段階の対象に含める。 | 次段階 |
| <a id="rq-09-017"></a>RQ-09-017 | Separator | Separatorを次段階の対象に含める。 | 次段階 |
| <a id="rq-09-018"></a>RQ-09-018 | ProgressBar | ProgressBarを次段階の対象に含める。 | 次段階 |
| <a id="rq-09-019"></a>RQ-09-019 | CollectionModel | 次段階で、UI要素の所属や単一値の種類と独立した項目集合を扱え、順序から独立した項目識別、構造化された変更内容、可視範囲の一貫した内容を利用できること。集合の一括更新は全件成功または全件拒否とし、独立した入力の値確定・業務検証を別入力待ちにしないこと。 | 次段階 |
| <a id="rq-09-020"></a>RQ-09-020 | SelectionModel | 次段階で、項目の安定した識別に基づく単一・複数選択を利用できること。現在項目は選択状態およびkeyboard focusから独立し、削除項目を選択中・現在項目として残さないこと。 | 次段階 |
| <a id="rq-09-021"></a>RQ-09-021 | ListView | ListViewを次段階の対象に含め、共通CollectionModel・SelectionModelの契約を利用する。 | 次段階 |
| <a id="rq-09-022"></a>RQ-09-022 | virtualized list | 仮想化Listを次段階の対象に含め、表示範囲とCollectionModelのvisible range snapshotを整合させる。 | 次段階 |
| <a id="rq-09-023"></a>RQ-09-023 | Style | Styleを次段階の対象に含め、Controlの表示規則を再利用可能にする。 | 次段階 |
| <a id="rq-09-024"></a>RQ-09-024 | Theme | Themeを次段階の対象に含め、複数Controlで共有する外観設定を切替可能にする。 | 次段階 |
| <a id="rq-09-025"></a>RQ-09-025 | Resource | Resourceを次段階の対象に含め、UI構成から再利用可能な値を参照できるようにする。 | 次段階 |
| <a id="rq-09-026"></a>RQ-09-026 | Template | Templateを次段階の対象に含め、Controlの内部表示構成を差し替え可能にする。内部visual部品をLogical Tree上の独立Controlとして扱うかは既承認のTree境界に従う。 | 次段階 |
| <a id="rq-09-027"></a>RQ-09-027 | ComboBox | ComboBoxは将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-028"></a>RQ-09-028 | TreeView | TreeViewは将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-029"></a>RQ-09-029 | TableView | TableViewは将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-030"></a>RQ-09-030 | Dialog | Dialogは将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-031"></a>RQ-09-031 | Popup | Popupは将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-032"></a>RQ-09-032 | Menu | Menuは将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-033"></a>RQ-09-033 | ToolBar | ToolBarは将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-034"></a>RQ-09-034 | TabView | TabViewは将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-035"></a>RQ-09-035 | binding診断 | binding診断は将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-036"></a>RQ-09-036 | UIテスト支援 | UIテスト支援は将来候補として保持し、現行releaseの必須機能には含めない。 | 将来候補・未採用 |
| <a id="rq-09-037"></a>RQ-09-037 | 共通Modelを個別Controlより先に設計 | List、ComboBox、Tree、Tableに共通するModel、Selection、Item template、Collection差分通知、virtualizationの契約を、個別Controlより先に設計する。Collection change batchの全件適用／全件拒否はModel整合性を対象とし、独立入力欄の有効なcommitを別入力のvalidation待ちにしない。Gridを初期Controlへ追加する判断は本要求に含めない。Controlごとのmode、gesture、navigationは、各Controlの採用範囲を承認した後に具体化する。 | 実施・管理 |
| <a id="rq-09-038"></a>RQ-09-038 | TextBox入力三値の結果 | Acceptは候補を確定して伝播し、Incompleteは編集文字列と選択位置に保持して確定値へ伝播しないこと。通常Rejectは従前の編集値・選択を維持し理由を示すこと。IME commitのRejectは候補を未受理編集値として保持すること。 | 初期共通基盤 |
| <a id="rq-09-039"></a>RQ-09-039 | TextBox自身のProperty検証拒否 | 入力判定でAcceptされた候補をTextBoxの確定値検証が拒否した場合、確定値とViewModel値を維持し、拒否された候補と理由を利用者が確認・修正できること。 | 初期共通基盤 |
| <a id="rq-09-040"></a>RQ-09-040 | IME cancel後の入力理由 | IMEの明示cancelではcomposition開始直前の編集文字列・caret・selectionと入力状態・理由へ戻り、確定値検証とBinding伝播を起こさないこと。 | 初期共通基盤 |
| <a id="rq-09-041"></a>RQ-09-041 | IME commit準備失敗 | IME確定候補の処理準備が失敗した場合、候補を回復可能に保持できたかと再試行方法を利用者が判断できること。保持できなかった候補を処理成功として扱わないこと。 | 初期共通基盤 |
| <a id="rq-09-042"></a>RQ-09-042 | 外部更新と編集中の値 | 外部から異なる確定値が設定された場合は編集値をその値へ更新し、入力状態・理由を初期化すること。旧IME sessionの遅延イベントでその値を再び上書きしないこと。 | 初期共通基盤 |
| <a id="rq-09-043"></a>RQ-09-043 | Detached subtreeの利用 | 親から切り離しても独立した所有参照を持つ対象とその子構造をCore上で利用できること。切離しだけで通常のProperty・Binding・Subscriptionを終了させず、描画・入力対象からは外すこと。 | 初期共通基盤 |
| <a id="rq-09-044"></a>RQ-09-044 | Tree変更の拒否結果 | 複数親、親子循環、異なるRuntime所属を作る親子変更を拒否すること。変更失敗で既存の親子・値・登録を部分変更せず、Detached対象の再所属失敗時はDetached状態を維持すること。 | 初期共通基盤 |
| <a id="rq-09-045"></a>RQ-09-045 | StackPanel公開設定と既定 | StackPanelの配置方向を縦・横から選び、子の配置位置と伸長を設定できること。 | 初期共通基盤 |
| <a id="rq-09-046"></a>RQ-09-046 | 有効性とFocusの共通結果 | Control自身またはLogical祖先が無効ならInput・Focus・Button操作を受け付けず、子自身の設定値を変更しないこと。Focus対象を失った場合は無効な対象を残さず、別Controlを暗黙に選ばないこと。 | 初期共通基盤 |
| <a id="rq-09-047"></a>RQ-09-047 | keybindingの初期範囲 | 特定キーによるFocus移動・shortcut/action bindingは初期必須機能に含めないこと。 | 初期共通基盤 |
| <a id="rq-09-048"></a>RQ-09-048 | Command未接続Button | Commandが設定されていないButtonの表示を許し、起動操作で処理を実行しないこと。 | 初期共通基盤 |
| <a id="rq-09-049"></a>RQ-09-049 | Focus通知と入力route | Focus変更後の状態を観測でき、旧対象のFocusLostを新対象のFocusGainedより先に扱うこと。入力対象がないkeyboard/textイベントを後の別対象へ遅延配送しないこと。 | 初期共通基盤 |
| <a id="rq-09-050"></a>RQ-09-050 | Collection stable identity | 次段階Collectionは順序位置から独立した項目IDを保持し、削除済みIDを別項目へ再利用しないこと。Controlが編集対象を識別する際も位置変化で別項目へ取り違えないこと。 | 次段階 |
| <a id="rq-09-051"></a>RQ-09-051 | Selection順序と拒否 | 次段階Selectionの公開順は現在のCollection順に従い、並べ替えで選択ID集合を変えないこと。Singleを破る複数選択の更新は全体を拒否し、項目削除でselected/currentからその項目を除くこと。 | 次段階 |
| <a id="rq-09-052"></a>RQ-09-052 | Collection batchの観測結果 | 無効なchange batchはCollection内容・順序・revision・Selection・公開通知を変更しないこと。有効なbatchは一つの更新として公開し、中間の不整合を外部callbackから観測させないこと。 | 次段階 |
| <a id="rq-09-053"></a>RQ-09-053 | text coverageと編集位置 | 初期版でUTF-8のLatin・日本語UI文字列を表示・編集でき、対応profileの文字・font依存条件を判断できること。 | 初期共通基盤 |
| <a id="rq-09-054"></a>RQ-09-054 | 初期textの対象外 | 初期版の保証にChinese、Korean、RTL、emojiの表示・編集を含めないこと。対応を追加する場合は対象文字・profileを明示すること。 | 初期共通基盤 |
| <a id="rq-09-055"></a>RQ-09-055 | Focus離脱時の編集保持 | Focus離脱では未確定文字列・caret・selectionを保持し、確定や明示取消として扱わないこと。OS候補一覧・学習状態・変換内部状態の復元は保証しないこと。 | 初期共通基盤 |
| <a id="rq-09-056"></a>RQ-09-056 | IME終了イベントの順序 | commitとFocusLostが競合した場合はPlatformでの受信順に従い、終了済み入力sessionの遅延イベントを適用しないこと。Undo/Redoの追加保証は含めないこと。 | 初期共通基盤 |
| <a id="rq-09-057"></a>RQ-09-057 | 初期配置の省略値 | StackPanelの省略時方向はVertical、Alignの省略値はStretchとすること。 | 初期共通基盤 |
| <a id="rq-09-058"></a>RQ-09-058 | 初期有効状態 | 初期Control自身のenabled設定の省略値はtrueとすること。 | 初期共通基盤 |
| <a id="rq-09-059"></a>RQ-09-059 | 初期focusable値 | 初期focusable値はTextBoxとButtonがtrue、Window・StackPanel・Textがfalseとすること。Window/StackPanelへの明示focusable=trueはRQ-09-070に従い拒否すること。 | 初期共通基盤 |
| <a id="rq-09-060"></a>RQ-09-060 | 初期visibility範囲 | 初期共通ControlにVisible/Hidden/Collapsedの切替機能を必須提供しないこと。 | 初期共通基盤 |
| <a id="rq-09-061"></a>RQ-09-061 | 直接Click範囲 | 初期Buttonの必須起動機能はCommandとし、直接Click callbackの提供を必須としないこと。 | 初期共通基盤 |
| <a id="rq-09-062"></a>RQ-09-062 | Collectionの型とsnapshot | 次段階Collectionの列に既存の五値種別を指定できること。可視範囲と安定ID・revisionを持つ一貫したsnapshotを保持し、元Modelの後続変更に影響されず読めること。 | 次段階 |
| <a id="rq-09-063"></a>RQ-09-063 | Collection通知と再入 | 同じ一括更新の集合変更通知を選択変更通知より先に観測し、両方から同じ確定revisionを読めること。通知中の再入一括更新は拒否し、後続処理へ依頼できること。 | 次段階 |
| <a id="rq-09-064"></a>RQ-09-064 | Selection縮退・差替え | 複数選択を保ったままSingleへ縮退する変更は拒否すること。削除後currentを自動補充せず、対象集合を変更する場合は別の選択状態として明示できること。 | 次段階 |
| <a id="rq-09-065"></a>RQ-09-065 | TextBox編集と位置の操作 | 文字列範囲の置換、selection/caretの明示変更、確定値へのresetを利用でき、無効な文字境界を指定する更新を拒否すること。文字単位navigationを採用する場合はUnicode16.0のgrapheme境界に従うこと。 | 初期共通基盤 |
| <a id="rq-09-066"></a>RQ-09-066 | TextBox通知の観測順 | 同じ編集確定のBinding伝播とProperty通知の後に、本文・selection・composition・入力状態/理由が一致した編集snapshotを通知すること。Preeditでは確定値通知を発生させないこと。 | 初期共通基盤 |
| <a id="rq-09-067"></a>RQ-09-067 | 座標・数値範囲・clip | 論理寸法とdevice scaleを区別でき、無効な数値や表現不能な配置結果を拒否して以前の公開配置を保つこと。初期Controlの描画・hit-testは祖先のclient範囲で切り取ること。 | 初期共通基盤 |
| <a id="rq-09-068"></a>RQ-09-068 | Input配送とpointer範囲 | 入力は対象からLogical祖先へ扱え、handled指定で以後の配送と既定動作を止められること。pointer move/down/up/wheelと単一pointer captureを追加候補として区別すること。 | 初期共通基盤 |
| <a id="rq-09-069"></a>RQ-09-069 | 再所属失敗と再試行 | 切離し対象の表示資源再作成に失敗してもDetached構造・確定値・登録を保持し、明示再試行できること。所属だけを成功として公開しないこと。 | 初期共通基盤 |
| <a id="rq-09-070"></a>RQ-09-070 | Containerのfocusable範囲 | 初期版ではWindow/StackPanelの明示focusable=trueへの変更を受け付けないこと。Windowのactive状態は子のkeyboard focusと区別すること。 | 初期共通基盤 |
| <a id="rq-09-071"></a>RQ-09-071 | Window開閉とclose判断 | Windowの表示開始・終了を明示でき、終了後もCore状態を保持して再表示できること。close要求をAccept/Decline/Deferで判断でき、既定Accept、未解決判断の破棄はDeclineとすること。Runtime終了は個別vetoの対象にしないこと。 | 初期共通基盤 |
| <a id="rq-09-072"></a>RQ-09-072 | 初期text表示の範囲 | 初期Textは単一paragraphの表示、client clipを提供し、soft-wrap/ellipsis/選択copyの追加を必須に含めないこと。Buttonの表示labelとAccessibility nameは独立すること。 | 初期共通基盤 |
| <a id="rq-09-073"></a>RQ-09-073 | 次段階Style/Theme/Resource/Template | 次段階でtyped表示設定の再利用、共有外観切替、local→Logical祖先→Runtime themeのResource参照、Logical identityを保つ内部表示差替えを提供すること。未定義/型不一致/cycleを明示失敗とし、任意script実行を必須に含めないこと。 | 次段階 |
| <a id="rq-09-074"></a>RQ-09-074（改訂案） | Window背景色 | 各Windowの背景色を利用者が指定・取得でき、未指定時は不透明な白（RGB #FFFFFF）を使用すること。指定解除後は規定値に戻ること。色はnormalized RGB各成分0～1で指定し、非有限値・範囲外値は拒否して既存色を保つ。背景色はwindow client領域へ適用され、Layout寸法・子Controlの表示色・Button label・Accessibility nameを変更しないこと。 | 初期共通基盤 |
| <a id="rq-09-075"></a>RQ-09-075 | 未指定fontの既定表示 | fontが明示指定されていない初期Text系Controlは、実行中のPlatform profileが提供するsystem default fontを用いて表示すること。Platform profile間でfamily・字形・pixel outputが異なることを許容し、同一font appearanceを要求しない。 | 初期共通基盤 |


## 適用境界

承認は要求本文の範囲で完結する。API名・内部データ構造・処理方式の確定を前提にしない。既存ADRによるRust Core・versioned C ABI等の明示的な構成制約は維持する。

## 承認履歴

| 日付 | 対象版・ID / 範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-09-26 | 旧要求仕様の該当範囲 | 利用者 | Acceptedを継承。後日追記の承認日・未決範囲は旧版で確認 |
| 2026-10-12 | 要求承認管理・章分割・RQ ID化 | 利用者 | 今回の指示による管理方針。旧本文に記載済みの承認事項を要求側で管理する |
| 2026-10-12 | RQ-09全掲載ID / 再編集版0.3 | 利用者 | 要求詳細と受入確認を承認。本文記載の将来候補は未採用のまま維持 |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 旧版 | 2026-09-25 | 旧要求仕様を作成。2026-09-26にAccepted | 旧文書参照 | 旧要求仕様の承認記録 |
| 0.1 | 2026-10-12 | 章分割、RQ ID、受入確認案・設計/testリンク、要求承認履歴の集約 | Codex | 管理方針の指示に基づく再編集。版全体のレビューは未完了 |
| 0.2 | 2026-10-12 | 下流参照リンクを除去し、参照方向を明記 | Codex | 要求本文・承認状態は変更なし |
| 0.3 | 2026-10-12 | 初期機能の条件と観測結果を明記し、次段階・候補・管理要求を表に整理 | Codex | 再編集版を利用者が同日承認。候補の採用は別途承認 |

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
| 2026-10-17 | 0.5 / 要求変更 RQ-09-062, RQ-09-063, RQ-09-064, RQ-09-065, RQ-09-066, RQ-09-067, RQ-09-068, RQ-09-069, RQ-09-070, RQ-09-071, RQ-09-072, RQ-09-073 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |
| 2026-10-30 | 0.6改訂案 / RQ-09-074 | — | Window単位の背景色指定・取得・規定値復帰とopaque white defaultを提案。全文レビュー待ち。0.5承認範囲を維持 |
| 2026-10-30 | 0.6全文 / RQ-09-074 | 利用者 | 「ok」によりWindow背景色、未指定時のopaque white、normalized RGB範囲、invalid値拒否と既存値保持を含む全文を承認。設計実装・test合格は別管理 |
| 2026-10-31 | 0.7改訂案 / RQ-09-075 | — | 利用者指示「指定無しならシステムのデフォルトフォント駆動で動かしたい」に基づきsystem default使用とplatform間の見た目差を提案。全文レビュー待ち。0.6承認baselineを維持 |
| 2026-10-31 | 0.7全文 / RQ-09-075 | 利用者 | 「ok」により未指定時Platform profile system default使用とprofile間appearance差を含む全文を承認。Platform固有resolution方式は設計へ委譲し、実装・test合格は別管理 |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.5 | 2026-10-17 | 要求変更 RQ-09-062, RQ-09-063, RQ-09-064, RQ-09-065, RQ-09-066, RQ-09-067, RQ-09-068, RQ-09-069, RQ-09-070, RQ-09-071, RQ-09-072, RQ-09-073を正本へ適用し、旧判断との対応を保持 | Codex |
| 0.6 | 2026-10-30 | Window背景色を未指定時opaque white・Window単位で指定可能とするRQ-09-074を追加。全文承認済み、既存73要求の承認状態を維持 | Codex |
| 0.7 | 2026-10-31 | 未指定fontはPlatform profile system defaultを使い、profile間のfont appearance差を許容するRQ-09-075を追加。利用者全文承認済み | Codex |
