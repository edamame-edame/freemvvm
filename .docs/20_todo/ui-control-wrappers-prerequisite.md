# UI Control wrapper基盤の追加計画案（旧案）

- 計画版: 0.1
- 状態: branch監査後に撤回。新案は[Input wrapperの既存Control branch統合計画](input-wrapper-control-branch-integration.md)
- 対象version: `0.0.1`
- 関連計画: [Input / Focus event path実装計画](input-focus-event-path-implementation.md)

## 当初の発見と監査結果

Input / Focus作業branchには`Window`、`Button`、`TextBox`、`StackPanel`等の公開Widget型がなかったため、初案はControl wrapper基盤を新規設計・実装する内容とした。

branch全体を監査するとControl wrapperは既存branchで実装済みであり、該当DD・unitも承認/completeだった。初案は誤ってworktreeの現在checkoutだけを見ており、完了済みbranchを考慮していなかったため撤回する。既存branchの統合順を扱う新案を作成した。

## 旧案の目的

既存UI Control wrapperを再設計・再実装しない。Input wrapper branchへ前提となる既存成果を統合する。

## 旧案の作業手順案（撤回）

| 順 | 作業 | 成果・確認 |
| --- | --- | --- |
| 1 | [Control詳細設計](../13_detail_design/ui/controls.md)、[ABI詳細設計](../13_detail_design/runtime-objects/abi.md)、該当RQ、4 wrapper DDを照合 | 既存C ABIで実現できる最小のWindow・Control生成/所有/論理ツリー操作を一覧化。Control種別と未決APIを分ける |
| 2 | 4言語wrapper DDへ公開Control型/APIの提案を作成 | 生成・所有・close/drop・UI thread・Runtime affinity・論理ツリー接続を各言語の慣習へ写す。共通Widget base classを増やすか否かは承認済みInput DDと矛盾させない |
| 3 | fake ABI試験と別consumer試験を設計 | Widget生成、異種Runtime拒否、parent/child接続、明示close/drop、失敗時ownershipを検証。OS実入力や描画結果を前提にしない |
| 4 | `.docs/32_unit_test/implementation-test-inventory.md`と`tools/units.yaml`へ依存unitを登録 | 各言語ごとに実行可能なsource/test pathとexact commandを設定し、着手時に`ready`へ更新 |
| 5 | 本計画とDD・test designを文書単位でレビュー・承認 | 承認範囲を確定してからwrapper実装branchを切る |
| 6 | wrapper Control foundationを言語別に実装・検証 | 各言語branchはCore APIsから独立に切り、各unitの同revision試験後に日本語commit/push。関連するControl foundationをまとめてからInput wrapper unitへ進む |
| 7 | Input / Focus計画へ依存関係を反映 | 各言語のInput wrapper unitは同言語Control wrapper foundationのcommitを親にする。consumerは具体Widget生成から購読・入力試験までを確認 |

## 旧案のスコープ候補と除外（撤回）

最小のControl種別・操作は既存の`.docs/13_detail_design/ui/controls.md`とCore/C ABIが提供する範囲から抽出し、設計提案時に明記する。現時点で具体的なRust/C++/C#/Python method名や型名は決定しない。

Pointer capture/move/wheel、Tab traversal、shortcut、追加描画機能、OS実機IME・keyboard/mouse操作は本基盤へ含めない。既存要求で保証されていない機能が必要と判明した場合は、要求改訂案として別に扱う。

## 既存branchとの関係

Control wrapperは次の既存branchにあり、各unitはそのbranchの`tools/units.yaml`でcomplete。Input APIを実装する前に、対応するlanguage wrapper baselineをCore Input child branchへ統合する。

| 言語 | 既存wrapper branch | 完了unit |
| --- | --- | --- |
| Rust | `impl-rust/ui-controls-wrapper` | `UNIT-WRAP-RS-UI-CTRL-001` |
| C++ | `impl-cpp/ui-controls-wrapper` | `UNIT-WRAP-CPP-009` |
| C# | `impl-csharp/ui-controls-wrapper` | `UNIT-WRAP-CS-CONTROLS-001` |
| Python | `impl-python/runtime-wrapper` | `UNIT-WRAP-PY-CONTROLS-001` |

4 branchとも`impl-rust/core-input-focus`とのmerge baseはCore UI Control C ABI commit `dfb86cf`。現在のRust Input wrapper branchは最新Core Input commitから作成済みで、まだ実装変更なし。詳細な統合順は新案でレビューする。

## レビュー対象

この旧案の新規Control API設計・実装は行わない。既存branch統合を対象とする新案へ移行する。
