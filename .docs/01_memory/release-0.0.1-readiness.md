# カテゴリ: 0.0.1 release readiness / UI vertical slice

## このmemoryの用途

0.0.1実装の次unitを決める前に、要求・設計・test・unit台帳の承認状態と依存を照合する際に参照する。

## 確認済み事項（2026-10-24）

- version branchは`version/0.0.1`。実装は`.docs/02_git/rule.md`に従いCore APIs child branchから始める。
- 現在の`tools/units.yaml`はschema 2で113 units（complete 76、ready 37）。`.docs/20_todo/release-0.0.1.md`の110（55/55）は計画作成時点の値として保持し、監査時点値と区別する。
- Core UI Treeの内部所有とRuntime UI object store統合はUNIT-CORE-067～069としてversion branchへ統合済み。C ABI Window/Controlとwrapper接続は別unit。
- `ui/controls.md`と`32_unit_test/ui.md`の承認履歴は全文承認済み。UI Control DDはpublic ABI signatureを別途ABI DDへ送っている。
- 現行C ABI DD/header/implementationにWindow・Widget・StackPanel・Text・Buttonを生成・接続するsymbol/APIはない。次の計画は初期UI C ABI契約のDD具体化から始め、承認前に実装しない。
- WebEngine branchは利用者による別途採否判断までversion branchへ統合しない。
