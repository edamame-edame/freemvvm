# 未設計レビュー / 次段階UI・将来候補の範囲と拡張点

- 文書版: 0.2
- 状態: 全判断承認済み・正本適用済み（2026-10-17）
- 判断ID: REM-SCOPE-01～08
- 入口: [全件索引](index.md)
- 根拠RQ: [RQ-09-012](../../11_requirements/RQ-09_ui-scope.md#rq-09-012)、[RQ-09-019](../../11_requirements/RQ-09_ui-scope.md#rq-09-019)、[RQ-09-021](../../11_requirements/RQ-09_ui-scope.md#rq-09-021)、[RQ-09-022](../../11_requirements/RQ-09_ui-scope.md#rq-09-022)、[RQ-09-023](../../11_requirements/RQ-09_ui-scope.md#rq-09-023)、[RQ-09-024](../../11_requirements/RQ-09_ui-scope.md#rq-09-024)、[RQ-09-025](../../11_requirements/RQ-09_ui-scope.md#rq-09-025)、[RQ-09-026](../../11_requirements/RQ-09_ui-scope.md#rq-09-026)、[RQ-09-047](../../11_requirements/RQ-09_ui-scope.md#rq-09-047)、[RQ-09-054](../../11_requirements/RQ-09_ui-scope.md#rq-09-054)
- 具体化するBD: [BD-05](../../12_design/BD-05_logical-visual-tree-upper-design.md)、[BD-11](../../12_design/BD-11_initial-control-contract-upper-design.md)、[BD-23](../../12_design/BD-23_text-shaping-upper-design.md)、[BD-25](../../12_design/BD-25_async-validation-upper-design.md)、[BD-33](../../12_design/BD-33_collection-grid-upper-design.md)
- 適用対象DD/SD: [DD-11](../../13_detail_design/11-ui-tree-lifetime.md)、[DD-13](../../13_detail_design/13-input-focus.md)、[DD-14](../../13_detail_design/14-textbox-ime.md)、[DD-15](../../13_detail_design/15-controls-v1.md)

本書は承認したレビュー記録である。推奨案と代替理由は提案時点の比較として保持する。現行の方式・正式判断IDは[DD-39: 次段階UI・将来候補の範囲と拡張点](../../13_detail_design/39-next-stage-ui.md)を正本とする。採用済みでも本文に将来候補と記した範囲は初期必須へ昇格しない。

## 判断案

| 判断ID | 推奨案・具体的な選択 | 代替と選択理由 | 承認区分 |
| --- | --- | --- | --- |
| REM-SCOPE-01 | 次段階Controlは既存採用リストを保持し、CheckBox/ToggleButtonはBOOL Property、ProgressBarは進捗値、ScrollViewerはviewportの独立modelへ接続する案。 | 今回の案作成を初期releaseへの採用にしない。 | 次段階DD |
| REM-SCOPE-02 | ListView/仮想化ListはCollection/Selection snapshotを使い、可視item presenterを再利用。item templateはitem ID+snapshotからvisualを構成する。 | 全itemを恒久Control/Binding化する方式を必須にしない。 | 次段階DD |
| REM-SCOPE-03 | Styleはtyped Property setter集合、Themeは共有ResourceMap、TemplateはControl-owned VisualFactoryとしLogical childを増やさない。 | 任意scriptをTemplateの評価へ暗黙導入しない。 | 次段階DD・REM-RQ-31 |
| REM-SCOPE-04 | Resource lookupはlocal→Logical ancestor→Runtime themeの順、未定義は明示error。DataContext自動伝播/name scopeは別候補として初期追加しない。 | 広域global mutable dictionaryに状態を隠さない。 | RQ・REM-RQ-31 |
| REM-SCOPE-05 | keybinding/Tab traversal/Ctrl+Z/UndoRedo/multi-seatは将来メモを維持し、今回のmandatory APIにしない。拡張点はsemantic actionとFocus serviceへ置く。 | 具体キーを再導入してユースケースを限定しない。 | 既承認範囲確認 |
| REM-SCOPE-06 | custom値/nullable/OBJECTは未採用。将来はtype descriptor+version+copy/equal/destroyと循環防止の契約が揃ってから新kindを割り当てる。 | 既存OBJECT予約を任意言語objectの抜け道にしない。 | 候補設計 |
| REM-SCOPE-07 | Core-managed async Validationは初期非提供を維持。ViewModelのrevision付きAsyncCommand+結果Propertyを推奨構成として示す。 | サーバ照会を同期Validatorで待たない。 | 既承認範囲確認 |
| REM-SCOPE-08 | Grid/TableView/TreeView/ComboBox、RTL/emoji/他script、画像codec、template script等はfeature候補。共通model設計だけで個別Controlを採用しない。 | 候補を必須test/pass数へ混入させない。 | 範囲確認 |

<a id="rem-scope-01"></a>

<a id="rem-scope-02"></a>

<a id="rem-scope-03"></a>

<a id="rem-scope-04"></a>

<a id="rem-scope-05"></a>

<a id="rem-scope-06"></a>

<a id="rem-scope-07"></a>

<a id="rem-scope-08"></a>

## 実現設計案

### 次段階の責務案

```text
Style { typed_setters, applicability }   // immutable
ResourceMap { key -> immutable value }   // generation付き
Theme { Arc<ResourceMap>, control_styles }
Template { VisualFactory, resource_scope } // owner Controlに従属
ItemPresenter { item_id, snapshot_revision, reusable_visual }
ViewportModel { offset, extent, viewport_size }
```

Style適用は全setterをprepareして成功時だけvisual/property stateへcommitし、入力/disabled/focus/Validation意味論をStyleが上書きしない。Theme切替はresource generationを進め参照中Visualをdirtyにする。Templateの旧Visualは新Visualのprepare成功後に切り替え、Core Logical ownership・Accessibility identityを維持する。

Resourceの見つからないkey、型不一致、lookup cycleは明示error。Template内部VisualからLogical parentを変えず、ItemPresenter再利用では旧item ID/sessionを失効させてから新snapshotへbindする。非可視itemのデータをCollectionから削除しない。

将来のCustomTypeDescriptor案は`type_id, schema_version, immutable_payload, clone, equals, destroy`。登録成功でcopy/destroy code lifetimeをRuntimeと結び、言語objectの直接pointerはABIへ渡さない。現時点では採用せず、nullableのtag比較とOBJECT参照cycle判定も拡張featureに分ける。

Async Validation推奨例は、入力revisionを増加→AsyncCommandで検査→結果revision一致なら状態Property更新。古い結果を適用せず、キャンセル要求と実終端を区別する。これはアプリ構成例でありCore async Validator APIを新設する案ではない。

## 適用結果

全判断を[DD-39](../../13_detail_design/39-next-stage-ui.md)へ正式IDで適用した。関連要求の変更もRQ本文へ反映した。実装・試験実行は未完了。[全200件の適用表](../remaining-design-application-result.md)で旧新対応を確認できる。

## 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 本書の案作成 | 利用者 | 一括生成指示。方式採用・正本適用は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 現行の未設計箇所を実現案・代替・interfaceへ具体化 | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.2 / 本書の全推奨案と補足の採用・正本適用 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.2 | 2026-10-17 | 本書の全推奨案と補足の採用・正本適用を正本へ適用し、旧判断との対応を保持 | Codex |
