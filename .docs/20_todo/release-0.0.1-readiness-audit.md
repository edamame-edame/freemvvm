# 0.0.1実装基線と初期UI縦断の着手条件確認

- 計画版: 0.1
- 状態: 完了（2026-10-24）
- 作成日: 2026-10-24
- 対象version branch: `version/0.0.1`
- 対象: 0.0.1の承認済み要求・設計・試験・実装unit状態を照合し、次の実装unitを特定する

## 背景

[0.0.1実装・リリース計画](release-0.0.1.md)は、release基線を固めた後にCore UI、Platform、wrapper、packageの順で進める構成だが、現文書の状態は「利用者レビュー待ち」のままである。計画中のunit集計は110件（complete 55、ready 55）だが、現在の`tools/units.yaml`は113件（complete 76、ready 37）である。Core UI Tree所有統合は完了した一方、Window/ControlのC ABI・wrapper接続を含む初期UI縦断は未実装である。

初期UIの要求は[RQ-09](../11_requirements/RQ-09_ui-scope.md)等に、Controlの詳細意味論は[UI Control詳細設計](../13_detail_design/ui/controls.md)に、予定試験は[UI単体試験設計](../32_unit_test/ui.md)および[release scenario](../30_release_test/system-scenarios.md)に記載される。ただし、設計・試験文書の承認状態と本文の案・未決表現に不整合がないか、C ABI・wrapperへ進む前に確認する必要がある。

## 目的

0.0.1の次の実装unitを、承認済みRQ・DD・testに根拠付けて小さく確定する。必要なAPI・動作判断が未承認なら、実装せず対応する設計改訂案を先に作り、文書単位のレビューへ送る。

## 対象と対象外

- 対象: 0.0.1計画の承認状態、unit台帳、初期UI要求、UI/Platform/ABI/wrapper設計、unit・release testの状態照合。
- 対象: Windows x64、Linux x64、組み込みLinux profileの計画上の境界と、先行するCore UI/API作業との依存関係の整理。
- 対象外: 製品コード・公開headerの変更、未承認API/Platform方式の確定、WebEngine branchの採否・統合、release tag作成、公開。

## 作業手順

1. RQ-09、RQ-10、RQ-11、RQ-12、RQ-14の承認状態と0.0.1現行対象を確認する。
2. UI Control、Tree/Layout、Platform abstraction/internal layers、ABI、wrapper DDの文書単位の承認履歴を確認し、本文に残る案・未決事項を別に記録する。
3. UI単体試験、release scenario、requirement-test matrix、version testの承認・実装・実行状態を照合する。
4. `tools/units.yaml`の現行件数と状態、完了済みTree unit、未実装Core/ABI/UI範囲、依存関係を抽出し、0.0.1計画の古い集計を更新案として整理する。
5. 次の最小実装unit候補を一つ提示し、対応RQ・承認済みDD・test case・同一revisionで実行するtest・親ブランチを特定する。設計差分が必要な場合は実装候補より先に設計レビュー対象を提示する。
6. 監査結果と次unit案を利用者レビューへ提示する。計画本文の改訂は承認後に反映・commitし、実装開始はそのunitの要求・設計・test状態が確認できた後とする。

## 完了条件

- 現行unit台帳の集計が再現可能な形で記録され、0.0.1計画の古い数値との差が説明されている。
- 初期UIの各候補に承認済みRQ/DD/testの根拠と未決差分が対応付けられている。
- 次のunit候補が、実装前に必要な設計承認・ABI/wrapper依存・試験方法を含めてレビュー可能である。
- WebEngine branchおよび公開・tag操作が本監査へ混入していない。

## 確認時点の観測

- `tools/units.yaml`: schema 2、113 units、complete 76、ready 37。
- `version/0.0.1`: Tree ownership統合済み。実装branchはこのversion branchからCore APIs childとして分岐する。
- `release-0.0.1.md`: 状態欄は利用者レビュー待ち。計画の承認履歴には承認記録がない。
- `ui/controls.md`: 文書冒頭は1.1全文承認済みだが、本文に「案」および後続設計へ送るAPI決定が残るため、文書全体の承認範囲を監査する。
- `32_unit_test/ui.md`: 承認履歴に0.2改訂案のレビュー待ちと後続の0.2全文承認が順に記録されている。状態の不整合ではない。
- この観測は実装・release gate・OS実行成功を意味しない。

## 監査結果

- RQ-09、RQ-10、RQ-11、RQ-12、RQ-14は各章の現行版が承認済み。初期UI・Platform・配布・開発順序は要求上定義されている。
- UI controls 1.1、tree-layout 1.2、input-focus 1.1、text-editing 1.1、Platform abstraction/internal-layers 1.1、共通C ABI 1.2は各文書の承認履歴で全文承認を確認した。
- UI単体test 0.2、Platform build test 0.1、release規約・system scenario、version test規約は承認済み。各承認はtest実装・実行成功の証拠ではない。
- 実装unit台帳はschema 2、113件（complete 76、ready 37）。内訳はCORE 43/26、ABI 12/7、WRAP 21/2、BUILD 0/1、PKG 0/1（complete/ready）。UI/Platform専用のunit prefixはまだない。UI Tree所有のUNIT-CORE-067～069以外に、Control/Layout/Focus/描画の実装unitは登録されていない。
- 共通C ABI 1.2はRuntime、Property、Binding、Validation、Command、TextBox値接続、dispatch等を定義するが、Window/Widget/StackPanel/Text/Buttonを生成・接続するC ABI symbol/API表はない。headerと`crates/c-abi`実装にも該当symbolはない。
- UI Control DDは共通意味論を承認済みとしている一方、公開ABI signature・完全なsymbol一覧をABI設計へ送っている。Platform DDも共通責務とpeer/frame接続を定義するが、各実装unitの境界・検査commandは未登録。
- `version/0.0.1`はTree ownership統合済み。次のCore/C ABI設計・実装はこのversion branchからCore APIs childとして開始し、WebEngine branchは含めない。

## 次工程候補

次に必要なのは製品コードではなく、初期UIのC ABI契約をDDとして具体化する作業である。最小範囲をWindow・共通Widget/Container・StackPanel・Textの生成、所有、Logical Tree接続とし、RQ-09-001～004/007/008/043/044に結び付ける。合わせてABI feature/table、失敗時出力、Core object/store接続、unit-testの観測条件を明示し、TextBox/Button/input/renderingを含む後続単位との境界を整理する。具体signatureや実装方式は次の計画・設計レビューで提案し、承認前には確定しない。

この候補は本監査の結論であり、実装開始承認ではない。次の作業計画は別文書で具体化し、利用者レビュー後にcommitする。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-24 | 0.1 / 0.0.1実装基線と初期UI縦断の着手条件確認 | 利用者承認・実施完了 | 計画は「ok」により承認。監査結果・次工程候補を記録。実装・release・公開を意味しない |
