# カテゴリ: 0.0.1 release readiness / UI vertical slice

## このmemoryの用途

0.0.1実装の次unitを決める前に、要求・設計・test・unit台帳の承認状態と依存を照合する際に参照する。

## 確認済み事項（2026-10-24）

- version branchは`version/0.0.1`。実装は`.docs/02_git/rule.md`に従いCore APIs child branchから始める。
- 現在の`tools/units.yaml`はschema 2で113 units（complete 76、ready 37）。`.docs/20_todo/release-0.0.1.md`の110（55/55）は計画作成時点の値として保持し、監査時点値と区別する。
- Core UI Treeの内部所有とRuntime UI object store統合はUNIT-CORE-067～069としてversion branchへ統合済み。C ABI Window/Controlとwrapper接続は別unit。
- `ui/controls.md`と`32_unit_test/ui.md`の承認履歴は全文承認済み。UI Control DDはpublic ABI signatureを別途ABI DDへ送っている。
- 現行C ABI DD/header/implementationにWindow・Widget・StackPanel・Text・Buttonを生成・接続するsymbol/APIはない。次の計画は初期UI C ABI契約のDD具体化から始め、承認前に実装しない。
- ABI 1.3とUI test 0.3は2026-10-24に全文承認され、`f6b6adc`で`version/0.0.1`へcommit/push済み。feature ID 8も設計承認済みだが、実装はwrapperへ広げず`impl-rust/core-ui-controls-c-abi`のCore APIs単位で進める。
- UNIT-CORE-070は初期Control objectごとに型付きstateを持ち、共通Property registryのPropertyをInternalOwnerで保持する。新しいopaque handle registryを足した場合、共通retain/release経路にも登録する。4件のCore focused tests、workspace Clippy、fmtを通過。
- UNIT-CORE-071ではWindowの単一contentとStackPanelの順序付きchildrenを既存Logical Treeへ接続し、child_at/content getterでTreeOwnerから独立public handleを再発行する。UI Control token表には再発行tokenのRuntime/kind/identityも登録しないと、後続getter・retain/releaseが解決できない。TreeOwnerのreleaseはtree mutex解放後に行い、旧subtree cleanupをlock内で実行しない。
- UNIT-CORE-072ではLayout dirtyを変更箇所からLogical Treeの全祖先へ伝播する。Window peer/callbackはRuntime状態mutexを解放してから呼び、shutdown時もObjectStoreからstateを移譲して外部peer closeをlock外で実施する。
- UNIT-ABI-020ではButtonがCommand entryのArcを保持し、public token解放後もgetterで独立tokenを再発行できる。交換時の旧Arc dropはObjectStore mutex解放後に行い、別Runtime Commandは状態変更前に拒否する。Windows SDKが不足する開発端末でも、MinGW CMakeでC/C++ ABI header probeとruntime smokeを検証できる。
- UNIT-ABI-021ではWindow close subscriptionを既存Subscription cancel/release lifecycleへ接続する。callback中borrowed requestからdeferでowned tokenを発行し、最後の未解決owner releaseをDeclineにする。詳細は[Window close-requestの実装メモ](abi-window-close-request.md)。
- 初期UI ControlのCore/C ABI計画は`UNIT-CORE-070..072`、`UNIT-ABI-020..021`まで完了。最終revision `f87c082` のWindows/Linux CI run `40163386417`が成功した。次層のwrapperは対象言語ごとにこのCore APIs branchから子branchを作り、C# Controlの実装前にC# DDとwrapper test designを補う。
- C# UI Control wrapper計画は`dfb86cf`で登録され、親Core revisionのWindows/Linux CIも成功（run `40164472118`）。言語別branch `impl-csharp/ui-controls-wrapper`を作成。C# wrapper DD v1.5全文は利用者承認済み。実装sourceを変更する前にC# focused/consumer test designとunit mappingを登録し、実装開始の承認境界を維持する。
- WebEngine branchは利用者による別途採否判断までversion branchへ統合しない。
