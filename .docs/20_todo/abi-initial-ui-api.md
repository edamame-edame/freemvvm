# 初期UI Control C ABI契約の具体化

- 計画版: 0.1
- 状態: 利用者レビュー待ち（本計画は未commit）
- 作成日: 2026-10-24
- 計画branch: `version/0.0.1`
- 対象: 承認済み初期Controlの意味論を、実装可能な共通C ABI操作契約へ具体化する

## 背景

[基線監査](release-0.0.1-readiness-audit.md)で、Window、StackPanel、Text、TextBox、Buttonの要求およびControl意味論・単体試験設計は承認済みだが、[共通C ABI詳細設計](../13_detail_design/runtime-objects/abi.md)にはこれらのobject作成・Tree接続・Control property/Command接続を表すsymbol/API tableがなく、C header・Core・ABI実装にも該当APIがないことを確認した。C ABI契約を実装前に具体化する[RQ-14-001/002]と設計文書の役割に従い、コード実装より前にDD上の境界を定める。

## 目的

承認済み要求とControl/Tree/Platform設計から、初期UI ControlをC ABI経由で構成する操作、所有・thread・failure境界、共通Property/Binding/Commandとの接点を整理する。文書改訂案を利用者がレビューするまで新しいAPI方式を確定しない。

## 対象範囲

- Window、StackPanel、Text、TextBox、Buttonの生成・解放とRuntime所属。
- Widget共通意味状態へのアクセスと、Containerの子Tree操作。
- StackPanel配置設定、Text/TextBox値と共通Property/Binding、ButtonとCommandの接続。
- WindowのCore object寿命とPlatform peer/open/close操作、Runtime shutdownとの境界。
- 各APIのowned/borrowed、UI thread/context、入力検査、成功commit点、失敗時出力・所有保持、ABI feature/version対応。
- 初期Controlだけに必要な機能に限定し、次段階Control・Style/Template等は扱わない。

## 対象外

- Core、C ABI、wrapperの製品コード・配布headerの変更。
- Renderer、OS peer、描画、IME、Accessibility providerの実装方式変更。既承認のPlatform DDを根拠とする。
- 既存RQの保証や例外の変更。必要な差分が見つかった場合は要求側レビューへ分離する。
- WebEngine branchの採否・統合、release tag・公開。

## 作業手順

1. RQ-04、RQ-05、RQ-06、RQ-07、RQ-09、RQ-10、RQ-14と、承認済み`ui/controls.md`、`ui/tree-layout.md`、`ui/input-focus.md`、`ui/text-editing.md`、Platform DD、C ABI 1.2を項目横断で照合する。
2. 初期Controlごとに必要なcreate/get/set/tree/command/open/close操作と既存Property・Binding・Command ABIの再利用点を分類する。利用者に見える結果と既存契約から導けない点は未決質問として分離し、実装都合で決定しない。
3. `.docs/13_detail_design/runtime-objects/abi.md`の次版案へ操作表を追加する。引数型、所有、thread、検査順、commit点、失敗時out、feature/version、release/cancelの違いを各操作で示す。既存symbolと既存status意味は変更しない。
4. 必要な場合のみ`.docs/13_detail_design/ui/controls.md`の参照・境界を更新し、UI意味論を重複記述しない。
5. `.docs/32_unit_test/ui.md`へC ABI境界の局所試験条件を追加し、`TC-TREE-01`、`TC-LAYOUT-01`、`TC-VERT-*`等との対応を確認する。OS描画・IMEのsystem testをABI単体試験の合格へ読み替えない。
6. 変更した設計文書全体、test文書全体、参照・feature inventoryを利用者レビューに提示する。承認後に文書承認履歴・改訂履歴、unit台帳案を同期する。
7. この計画と承認済み設計改訂を`version/0.0.1`へcommit/pushする。コード実装は別のCore APIs child branchで、実装・testを同じunit revisionにまとめる。

## 検討する決定事項

- C ABIで公開する初期UI objectの集合と、生成順・型安全なhandle/token表現。
- Window content root、Container child操作、Detached subtreeと外部ownerの操作境界。
- Text/TextBoxの共通Property生成・取得方法、Binding可能性とTextBox編集snapshotへの接続。
- Button label/AccessibleName/Command参照の設定と、Command ownerから独立した寿命。
- Window open/close、OS close intent、Core Window保持、Runtime shutdownの分離境界。
- ABI feature groupの切り方と既存feature IDとの互換追加方法。
- OOM・無効handle・所属Runtime不一致・wrong thread・closed Runtime・部分準備失敗時に既存状態と入力所有を維持する操作ごとの規則。

この項目はレビュー範囲の見落とし防止用であり、設計項目別ID・項目別承認台帳ではない。決定結果はC ABI詳細設計文書全体を単位に承認する。

## 完了条件

- 初期Controlの要求とC ABI操作が追跡され、既存Core/Property/Binding/Commandの責務と重複していない。
- 操作ごとの型・thread・所有・commit・失敗境界が実装可能な粒度で記述される。
- RQ/DDから決められない事項は明示的に利用者レビュー待ちとなり、確定仕様扱いされない。
- UI単体testおよび後続system scenarioとの分担が明確で、局所的に試験可能。
- API設計文書全体の承認を得る。これは実装・試験合格・release許可を意味しない。

## ブランチと変更単位

- 本計画・設計案は`version/0.0.1`上で管理する。
- C ABI/Core実装は`.docs/02_git/rule.md`に従い、本version branchを親とするCore APIs childから着手する。
- C++/C#/Rust/Python wrapperは、その時点の最新Core APIs branchを親に個別に作り、別言語wrapperを親にしない。
- WebEngine branchは対象に含めない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-24 | 0.1 / 初期UI Control C ABI契約の具体化 | 利用者レビュー待ち | 計画のみ。設計方式・実装・公開を承認済みと扱わない |
