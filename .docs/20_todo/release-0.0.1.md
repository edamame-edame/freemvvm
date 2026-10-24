# 0.0.1 利用可能なUIライブラリを目指す実装・リリース計画

- 計画版: 0.1
- 状態: 利用者レビュー待ち。実装・計画commit・タグ・公開は未承認
- 作成日: 2026-10-24
- 対象: 0.0.0を基点に、利用者アプリから実際にUIを構築・表示・操作できる0.0.1候補を完成させる
- 要求の正本: [要求仕様索引](../11_requirements/reqirements-specification.md)と各RQ章
- 試験の正本: [release test](../30_release_test/release-test.md)、[version test](../31_version_test/version-test.md)、[実装unit台帳](../32_unit_test/implementation-test-inventory.md)

## 目的

0.0.0はRuntime、Property、Binding、Async Command等の実装基盤を含む内部基点であり、UIライブラリとしての公開候補ではない。0.0.1では、インストールした成果物を使うアプリがWindowを開き、基本Controlを配置し、表示と入力、Binding、Commandを一続きで利用できる状態を目指す。

公開・配布は、要求に対応する実装と試験が完了し、release gateを満たした後に別途判断する。この計画の承認だけでは、製品仕様の追加承認、release合格、公開許可を意味しない。

## 現状と計画上の境界

- 0.0.0 tagのmainにはC++、C#、Rust wrapperのRuntime/Property/Binding/Async Command基盤がある。Pythonは現在のREADMEと実装状況に従い、未完成の骨格として扱う。
- [実装unit台帳](../32_unit_test/implementation-test-inventory.md)が参照する`tools/units.yaml`では、現時点で110 unit中55 unitがcomplete、55 unitがready。UI表示・入力を完成させた証拠とは区別する。
- UIの現行要求は[UI機能と対象範囲](../11_requirements/RQ-09_ui-scope.md)、Platformは[Platformと組み込みLinux](../11_requirements/RQ-11_platform.md)、配布は[ビルドと配布](../11_requirements/RQ-12_build-distribution.md)、順序は[開発・検証の実施順序](../11_requirements/RQ-14_delivery-sequence.md)に従う。
- 既存要求や承認済み設計にない外部保証・Control動作・Platform選択を、この計画だけで追加しない。差異が見つかった場合は、要求・設計・試験への影響と変更案を分けてレビューする。
- 別作業のRust inner WebEngine crate branchは自動的に統合しない。適用要求、既存Platform設計との適合、独立した試験結果を確認してから、0.0.1への採否を別途レビューする。

## 完了条件

以下をすべて満たした候補だけを0.0.1 release reviewへ進める。

1. 承認済みRQにあるWindow、Widget/Container、Text、TextBox、Button、StackPanel、基本layout、Focus、keyboard/IME、accessibilityの初期共通範囲が実装され、仕様で定める結果を利用者側から確認できる。
2. BindingとCommandを含むMVVMの縦断例が、C++、C#、Rust、Pythonの各wrapperから同じ共通Coreへ到達する。未対応の言語・機能を利用可能と表示しない。
3. 開発tree外のconsumerが配布形態からartifactを導入し、ネットワーク遮断時にも明示した手順で実行できる。artifactのtarget、ABI、profile、identity、digestの記録を残す。
4. Windows x64、Linux x64の初期targetと、要求に定める組み込みLinux ARM target/profileについて、cross-buildと代表機器上のRuntime/UI確認を別々に記録する。実機未確認を合格扱いにしない。
5. [共通UI縦断・profile・配布物](../30_release_test/system-scenarios.md)の対象scenario、要求test、unit test、version回帰を候補の同一commitで実施し、report・対象SHA・profileを記録する。
6. 文書link、`tools/dev.py check`、`tools/dev.py minimum`、対象platform build、全unit/consumer testが成功し、対象要求とtestの未対応・孤立がない。
7. README、利用者向け導入手順、対応target/profile、未対応範囲、既知制約が実成果物と一致する。

## 作業段階

### 要求・実装unitのrelease基線を固める

- RQ-09、RQ-10、RQ-11、RQ-12、RQ-14とrelease testの現行承認状態を再確認する。
- 0.0.0 tagを回帰baselineとし、現行unit台帳から0.0.1の依存順・未完了項目・対象profileを抽出する。
- 各実装unitについて、対応RQ、BD/DD/SD根拠、test case、実行コマンド、完了証拠を登録する。設計独自IDや設計項目別の承認台帳は追加しない。
- UI backend候補とWebEngine branchを照合する。採用には要求適合と必要な設計レビューを求め、未採用の場合も0.0.1で実現する既承認方式を明確にする。

### Core UIと基本Controlの縦断実装

- Rust CoreにWindow/Widget/Containerのtree、Text、TextBox、Button、StackPanelと基本layoutを、承認済みRQ/DDに従って実装する。
- ViewModelのProperty/Binding/CommandをUI treeへ接続し、親領域・配置条件の更新後も表示と入力対象が整合する。
- 実装unitごとにCore単体試験を同じ変更revisionで追加し、境界・拒否・close/shutdownを検証する。

### Platform、描画、入力、Accessibility

- Platform境界、Window/surface、OS event loop、描画、keyboard/text/IME、Accessibility接続を、承認済みPlatform設計と要求に従って実装する。
- 初期targetと組み込みLinux profileを分けてbuild・実行する。cross-build成功、OS上のUI実行、代表boardでの実機確認を別証拠として記録する。
- TextBox編集、Focus移動、IME commit/cancel、遅延event、Accessibility name/role/actionを、release scenarioと対応付けて検証する。
- 新しいRenderer、text engine、OS API、WebEngineを採用する必要が生じた場合は、実装前に該当設計文書全体のレビューを行う。

### 4言語wrapperとconsumer例

- C++、C#、Rust、Pythonの各wrapperから同じUI縦断機能を使えるようにする。言語間で型・所有・例外境界の意味を揃える。
- 各言語に、Window、Text/TextBox、Binding、Button/Commandを含む最小consumer例を用意する。
- 独立consumer試験でABI接続・native libraryの発見・終了・callback lifetimeを確認する。
- 各言語wrapperは最新Core APIs branchから個別に切り、他言語wrapper branchを親にしない。

### package、target artifact、導入検証

- 要求済みのCMake/C++、NuGet/C#、Cargo/Rust、Python配布経路にUI利用に必要なartifactを接続する。
- source tree外のconsumer、relocation、offline導入、ABI/profile不一致拒否を検証する。
- Windows/Linux初期targetと組み込みLinux ARM profileのmanifest、artifact identity/digest、license/noticeを揃える。
- 公開先へのupload、release作成、鍵や署名materialの発行・保管は、この実装計画では実行しない。公開前reviewと明示的なrelease判断を別に行う。

### release qualificationと0.0.1判断

- 変更unitと依存逆閉包に応じて[回帰計画](../31_version_test/regression-plan.md)の選択範囲を実行する。
- [共通UI縦断・profile・配布物](../30_release_test/system-scenarios.md)のシナリオを各対象profileで実施し、必須gateの未実行・失敗を残さない。
- requirement-test manifestへ要求・case・SHA・artifact/profile・reportを記録し、未実装・未実行と成功を区別する。
- READMEと導入文書を候補の実状態に合わせ、利用可能機能・target・制約を明示する。
- 全gate完了後にrelease reviewを提出する。利用者のrelease判断前に0.0.1 tag作成や公開を行わない。

## ブランチと変更単位

`.docs/02_git/rule.md`に従い、0.0.1作業では `main -> version/0.0.1 -> core APIs -> wrapper language / platform feature -> test` の依存順を使う。Core/C ABI変更の後に、そのCore branchから必要な言語wrapper branchを個別に切る。並行作業の既存WebEngine branchは自動mergeせず、採否判断が終わるまで別作業として扱う。

大きな0.0.1作業はversion branchで束ね、各実装・試験は単独で確認可能なunitへ分ける。各unitの実装とtestを同一revisionで完了・commitする。計画文書は利用者レビュー後にcommitし、実装開始は計画承認後とする。

## 段階ごとのreview gate

- 本計画のレビュー: 0.0.1の到達点、対象profile、作業分割、release gateを利用者が確認する。
- 要求・設計差分のレビュー: 実装前にRQ/BD/DD/SDの承認状態を照合し、追加判断が必要な変更だけを別文書で提案する。
- 実装unitの開始: 対応する要求と設計が承認済みであることを確認する。未承認方式は確定仕様として実装しない。
- release review: 実機・consumer・package・回帰の証拠を同一候補SHAに対応付け、公開可否を判断する。

## 計画承認・改訂履歴

| 日付 | 計画版・範囲 | 状態 | 根拠 |
| --- | --- | --- | --- |
| 2026-10-24 | 0.1 / 0.0.1利用可能UIライブラリ計画 | 利用者レビュー待ち | READMEと承認済みRQ/設計/release testを照合して初版作成。承認・実装開始・公開を意味しない |
