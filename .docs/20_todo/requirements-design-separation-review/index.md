# 要求・設計分離 / 全案件レビュー案

- 文書版: 0.2
- 状態: 全案反映承認済み・正本適用済み（2026-10-17）。具体化未了は適用結果を参照
- 作成日: 2026-10-17
- 作業計画: [一括作成計画](../requirements-design-separation-batch-review.md)
- 元計画: [要求・設計分離](../requirements-design-separation.md)
- 元台帳: [全66文書](../requirements-design-separation-inventory.md)

## 読み方と承認範囲

2026-10-17の作成時点では案の一括作成が指示された。その後、利用者から全案反映の指示を受け、正本適用を完了した。元文書65ファイルと廃止済みDD-23の再分類記録を対象に、残る62文書案、横断課題10件、全15RQ章の改訂案を用意した。D00/D01/D02/D23の4件は適用済みとして既存記録を参照する。

仮要求候補は63件。既存要求の意味維持、要求補足、未決の選択を区別し、引用枠に要求本文案を示す。文書別案は原文箇所/承認範囲、RQ候補、改訂後の責務・data/ownership flow・state、残課題、ID移行を持つ。案の作成完了を仕様承認・実装・試験合格へ換算しない。

要求IDは現行187件を照合した。旧移行時の195 IDには現在廃止済みのRQ-08-001～008が含まれるため、現行件数と履歴上の件数を区別する。

文書内のレビューID `SEP-*` は作業用で、正式RQ/BD/DD/SD/test IDではない。文書分離だけの承認と、新しい要求・未決選択肢の採用を[承認台帳](approval-register.md)で区別する。旧判断IDは原則維持し、要求へ移す文だけを意味単位で対応付ける。

## 共通のレビュー入口

| 対象 | レビュー案 | 確認する内容 |
|---|---|---|
| 要求の本文案 | [全63候補](rq-candidates.md) | 最終的な機能・条件・結果。正式番号は移行表へ記録済み |
| 全現行要求 | [全RQ IDの移行](rq-id-map.md) | 187件を保持/分割/統合へ対応 |
| RQ-15 | [22 IDの意味単位移行](rq15-migration.md) | 条件の取りこぼしと重複正本を解消 |
| 検証 | [test改訂](test-impact.md)、[全RQ→case](rq-test-map.md) | 個別assertion、異常系/シナリオ、内部unitと外部条件 |
| DD内API/SD/運用 | [責務と移管案](sd-api-operations.md) | 未定API、暗号/trust、license/supportの所管 |
| ID/履歴/参照 | [移行手順](id-migration.md)、[全設計ID](design-id-map.md) | 旧新対応、承認範囲、参照方向 |
| 採否・適用 | [承認台帳](approval-register.md) | 要求・方式・case・選択肢を区別 |
| 作成coverage | [全案件照合](coverage.md) | SEP-01～07 / X01～10 / 全66文書 |

## 横断課題 X01～X10

| 項目 | 案 | 要点 |
|---|---|---|
| X01 | [公開参照とregistration解除](x01.md) | RQ-05/07を再度未承認へ戻さず、残るABI/DD本文の適用承認だけを記録する。要求testのRQ-05承認範囲以外は別レビューとして扱う。 |
| X02 | [値型とF32 ABI要求](x02.md) | どちらも現行要求の適合済み表示を禁止する。OBJECT/null/custom値や内蔵converterは将来候補を維持する。 |
| X03 | [終了要求の同期性・優先度](x03.md) | 停止済み/停止中loopで同期waitを解除する外部保証の採否が必要。timeout数値やcompletion observation APIをレビュー整理だけで新設しない。 |
| X04 | [Python通常終了とcleanup](x04.md) | atexit単独、全__del__確実実行、builtin/os._exit上書きで完遂を保証しない。外部hostへの登録義務を利用者条件として採用するかを選択する。 |
| X05 | [Collection/SelectionとGrid対象段階](x05.md) | 専用Collection/Selection DDが未計画というgapを残し、今回の分離案をその機能DD完成と記さない。 |
| X06 | [Control範囲と既定値](x06.md) | Vertical/Stretch、ancestor enabled、Focusable既定、visibility、直接Click非提供は文単位に採否を記録する。keybinding追加は既存利用者指示により将来メモへ保持する。 |
| X07 | [解析警告とmarker拡張](x07.md) | C++属性/C#属性/Rust macro/Python decoratorの構文、JIT、base-call必須method一覧は未確定。DD-24開始/実装採用まで一括承認したと扱わない。 |
| X08 | [署名・trust・schemaのSD移管](x08.md) | algorithmを未承認へ戻さず、未定parametersは未決を維持。署名の前にmanifestを無検査で意味利用する矛盾と、未知optional fieldを署名前に捨てる設計を避ける。 |
| X09 | [旧参照・進行履歴・縦断case](x09.md) | 参照修正で履歴を削除したり、廃止RQ08を復活させたりしない。 |
| X10 | [意味単位のIDとAPI具体化](x10.md) | IDの穴詰めや自動全置換、独立API工程の復活は行わない。case ID、実test ID、実装unit IDの混同を移行監査で検出する。 |

## 要求章ごとの改訂案

| 章 | 案 |
|---|---|
| RQ-01 | [共通機能とアーキテクチャ制約](rq-01.md) |
| RQ-02 | [Core機能と値の外部結果](rq-02.md) |
| RQ-03 | [言語別保証と解析能力](rq-03.md) |
| RQ-04 | [ABIの値・失敗・互換条件](rq-04.md) |
| RQ-05 | [参照寿命・cleanup・callback](rq-05.md) |
| RQ-06 | [dispatch・shutdown・Command](rq-06.md) |
| RQ-07 | [Bindingと通知・検証](rq-07.md) |
| RQ-09 | [UI機能・編集・次段階の範囲](rq-09.md) |
| RQ-10 | [Accessibilityの外部意味](rq-10.md) |
| RQ-11 | [platform範囲と分離制約](rq-11.md) |
| RQ-12 | [配布・互換・信頼・保守](rq-12.md) |
| RQ-13 | [非機能の測定要求](rq-13.md) |
| RQ-14 | [実施順とunit-testの管理要求](rq-14.md) |
| RQ-15 | [判断履歴から要求正本への統合](rq-15.md) |
| RQ-16 | [文書ロール・履歴・不整合](rq-16.md) |

## BD番号付き37文書

| 台帳項目 | 元文書・記録 | レビュー案 / 適用済み記録 | 状態 |
|---|---|---|---|
| B00 | [MVVM UIライブラリ 上位設計再開計画](../../12_design/BD-00_basic-design-resumption.md) | [B00の案/記録](b00.md) | 分離改訂承認済み・適用済み |
| B01 | [MVVM UIライブラリ 初期公開値型の上位判断](../../12_design/BD-01_value-types-upper-design.md) | [B01の案/記録](b01.md) | 分離改訂承認済み・適用済み |
| B02 | [MVVM UIライブラリ UI transactionの上位判断](../../12_design/BD-02_ui-transaction-upper-design.md) | [B02の案/記録](b02.md) | 分離改訂承認済み・適用済み |
| B03 | [MVVM UIライブラリ Runtime通知上限・診断の上位判断案](../../12_design/BD-03_runtime-notification-upper-design.md) | [B03の案/記録](b03.md) | 分離改訂承認済み・適用済み |
| B04 | [MVVM UIライブラリ UI treeの所有・寿命に関する上位判断案](../../12_design/BD-04_ui-tree-lifetime-upper-design.md) | [B04の案/記録](b04.md) | 分離改訂承認済み・適用済み |
| B05 | [MVVM UIライブラリ Logical Tree／Visual Treeの役割に関する上位判断案](../../12_design/BD-05_logical-visual-tree-upper-design.md) | [B05の案/記録](b05.md) | 分離改訂承認済み・適用済み |
| B06 | [MVVM UIライブラリ Layoutの責務境界に関する上位判断案](../../12_design/BD-06_layout-upper-design.md) | [B06の案/記録](b06.md) | 分離改訂承認済み・適用済み |
| B07 | [MVVM UIライブラリ Input eventの責務境界に関する上位判断案](../../12_design/BD-07_input-event-boundary-upper-design.md) | [B07の案/記録](b07.md) | 分離改訂承認済み・適用済み |
| B08 | [MVVM UIライブラリ Focusの管理単位に関する上位判断案](../../12_design/BD-08_focus-upper-design.md) | [B08の案/記録](b08.md) | 分離改訂承認済み・適用済み |
| B09 | [MVVM UIライブラリ Focus移動時のIME compositionに関する上位判断案](../../12_design/BD-09_ime-focus-transition-upper-design.md) | [B09の案/記録](b09.md) | 分離改訂承認済み・適用済み |
| B10 | [MVVM UIライブラリ IME Platform境界に関する上位判断案](../../12_design/BD-10_ime-platform-boundary-upper-design.md) | [B10の案/記録](b10.md) | 分離改訂承認済み・適用済み |
| B11 | [MVVM UIライブラリ 初期Control契約に関する上位判断案](../../12_design/BD-11_initial-control-contract-upper-design.md) | [B11の案/記録](b11.md) | 分離改訂承認済み・適用済み |
| B12 | [MVVM UIライブラリ 初期描画方式と検証順序に関する上位判断案](../../12_design/BD-12_rendering-platform-upper-design.md) | [B12の案/記録](b12.md) | 分離改訂承認済み・適用済み |
| B13 | [MVVM UIライブラリ Accessibility共通意味論に関する上位判断案](../../12_design/BD-13_accessibility-upper-design.md) | [B13の案/記録](b13.md) | 分離改訂承認済み・適用済み |
| B14 | [MVVM UIライブラリ Python wrapper方式に関する上位判断案](../../12_design/BD-14_python-wrapper-upper-design.md) | [B14の案/記録](b14.md) | 分離改訂承認済み・適用済み |
| B15 | [MVVM UIライブラリ Python lint/type check とMCP連携に関する上位判断案](../../12_design/BD-15_python-lint-mcp-upper-design.md) | [B15の案/記録](b15.md) | 分離改訂承認済み・適用済み |
| B16 | [MVVM UIライブラリ 初期Build／CI target matrixに関する上位判断案](../../12_design/BD-16_build-ci-target-matrix-upper-design.md) | [B16の案/記録](b16.md) | 分離改訂承認済み・適用済み |
| B17 | [MVVM UIライブラリ Python package/native distribution](../../12_design/BD-17_python-package-distribution-upper-design.md) | [B17の案/記録](b17.md) | 分離改訂承認済み・適用済み |
| B18 | [MVVM UIライブラリ C# NuGet native asset](../../12_design/BD-18_csharp-nuget-native-assets-upper-design.md) | [B18の案/記録](b18.md) | 分離改訂承認済み・適用済み |
| B19 | [MVVM UIライブラリ C++ CMake package](../../12_design/BD-19_cpp-cmake-package-upper-design.md) | [B19の案/記録](b19.md) | 分離改訂承認済み・適用済み |
| B20 | [MVVM UIライブラリ Rust Cargo packageに関する上位判断](../../12_design/BD-20_rust-cargo-package-upper-design.md) | [B20の案/記録](b20.md) | 分離改訂承認済み・適用済み |
| B21 | [MVVM UIライブラリ first-party licenseに関する上位判断案](../../12_design/BD-21_license-upper-design.md) | [B21の案/記録](b21.md) | 分離改訂承認済み・適用済み |
| B22 | [MVVM UIライブラリ サポート期間に関する上位判断案](../../12_design/BD-22_support-lifecycle-upper-design.md) | [B22の案/記録](b22.md) | 分離改訂承認済み・適用済み |
| B23 | [MVVM UIライブラリ text shaping／Unicode対応の上位判断案](../../12_design/BD-23_text-shaping-upper-design.md) | [B23の案/記録](b23.md) | 分離改訂承認済み・適用済み |
| B24 | [MVVM UIライブラリ IME edit sessionに関する上位判断案](../../12_design/BD-24_ime-edit-session-upper-design.md) | [B24の案/記録](b24.md) | 分離改訂承認済み・適用済み |
| B25 | [MVVM UIライブラリ 非同期Validationの初版範囲に関する上位判断案](../../12_design/BD-25_async-validation-upper-design.md) | [B25の案/記録](b25.md) | 分離改訂承認済み・適用済み |
| B26 | [MVVM UIライブラリ Async Command進捗配信に関する上位判断案](../../12_design/BD-26_async-command-progress-upper-design.md) | [B26の案/記録](b26.md) | 分離改訂承認済み・適用済み |
| B27 | [MVVM UIライブラリ Error objectの安定性境界に関する上位判断案](../../12_design/BD-27_error-object-upper-design.md) | [B27の案/記録](b27.md) | 分離改訂承認済み・適用済み |
| B28 | [MVVM UIライブラリ ABI release version schemeに関する上位判断案](../../12_design/BD-28_abi-release-version-upper-design.md) | [B28の案/記録](b28.md) | 分離改訂承認済み・適用済み |
| B29 | [MVVM UIライブラリ Native artifact provenanceに関する上位判断案](../../12_design/BD-29_artifact-provenance-upper-design.md) | [B29の案/記録](b29.md) | 分離改訂承認済み・適用済み |
| B30 | [MVVM UIライブラリ Native artifact配布経路に関する上位判断案](../../12_design/BD-30_artifact-distribution-upper-design.md) | [B30の案/記録](b30.md) | 分離改訂承認済み・適用済み |
| B31 | [MVVM UIライブラリ Artifact署名trust policyに関する上位判断案](../../12_design/BD-31_artifact-signing-trust-upper-design.md) | [B31の案/記録](b31.md) | 分離改訂承認済み・適用済み |
| B32 | [MVVM UIライブラリ Artifact manifest schema evolutionに関する上位判断案](../../12_design/BD-32_artifact-manifest-schema-upper-design.md) | [B32の案/記録](b32.md) | 分離改訂承認済み・適用済み |
| B33 | [MVVM UIライブラリ CollectionModel／Gridに関する上位判断案](../../12_design/BD-33_collection-grid-upper-design.md) | [B33の案/記録](b33.md) | 分離改訂承認済み・適用済み |
| B34 | [MVVM UIライブラリ SelectionModel意味論に関する上位判断案](../../12_design/BD-34_selection-model-upper-design.md) | [B34の案/記録](b34.md) | 分離改訂承認済み・適用済み |
| B35 | [MVVM UIライブラリ Collection change batch原子性に関する上位判断](../../12_design/BD-35_collection-change-batch-upper-design.md) | [B35の案/記録](b35.md) | 分離改訂承認済み・適用済み |
| B36 | [Runtime shutdownのcallback中dispatchに関する上位判断](../../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md) | [B36の案/記録](b36.md) | 分離改訂承認済み・適用済み |

## 基本設計5文書

| 台帳項目 | 元文書・記録 | レビュー案 / 適用済み記録 | 状態 |
|---|---|---|---|
| S01 | [MVVM UIライブラリ 設計仕様骨格](../../12_design/design-overview.md) | [S01の案/記録](s01.md) | 分離改訂承認済み・適用済み |
| S02 | [MVVM UIライブラリ Runtime契約（Slice 1）](../../12_design/runtime-contract.md) | [S02の案/記録](s02.md) | 分離改訂承認済み・適用済み |
| S03 | [MVVM UIライブラリ C ABI仕様案（Slice 1基盤・Slice 2 MVVM）](../../12_design/abi-spec.md) | [S03の案/記録](s03.md) | 分離改訂承認済み・適用済み |
| S04 | [MVVM意味論 — Property・Binding・Command・Validation（Slice 2）](../../12_design/mvvm-semantics.md) | [S04の案/記録](s04.md) | 分離改訂承認済み・適用済み |
| S05 | [言語Binding設計 — ViewModel接続の第1段階](../../12_design/language-bindings.md) | [S05の案/記録](s05.md) | 分離改訂承認済み・適用済み |

## DD24項目

| 台帳項目 | 元文書・記録 | レビュー案 / 適用済み記録 | 状態 |
|---|---|---|---|
| D00 | [DD-00 詳細設計の出典と判断の対応表](../../13_detail_design/00-decision-map.md) | [D00の案/記録](../dd00-current-rq-alignment.md) | 適用済み（要求候補は別途） |
| D01 | [DD-01 Handle登録と検証](../../13_detail_design/01-handle-registry.md) | [D01の案/記録](../dd01-requirements-design-separation.md) | 適用済み（要求候補は別途） |
| D02 | [DD-02 所有権と終了](../../13_detail_design/02-runtime-lifetime.md) | [D02の案/記録](../dd02-requirements-design-separation.md) | 適用済み（要求候補は別途） |
| D03 | [DD-03 UI threadと投稿キュー](../../13_detail_design/03-dispatcher.md) | [D03の案/記録](d03.md) | 分離改訂承認済み・適用済み |
| D04 | [DD-04 Callbackと再入](../../13_detail_design/04-callback-lifecycle.md) | [D04の案/記録](d04.md) | 分離改訂承認済み・適用済み |
| D05 | [DD-05 値の受渡しと失敗契約](../../13_detail_design/05-boundary-values-errors.md) | [D05の案/記録](d05.md) | 分離改訂承認済み・適用済み |
| D06 | [DD-06 ABIシンボルと互換性](../../13_detail_design/06-abi-surface.md) | [D06の案/記録](d06.md) | 分離改訂承認済み・適用済み |
| D07 | [DD-07 Propertyの保存と通知](../../13_detail_design/07-property-notification.md) | [D07の案/記録](d07.md) | 分離改訂承認済み・適用済み |
| D08 | [DD-08 Bindingの伝播とグラフ](../../13_detail_design/08-binding-graph.md) | [D08の案/記録](d08.md) | 分離改訂承認済み・適用済み |
| D09 | [DD-09 ValidationとView入力](../../13_detail_design/09-validation-input.md) | [D09の案/記録](d09.md) | 分離改訂承認済み・適用済み |
| D10 | [DD-10 非同期Command](../../13_detail_design/10-async-command.md) | [D10の案/記録](d10.md) | 分離改訂承認済み・適用済み |
| D11 | [DD-11 UI treeと要素寿命](../../13_detail_design/11-ui-tree-lifetime.md) | [D11の案/記録](d11.md) | 分離改訂承認済み・適用済み |
| D12 | [DD-12 Layout](../../13_detail_design/12-layout.md) | [D12の案/記録](d12.md) | 分離改訂承認済み・適用済み |
| D13 | [DD-13 InputとFocus](../../13_detail_design/13-input-focus.md) | [D13の案/記録](d13.md) | 分離改訂承認済み・適用済み |
| D14 | [DD-14 TextBoxとIME](../../13_detail_design/14-textbox-ime.md) | [D14の案/記録](d14.md) | 分離改訂承認済み・適用済み |
| D15 | [DD-15 初期Control](../../13_detail_design/15-controls-v1.md) | [D15の案/記録](d15.md) | 分離改訂承認済み・適用済み |
| D16 | [DD-16 描画とPlatform Adapter](../../13_detail_design/16-platform-rendering.md) | [D16の案/記録](d16.md) | 分離改訂承認済み・適用済み |
| D17 | [DD-17 Accessibility](../../13_detail_design/17-accessibility.md) | [D17の案/記録](d17.md) | 分離改訂承認済み・適用済み |
| D18 | [DD-18 C++ wrapper](../../13_detail_design/18-wrapper-cpp.md) | [D18の案/記録](d18.md) | 分離改訂承認済み・適用済み |
| D19 | [DD-19 C# wrapper](../../13_detail_design/19-wrapper-csharp.md) | [D19の案/記録](d19.md) | 分離改訂承認済み・適用済み |
| D20 | [DD-20 Rust safe API](../../13_detail_design/20-wrapper-rust.md) | [D20の案/記録](d20.md) | 分離改訂承認済み・適用済み |
| D21 | [DD-21 Python 3 wrapper](../../13_detail_design/21-wrapper-python.md) | [D21の案/記録](d21.md) | 分離改訂承認済み・適用済み |
| D22 | [DD-22 Buildと配布](../../13_detail_design/22-build-distribution.md) | [D22の案/記録](d22.md) | 分離改訂承認済み・適用済み |
| D23 | [旧DD-23 / 要求testルールへ再分類](../../30_release_test/requirements-test-manifest.md) | [D23の案/記録](../requirements-test-manifest.md) | 適用済み（要求候補は別途） |

## 選択を必要とする主な項目

以下は作成時点の比較である。推奨案の採用と未設計範囲は末尾の適用結果を正本とする。その他の未決も各文書の改訂判断案に記載する。

| 要求候補 | 推奨・代替と未決理由 |
|---|---|
| [SEP-RQ-010 停止したUI loopへの同期要求](rq-candidates.md#candidate-010) | 選択要。推奨はhost停止通知による未完了結果。代替は利用者設定timeoutを併用。任意の秒数・進行中cleanupの強制中断は今回定めない。 |
| [SEP-RQ-011 ABI F32要求の未対応範囲](rq-candidates.md#candidate-011) | 差異解決要。推奨は現行F32要求を保持し、F32利用APIを未対応として計画化。代替はF32の対象段階を要求承認で変更。型宣言があるだけで充足扱いにしない。 |
| [SEP-RQ-027 TextBox自身のProperty検証拒否](rq-candidates.md#candidate-027) | 選択要。推奨Bは編集候補保持＋確定値旧値＋専用拒否理由。代替Aは編集前へ戻す、Cは新状態追加。既存未決DD-TEXT-09を確定扱いにしない。 |
| [SEP-RQ-028 IME cancel後の入力理由](rq-candidates.md#candidate-028) | 選択要。推奨は開始snapshotの状態/理由も復元。代替はcancel専用の入力結果を追加。既承認は文字列・位置復元まで。 |
| [SEP-RQ-029 IME commit準備失敗](rq-candidates.md#candidate-029) | 選択要。Core事前退避とAdapter再送保証を比較し、OS別に成立を確認する。無限のOOM下でも全入力保持という実現不能な無条件保証は追加しない。 |
| [SEP-RQ-035 StackPanel公開設定と既定](rq-candidates.md#candidate-035) | 範囲確認要。方向/Alignの提供はRQへ補足。Vertical/Stretch等の初期値は設定省略時の結果を保証するかを別に確認し、保証するならRQへ追加。計算式はDDへ。 |
| [SEP-RQ-036 有効性とFocusの共通結果](rq-candidates.md#candidate-036) | 範囲確認要。ancestor enabledとFocusable既定の外部保証はRQ。Focus復帰候補・同一要求no-opはDD。 |
| [SEP-RQ-037 初期機能の除外とButton未接続](rq-candidates.md#candidate-037) | 選択要。key割当除外は利用者指示を維持。CommandなしButtonは表示・無操作を推奨、生成拒否が代替。visibility/直接Click非提供は既存要求への影響を確認する独立範囲論点。 |
| [SEP-RQ-041 Accessibility nameの空欄](rq-candidates.md#candidate-041) | 範囲確認要。既存RQ-10-003は自動命名を未定義としているため、非提供の確定は今回要求承認を必要とする。 |
| [SEP-RQ-046 Python通常終了・例外と保証外](rq-candidates.md#candidate-046) | 境界選択要。管理外threadの検知はhost登録/entry wrapper契約を前提にする案。移植可能な検知元が未確定の経路は設計gapとし、atexit単独やbuiltin上書きで保証済みとしない。 |
| [SEP-RQ-051 artifactの信頼と検証失敗](rq-candidates.md#candidate-051) | RQ-12へ分割移管案。期限切れmetadataの利用可否と緊急失効の運用条件は別選択要。暗号方式はSD。 |

## 適用順の案

案の作成は一括で完了した。正本への適用は、RQの採否→requirements-testのassertion整合→BD/DD/SDへの条件・方式分離→ID/履歴/リンク監査の順で行う。独立した文書整形は承認済み範囲を保って先に適用できる。X01既承認範囲とD00/D01/D02/D23適用実績は再提案から切り分ける。

Collection/Selection専用DD、callback解析拡張DD-24、具体backend/profile、言語公開API名、署名envelope/鍵運用等は、今回の「分離レビュー案がある」ことと機能設計が完成したことを区別する。


## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-17 | 全案件レビュー案 | — | 一括作成指示に基づくレビュー案。採用・正本適用は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 状態 |
|---|---|---|---|
| 0.1 | 2026-10-17 | 現行文書から責務分離・改訂文案・影響を作成 | レビュー待ち |

## 2026-10-17 適用結果

正本は [適用結果](../requirements-design-separation-application-result.md) と [正式要求ID移行](../../11_requirements/requirements-separation-id-migration.md) に記録した。以下の比較・引用・選択肢は作成時点の案として保持する。採用した選択は推奨案、未提示の実装詳細は後続課題として扱った。

### 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全レビュー案の反映 | 利用者 | 「すべて反映して下さい。」により採用・正本適用を承認 |

### 改訂履歴

| 版 | 日付 | 内容 |
| --- | --- | --- |
| 0.2 | 2026-10-17 | 承認と正本適用結果を追記。作成時点の比較・引用を保持 |
