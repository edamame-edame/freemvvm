# 初期UI Control C ABI Core実装

- 計画版: 0.1
- 状態: 利用者承認済み（2026-10-25）。Core UI Control実装を開始
- 作成日: 2026-10-24
- 計画branch: `impl-rust/core-ui-controls-c-abi`
- 親branch: `version/0.0.1`
- 対象: 承認済みC ABI DD 1.3とUI単体試験設計0.3に基づくRust Core/C ABI実装

## 背景

[初期UI Control C ABI契約の具体化](abi-initial-ui-api.md)が完了し、[共通C ABI詳細設計](../13_detail_design/runtime-objects/abi.md) 1.3と[UI単体試験設計](../32_unit_test/ui.md) 0.3の全文が承認された。現行コードにはControl object store/APIとC ABI symbolがない。一方、Logical Treeのowner・attach/detach基盤はCoreに存在するため、これを再利用して段階的に実装する。

## 目的

Window、StackPanel、Text、TextBox、Buttonを共通C ABIから生成・構成できるよう、Core object state、Property/Command接続、Logical Tree操作、配置設定、Focus要求、Window peer/close-request境界を実装する。各実装unitと局所試験は同じcommit revisionに含め、完了ごとにcommitする。

## 対象範囲

- Core Runtime内の型付きUI object stateとControl生成・状態アクセス。
- 既存Logical Tree coordinatorを用いたWindow単一contentとContainer/StackPanel child操作。
- 共通Property・Binding API、typed Command、FocusManagerへの接続。
- C ABI header/export/API error mappingとfeature 8 (`core.ui.controls.v1`) の完了時advertisement。
- Window peer操作境界・close request lifecycle。OS固有backendや描画実装は含めない。
- `tools/units.yaml`の実unit登録と、承認済みUI test表に対応するfocused test/verification記録。

## 対象外

- C++/C#/Rust/Python wrapper変更。このCore APIs branchの後続child branchで個別に行う。
- 実OS window/render/input/accessibility backendの実装・統合。
- 要求、承認済みDD、試験期待結果の変更。新たな外部挙動が必要と判明した場合は実装を止め、差分を設計・要求レビューへ戻す。
- WebEngine branchの統合、version release/tag、公開。

## 実装前提と依存関係

- feature 8は全Control symbolと契約が実装済みになるまでadvertiseしない。途中unitで個別APIが存在しても、feature queryはfalseのままとする。
- 現行repositoryに製品用Platform Adapter実装はない。Window APIはCore/ABIからadapter境界へ委譲し、局所試験ではfake peerを注入する。adapter未接続時は共通`UNSUPPORTED`等の既定statusを使用し、成功したopen/closeを偽装しない。実backend対応は別unit/branchで行う。
- Window close requestはfake peerからのnative close intentを使ってcallback、Accept/Decline/Defer、generation失効、shutdown境界を検証する。
- 新規公開symbolのfeature ID 8は承認済みDDで提案された値を使う。headerと実装の完全性を同一commit内で確認する。

## 実装単位と順序

### 1. `UNIT-CORE-070` — Runtime UI Control stateとProperty接続

- `Window`、`StackPanel`、`Text`、`Button`のCore stateを追加し、既存TextBox stateとの共通Widget意味を整理する。
- IsEnabled、AccessibleName、Window title、Text content、Button labelを既存Property storeへ一度だけ登録し、default valueと独立Property ownerを実現する。
- 同一Runtime、UI thread、Detached初期状態、cleanup/shutdownを検証する。
- 対応試験: `UT-CONTROL-ABI-001`のCore側fixture/assertion。実装前にtest command/pathを実ファイルへ確定する。

### 2. `UNIT-CORE-071` — Window/Container Logical Tree接続

- 既存RuntimeTreeStore/TreeCoordinatorを使い、Windowの単一content root、Containerのinsert/remove/move/count/child accessを実装する。
- Tree owner再発行、Detached subtree、異Runtime、複数parent、cycle、範囲外index、準備失敗での原子性を検証する。
- 対応試験: `UT-TREE-ABI-001`のCore側focused assertions。親子状態・owner不変を注入可能なfixtureで検証する。

### 3. `UNIT-CORE-072` — StackPanel配置・Focus・Window close state

- Vertical/Horizontalとchild Alignの検査・commit・Layout dirty境界を実装する。
- Widget focus要求を既存/新設FocusManagerへ接続し、Detached/非active/disabled/non-focusable時のno-opを検証する。
- Window peer interfaceとCloseRequest state machineを実装し、同時subscriber上限、即時resolve/defer、未解決owner release、stale generation、shutdown teardownをfake peerで検証する。
- 対応試験: `UT-LAYOUT-ABI-001`、`UT-WINDOW-ABI-001`のCore側focused assertions。

### 4. `UNIT-ABI-020` — C ABI Control生成・Property・Tree・Command・配置API

- 承認済み宣言を`abi.h`と`crates/c-abi/src/lib.rs`へ反映し、Window/StackPanel/Text/Button生成、共通Property取得、Text/Button property、Button Command、Window content/Container Tree、StackPanel orientation/Alignを公開する。
- 必須out初期化、null/length、handle種別/Runtime/thread検査、status/error mapping、入力token非消費、失敗時状態不変を確認する。
- CとC++ header compile probeを更新し、symbol exportをC ABI focused testで検証する。
- 対応試験: `UT-CONTROL-ABI-001`、`UT-TREE-ABI-001`、`UT-LAYOUT-ABI-001`。

### 5. `UNIT-ABI-021` — C ABI Focus・Window peer/close requestとfeature完成

- `widget_request_focus`、Window open/close、close-request subscribe/defer/resolve symbolを公開し、subscription cancel/release lifecycleとfake peer接続を確認する。
- 全APIが完成した同revisionでのみfeature 8を有効化し、既存feature 1–7の値・結果を不変とする。
- C/C++ probeとABI runtime smokeを更新し、既存ABI利用が壊れていないことを確認する。
- 対応試験: `UT-WINDOW-ABI-001`および必要な既存ABI regression。

## 各unitの実行・commit規則

- unit着手時に`tools/units.yaml`へ`UNIT-CORE-070..072`および`UNIT-ABI-020..021`の実在source、依存、test design、test commandを登録してreadyにする。未着手unitはplannedのままにする。
- 各unitは対応source、focused test、header/probe等の同revision変更を含め、unitごとにcommitする。feature 8の有効化は最後のABI unitで行う。
- Unit別focused test後に`cargo fmt --all -- --check`、`cargo clippy --workspace --all-targets --locked -- -D warnings`、該当ABI probesを実行する。最終unitでは`.venv/Scripts/python.exe tools/dev.py check`と`.venv/Scripts/python.exe tools/dev.py minimum`を実行する。
- CIはbranch push後の同一commit SHAで確認し、失敗時は原因修正を同じunitに反映して再実行する。
- すべての変更は本Core APIs branchに限定し、wrapper branch、main、WebEngine branchへ混在させない。

## 完了条件

- 承認済みABI 1.3の全Control symbolと所有/thread/failure契約をCore/C ABIが満たす。
- UI test 0.3の追加試験、C/C++ header probe、existing ABI regressionが同じ実装revisionでpassする。
- feature 8は全契約の実装時だけtrueであり、既存feature behaviorと既存API compatibilityを維持する。
- unit台帳と実行記録が実revision・test command・結果へ対応する。
- branchはpush済みで、wrapper実装が必要ならこのbranchから対象言語child branchを作成できる。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-24 | 0.1 / 初期UI Control C ABI Core実装 | 利用者レビュー待ち | 計画案のみ。製品コード・unit台帳をまだ変更しない |
| 2026-10-25 | 0.1 / 初期UI Control C ABI Core実装 | 利用者 | 「ok」により計画全体を承認。unitごとの実装・試験・commitを開始 |
