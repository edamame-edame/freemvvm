# 0.0.1 TextBox Core編集状態・TextCommit既定動作 実装計画

- 計画版: 0.1
- 状態: 承認済み計画。UNIT-CORE-092実装・local検証完了、push後にUNIT-CORE-091を継続
- 対象version: `0.0.1`
- 作業branch: `impl-rust/core-input-focus`（Core/Rust/C ABI層）
- 親計画: [Input / Focus event path実装計画](input-focus-event-path-implementation.md)
- 承認済み実現仕様: [TextBox・Validation・IME編集](../13_detail_design/ui/text-editing.md)、[Input/Focus](../13_detail_design/ui/input-focus.md)、[Control](../13_detail_design/ui/controls.md)

## 目的

Coreに未実装のTextBox編集stateとcandidate transactionを追加する。TextBoxの既存Value Property、Binding、Property Validatorの意味を保ち、承認済みDD-14に従った編集candidate処理を用意する。このunitではInputRouterとの接続を含めず、既存CORE-091のTextCommit default actionは本unitの後に接続する。

## 現状と不足

- RuntimeにはTextBoxの`TextBoxValue` UTF-8 PropertyとFocus eligibilityがある。
- `TextBoxState`、不変`TextSnapshot`、editing text、caret/selection、View Input Validator、TextCommitからValue Propertyへ反映する経路は存在しない。
- 承認済み[UT-INPUT-004](../32_unit_test/ui.md#ut-input-004)はTextCommitがTextBox editing/Validatorへ進むことを期待する。既存`UNIT-CORE-091`はroute接続を担当し、独立`UNIT-CORE-092`で編集state/candidate transactionを先行実装する。
- DD-14はUTF-8 byte offset、編集candidateの原子的準備、View Input Validatorと確定Property Validatorの順、外部Value更新、FocusLost保持を規定する。入力API、TextBox state API、TextInputChanged通知の公開ABI/wrapper signatureは別設計範囲である。

## 実装範囲案

1. TextBoxごとにCore-owned editing stateを追加する。確定Value Propertyと編集中snapshotは分離し、snapshotはUTF-8全文、anchor/caret byte offset、feedback/reason、revisionを所有する。
2. 生成時は初期Valueを編集全文へcopyし、有効なUTF-8境界にcollapsed selectionを置く。初期caret位置は承認済みDDで定義されていないため、**末尾（ValueのUTF-8 byte長）を提案**する。
3. 編集stateはTextCommit payloadをselection置換candidateへ変換できる内部操作を持つ。このunitではInputRouterを呼ばず、route対象・handled確認は後続`UNIT-CORE-091`が担当する。
4. candidateは必要UTF-8 bytes・次snapshot・Property更新・通知予約を先に準備する。準備失敗時はValueとediting snapshotを変えない。commitはDD-14所定の一つのUI transactionとして行う。
5. View Input Validatorが未登録ならAcceptとして処理する。登録APIはこのCore sliceへ追加せず、後続C ABI/wrapper設計・実装unitで公開する。TextBox ValueにProperty Validatorが登録されている場合は既存Property validationを通し、その結果とDD-14の候補保持規則に従う。
6. 外部からValue Propertyが異なる値へ更新された場合は、DD-14どおり既存編集snapshot/sessionを失効させて編集全文を同期する。同値更新は編集中全文を上書きしない。

## 対象外

- OS IME composition session、preedit、IME candidate UI、platform event ordering。
- caret移動key mapping、selection操作API、undo/redo、TextBox state getter/notificationのpublic ABI。
- View Input Validatorの登録APIと各言語wrapper surface。
- Button pointer activation、C ABI Input borrowed event API、Windows native input mapping。

## 試験・unit案

- `.docs/32_unit_test/ui.md`へCORE state単体caseを追加し、UTF-8境界、Property Validator accept/reject、外部Value更新/同値更新、準備失敗時atomicityを検証する。InputRouterのTextCommit target/handled確認とBinding統合は既存UT-INPUT-004に留める。
- `UNIT-CORE-092`を独立planned登録し、`UNIT-CORE-091`から依存させる。前者は編集state/candidate transaction、後者はInput routeとの接続を担当する。
- 該当unitのRust tests、workspace Clippy、format、minimum Rust、文書リンク、`git diff --check`を同一source revisionで実施する。通過後、日本語commit・pushする。

## 作業段階

| 順 | 成果物 | 完了条件 |
| --- | --- | --- |
| 1 | 本計画、UI試験追補、unit inventory | 完了。承認済み判断を反映し、unit path/command/test/design根拠を登録 |
| 2 | Core TextBox editing stateとcandidate transaction | 完了。Core 200 tests、Clippy、format、Rust 1.85 checkを通過 |
| 3 | CORE-091 route接続 | 依存unitとして後続実装し、unhandled TextCommitだけactive TextBoxへ適用。route invalid/handled/targetなしでは状態を変えない |
| 4 | CORE-092 commit/push | 実行中。計画と`.docs/01_memory/`を含む同revisionを日本語commit・pushし、CI結果を確認 |

## 承認済み判断

- 初期caretをValue末尾に置く（承認済み）。
- View Input Validatorのpublic registration APIは後続C ABI/wrapper unitへ分け、今回Core側はvalidator未登録時Acceptとする（承認済み）。
- TextBox Coreは独立`UNIT-CORE-092`とし、入力routeの`UNIT-CORE-091`が本unitへ依存する（承認済みscopeの依存を明確化）。
