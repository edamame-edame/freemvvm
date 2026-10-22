# 適用前記録 / 00-decision-map.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [00-decision-map.md](../../13_detail_design/00-decision-map.md)

---

# DD-00 詳細設計の出典と判断の対応表

- 文書版: 0.2
- 状態: 2026-09-27承認範囲を維持し、現行整合改訂を承認済み（2026-10-15）
- 更新日: 2026-10-15
- 対象: 詳細設計の上位出典・要求根拠・基本設計判断の対応
- 目的: 詳細設計で前提とする承認済み要求・基本設計・ADRと、未決の境界を整理する

本書は詳細設計が参照する上流の判断と要求の対応を整理する。**新しい製品要求・設計判断は行わない。** 基本設計判断が承認済みであることは、個別の詳細設計や実装方式が承認済みであることを意味しない。詳細設計の作業割当と進捗は作業計画で管理する。

## 1. 出典と優先関係

| 出典 | 確認した状態・範囲 | 詳細設計での扱い |
|---|---|---|
| [ADR-0001](../../04_approval/adr-0001-rust-core.md) | `Accepted` | Rust Core、versioned C ABI、4言語wrapper、組み込みLinuxを前提とする |
| 要求仕様 | 各RQ章の要求本文・承認履歴を参照 | 対象範囲と利用者から見える条件の上位根拠とする。共通アーキテクチャは[RQ-01](../../11_requirements/RQ-01_purpose-architecture.md)～[RQ-03](../../11_requirements/RQ-03_language-support.md)、ABI・所有・thread/callbackは[RQ-04](../../11_requirements/RQ-04_abi-boundary.md)～[RQ-06](../../11_requirements/RQ-06_threading-dispatch.md)、Bindingは[RQ-07](../../11_requirements/RQ-07_binding.md)、UI/Accessibilityは[RQ-09](../../11_requirements/RQ-09_ui-scope.md)・[RQ-10](../../11_requirements/RQ-10_accessibility.md)、Platform・配布・非機能・実施順は[RQ-11](../../11_requirements/RQ-11_platform.md)～[RQ-14](../../11_requirements/RQ-14_delivery-sequence.md)を参照する。要求上の決定事項は[RQ-15](../../11_requirements/RQ-15_requirement-decisions.md)、文書ロール・要求承認と不整合処理は[RQ-16-003/004](../../11_requirements/RQ-16_document-governance.md)に従う。本文で追跡する場合は該当RQ IDを記し、未決事項を確定扱いしない |
| [設計骨格](design-overview.md) | `Accepted` | 責務分割、依存順、初期縦断シナリオを参照する。個別の後続設計事項を確定扱いしない |
| [Runtime契約](runtime-contract.md) | R1～R4承認済み | handle、所有権、thread、callbackの承認範囲を参照する |
| [C ABI仕様案](abi-spec.md) | A1～A32承認済み。A5の初期値型の種類数をBD-01が改訂 | C宣言はレビュー用であり、配布用ヘッダ・実装済みAPIではない。F64/DECIMALの具体ABIは未設計 |
| [MVVM意味論](mvvm-semantics.md) | P1～P4、B1～B4、C1～C4、D1～D4、V1～V8、BD-03承認済み | 各判断の適用範囲と、後続へ残した意味論を区別する |
| [言語Binding設計](language-bindings.md) | L1～L4承認済み | Core Propertyを正本とする接続の上位方針を参照する。 |
| 基本設計判断 | §2に列挙する各BD文書 | 判断ごとの状態欄・承認記録を確認し、承認済み範囲のみを詳細設計の前提とする。 |
| [Runtime shutdown dispatch判断 BD-36](BD-36_runtime-shutdown-dispatch-upper-design.md) | BD-36-01/02/04は利用者確認済み、BD-36-03は実装提案・未確認 | 個別判断の状態を区別して参照する。BD-36全体を承認済みとして扱わない |

ADRと下位文書が矛盾した場合はAcceptedのADRを優先し、必要な変更は上位文書のレビューに戻す。BDで採用した上位境界を、詳細設計の都合だけで変更しない。

## 2. 基本設計判断の索引

BDは基本設計（Basic Design）に係る判断の識別子である。各文書の詳細はリンク先を参照する。

| 判断 | 承認済みの基本契約 |
|---|---|
| [BD-01](BD-01_value-types-upper-design.md) | 初期Property値型をBOOL / I64 / UTF8 / F64 / 独自8-byte DECIMALとする。カスタム値、一般nullable値、OBJECTは後続。Collectionは別Core機能 |
| [BD-02](BD-02_ui-transaction-upper-design.md)、[BD-03](BD-03_runtime-notification-upper-design.md) | 初期版は明示UI transactionと複数setter自動rollbackなし。通知配信には有限作業枠、yield、保留、非収束連鎖の停止・診断を設ける |
| [BD-04](BD-04_ui-tree-lifetime-upper-design.md)～[BD-06](BD-06_layout-upper-design.md) | Core objectとPlatform peerの寿命を分け、Logical/Visual Treeを区別し、Layout計算をCore Visual Tree側に置く |
| [BD-07](BD-07_input-event-boundary-upper-design.md)～[BD-10](BD-10_ime-platform-boundary-upper-design.md) | Platform input/IME境界、Runtime focus、Focus移動時のcomposition保持、native IME sessionとCore編集値の責務を定める |
| [BD-11](BD-11_initial-control-contract-upper-design.md)～[BD-13](BD-13_accessibility-upper-design.md) | 初期ControlのWidget/Container責務、独自描画と対応順、共通Accessibility意味情報を定める |
| [BD-14](BD-14_python-wrapper-upper-design.md)、[BD-15](BD-15_python-lint-mcp-upper-design.md) | Pythonはctypesを使う。共通diagnosticsとPython lint/type check・MCP/CLI/CI連携を採る |
| [BD-16](BD-16_build-ci-target-matrix-upper-design.md)～[BD-22](BD-22_support-lifecycle-upper-design.md) | 初期CI target、各言語のpackage境界、Apache-2.0、support lifecycleを定める |
| [BD-23](BD-23_text-shaping-upper-design.md)、[BD-24](BD-24_ime-edit-session-upper-design.md) | 初版のLatin/Japanese text範囲とtarget別shaping管理、IME edit-session/FocusLost/cancel順を定める |
| [BD-25](BD-25_async-validation-upper-design.md)、[BD-26](BD-26_async-command-progress-upper-design.md) | 初版validatorは短時間同期。長時間検証はAsyncCommand＋状態Property。進捗集約と終端結果の独立適用を定める |
| [BD-27](BD-27_error-object-upper-design.md)、[BD-28](BD-28_abi-release-version-upper-design.md) | machine-readable errorと説明message/cause/diagnosticを区別し、runtime MAJOR.MINOR.PATCHとABI symbol majorの対応を定める |
| [BD-29](BD-29_artifact-provenance-upper-design.md)～[BD-32](BD-32_artifact-manifest-schema-upper-design.md) | immutable artifact identity、署名・digest検証、公式catalog、trust root、manifest schema evolutionを定める |
| [BD-33](BD-33_collection-grid-upper-design.md)～[BD-35](BD-35_collection-change-batch-upper-design.md) | Collection/Selectionを独立Core objectとし、stable ID・change set・virtualization・selection意味論・一つのCollection batchの原子性を定める |
| [BD-36](BD-36_runtime-shutdown-dispatch-upper-design.md) | callback/event中のshutdown要求と処理完了後の優先実行、UI thread上でのshutdown本体実行、呼出しモードごとの待機境界を定める。BD-36-03は未確認の実装提案 |

## 3. 要求と基本設計の対応

本節はDD全体で参照する上位根拠を示す。個別の詳細設計では、対象となるRQ IDとBD判断IDを各文書の出典欄に記載し、当該要求を実現する責務・データ/所有フロー・状態・interfaceを具体化する。本表は要求と設計の網羅性を判定する要求test対応表ではない。

| 設計対象の範囲 | 要求根拠 | 基本設計根拠 |
|---|---|---|
| 共通アーキテクチャ・Rust Core・4言語共通機能 | [RQ-01-001～002](../../11_requirements/RQ-01_purpose-architecture.md)、[RQ-03](../../11_requirements/RQ-03_language-support.md) | Accepted ADR-0001、設計骨格、L1～L4 |
| Object・Property・Binding・Command・Collection等のCore機能 | [RQ-02-001～014](../../11_requirements/RQ-02_core-features.md)、Collection/Selectionは[RQ-09-019～020](../../11_requirements/RQ-09_ui-scope.md) | P1～P4、B1～B4、C1～C4、D1～D4、V1～V8、BD-01～03、BD-25～27、BD-33～35 |
| ABI型、handle、参照、callback、threadとdispatch | [RQ-04-001～017](../../11_requirements/RQ-04_abi-boundary.md)、[RQ-05-001～009](../../11_requirements/RQ-05_ownership-callbacks.md)、[RQ-06-001～004](../../11_requirements/RQ-06_threading-dispatch.md) | R1～R4、A1～A32、BD-01、BD-27～28、BD-36（各判断の状態は§2参照） |
| Bindingの種類・方向・失敗・解除 | [RQ-02-003](../../11_requirements/RQ-02_core-features.md)、[RQ-07](../../11_requirements/RQ-07_binding.md) | P1～P4、B1～B4、D1～D4、V1～V8、A13～A20 |
| UI Control・Layout・Input・Focus・IME・Accessibility | [RQ-02-007～011](../../11_requirements/RQ-02_core-features.md)、[RQ-09-001～011](../../11_requirements/RQ-09_ui-scope.md)、[RQ-10-001～010](../../11_requirements/RQ-10_accessibility.md) | BD-04～13、BD-23～24、A25～A32 |
| Platform境界・Build・配布・非機能測定と工程 | [RQ-11](../../11_requirements/RQ-11_platform.md)、[RQ-12](../../11_requirements/RQ-12_build-distribution.md)、[RQ-13](../../11_requirements/RQ-13_nonfunctional-measurement.md)、[RQ-14](../../11_requirements/RQ-14_delivery-sequence.md) | BD-12、BD-14～24、BD-28～32 |

要求の例外・保証・利用者に見える結果はRQ本文を正とする。設計で未決事項を見つけた場合は、要求範囲の判断を設計内で追加せず、該当する上位文書の変更として扱う。設計文書の参照はRQ・BD・ADRなど上位根拠へ向け、要求testや他の下流成果物へのリンクを設けない。

## 4. 上位判断で未確定の詳細境界

以下は上位文書の承認範囲から詳細方式を一意に定められない事項である。表は未決の層と根拠を示し、実現方式や担当DDフェーズをここで決めない。

| 領域 | 未確定の内容 | 上位根拠 | 未決の層 |
|---|---|---|---|
| 値型 | F64/DECIMALの具体ABI、decimal encoding、operator/converter、カスタム値・nullable・OBJECTの将来契約 | RQ-02-002、RQ-04-001～003、BD-01 | 詳細方式の具体化待ち。要求範囲を変更する場合はRQへ戻す |
| Runtime・通知・shutdown | 作業枠の数値、非収束判定詳細、公平性、shutdown完了観測/API表現、UI loop停止時の同期worker動作 | RQ-02-013～014、RQ-05-003/009、RQ-06-001～002、BD-03、BD-36-03/04 | BD-36-03は実装提案・未確認。その他は該当上位契約を維持した詳細化待ち |
| Tree・Control・Layout・Input | 個別操作API、Control別Property/event、template、単位/scale、key routing細則 | RQ-02-007～011、RQ-09-001～010、BD-04～11 | 詳細方式の具体化待ち |
| Text・IME・Renderer・Accessibility | font/corpus、caret/selection/Undo/Redo、OS別IME接続、描画backend API、OS Accessibility写像 | RQ-09-005、RQ-10-001～010、RQ-11-008～013、BD-12～13、BD-23～24 | 詳細方式の具体化待ち |
| Collection・Selection | change set ABI/revision、callback順、reentrancy、既定navigation、virtualization閾値 | RQ-02-005～006、RQ-09-019～022、BD-33～35 | 詳細方式の具体化待ち |
| Error・Async | 全error code catalog、diagnostic payload、progress型・頻度閾値、cause保持の表現 | RQ-02-004/012、RQ-04-006～007、RQ-06-004、BD-25～27 | 詳細方式の具体化待ち |
| Tooling・distribution | compiler/sysroot/RID/TFM/wheel matrix、artifact catalog/crypto/fetch/cache、Python checker/MCP schema | RQ-11～12、BD-15～22、BD-28～32 | 詳細方式の具体化待ち |

## 5. 承認範囲

2026-09-27の初回承認は、当時の本書に記載した上位出典・判断対応と未決事項の整理を対象とする。2026-10-15の改訂では、現行RQへの対応、BD-36の判断状態、参照方向、文書ロールを整理した。どちらの承認も、未決事項の採用、具体API/実装方式、個別要求testや下流DD文書を承認するものではない。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-09-27 | DD-00 初回版 | 利用者 | 上位出典・判断対応と未決事項の整理を承認。詳細設計判断や実装方式の承認は含まない |
| 2026-10-15 | DD-00 文書版0.2 / D00-RQ-01、D00-BD-01、D00-REF-01、D00-TEST-01、D00-GAP-01、D00-STATUS-01 | 利用者 | 現行RQ対応、BD-36の個別状態、上流参照への整理、作業計画との分離を承認 |

## 文書改訂記録

| 日付 | 変更 | 承認状態 |
|---|---|---|
| 2026-10-14 | DD-23再分類後の参照・フェーズ範囲更新案 | 編集案を作成。Requirements Test Manifest 0.2承認後、2026-10-15のDD-00 v0.2承認に含めて反映 |
| 2026-10-15 | 現行RQ対応、BD-36、参照方向・作業計画との分離を反映 | 利用者承認済み。DD-00文書版0.2 |
