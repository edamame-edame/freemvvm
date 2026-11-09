# RQ-11: Platformと組み込みLinux

- 文書版: 0.6
- 状態: 0.6全文承認済み（利用者、2026-11-09）。実装・試験状態は別管理
- 更新日: 2026-11-09
- 入口: [要求仕様索引](reqirements-specification.md)

## 概要

対象platform、Rust Coreから分離するplatform依存機能、組み込みLinux backendの検証順を定める。target向けbuild成功、target上でのruntime確認、特定board/profileでの確認は別々の結果として扱う。

本章は提供機能・利用条件・外部から観測できる結果を定める。各IDの対象段階を区別し、将来候補の列挙を採用承認とはしない。

## 要求本文

| 要求ID | 項目 | 要求詳細 | 対象段階 |
| --- | --- | --- | --- |
| <a id="rq-11-001"></a>RQ-11-001 | Linux ARMv7 | Linux ARMv7向けtargetを保持する。0.0.1ではframebuffer経由の描画に限定した実験的経路とcross-build/linkを対象候補とする。代表機器profileの実行確認がない間は、ARMv7機器の正式runtime supportを表示しない。 | 0.0.1限定候補 / hardware確認は後続 |
| <a id="rq-11-002"></a>RQ-11-002 | Linux AArch64 | Linux AArch64向けtargetを保持する。0.0.1ではframebuffer経由の描画に限定した実験的経路とcross-build/linkを対象候補とする。代表機器profileの実行確認がない間は、AArch64機器の正式runtime supportを表示しない。 | 0.0.1限定候補 / hardware確認は後続 |
| <a id="rq-11-003"></a>RQ-11-003 | Linux x64 | Linux x64を開発・CI targetとする。0.0.1のruntime profileはUbuntu 24.04 x64と、対象hypervisor/virtual GPUを特定して描画確認に合格したDebian 12 x64 VM profileとする。OS起動やbuild成功だけではruntime profile passとしない。 | 0.0.1候補 |
| <a id="rq-11-004"></a>RQ-11-004 | Windows x64 | Windows x64を開発・CI targetとし、Windows 11 x64で配布artifactを用いたUI縦断と可視描画を確認する。 | 0.0.1候補 |
| <a id="rq-11-005"></a>RQ-11-005 | Windows ARM64 | Windows ARM64を後続target候補として保持する。初期CI matrixへの追加や正式supportは本要求だけでは確定しない。 | 現行 |
| <a id="rq-11-006"></a>RQ-11-006 | macOS x64 / ARM64 | macOS x64およびARM64を後続target候補として保持する。初期CI matrixへの追加や正式supportは本要求だけでは確定しない。 | 現行 |
| <a id="rq-11-007"></a>RQ-11-007 | bare metal / MCU | bare metalおよびMCUは通常のLinux targetと同一視せず、独立したplatform profile候補として扱う。 | 現行 |
| <a id="rq-11-008"></a>RQ-11-008 | renderer | Controlの状態と意味は共通Coreを正本とし、特定OSの描画APIやnative control状態を正本としないこと。描画backendを交換してもCoreのControl意味を保てる構成とすること。 | 現行 |
| <a id="rq-11-009"></a>RQ-11-009 | platform backend | Window/surface、OS event loop、device接続等のplatform backendをRust Coreから分離する。 | 現行 |
| <a id="rq-11-010"></a>RQ-11-010 | text engine | text engineをRust Coreから分離する。Coreの文字列・Text意味論を特定OSのtext engine型に依存させない。 | 現行 |
| <a id="rq-11-011"></a>RQ-11-011 | image codec | image codecをRust Coreから分離する。Coreの共通UI機能が特定codec実装に直接依存しない構成とする。 | 現行 |
| <a id="rq-11-012"></a>RQ-11-012 | input backend | OS/device由来の入力backendをRust Coreから分離する。入力は共通処理へ渡す境界を持つ。 | 現行 |
| <a id="rq-11-013"></a>RQ-11-013 | accessibility backend | Accessibility backendをRust Coreから分離する。Core意味情報をOS固有Accessibility機構へ接続する責務をbackendに置く。 | 現行 |
| <a id="rq-11-014"></a>RQ-11-014 | 初期runtime profile backend | Windows 11 x64と選定Linux x64 profileについて、profileごとの描画backendと依存条件を識別し、対象profile上でruntime描画を検証する。 | 0.0.1候補 |
| <a id="rq-11-015"></a>RQ-11-015 | Linux window-system backend候補 | WaylandをLinux desktop向け後続backend候補として保持し、0.0.1正式profileとして表示する場合は独立したprofile evidenceを必要とする。 | 後続候補 |
| <a id="rq-11-016"></a>RQ-11-016 | Linux window-system backend候補 | X11をLinux desktop向け後続backend候補として保持し、0.0.1正式profileとして表示する場合は独立したprofile evidenceを必要とする。 | 後続候補 |
| <a id="rq-11-017"></a>RQ-11-017 | Linux framebuffer限定backend | 追加Linux target向けにframebuffer経由の描画だけを提供する限定経路を0.0.1対象候補とする。対応profile、pixel format、表示初期化、failure境界を明示し、実機未確認targetをruntime supportedと表示しない。通常のWindow管理、input、IME、Accessibilityを暗黙に含めない。 | 0.0.1限定候補 |
| <a id="rq-11-018"></a>RQ-11-018 | software renderer | software rendererはheadless test等の明示profile候補として扱い、他backendからの暗黙fallbackを行わない。 | 後続候補 |
| <a id="rq-11-019"></a>RQ-11-019 | 実機依存機能の隔離 | GPU、input device、IME、display rotation、power management、window systemなどの実機依存要素はPlatform Adapterに閉じ込める。Coreの共通意味論や共通状態が特定実機・driverの表現を正本として持たないこと。 | 現行 |
| <a id="rq-11-020"></a>RQ-11-020 | Platform資源喪失と終了 | 表示資源の喪失でCoreの確定済みUI状態を失わず、失敗した描画を成功として扱わないこと。OSのWindow close要求だけでRuntime全体を暗黙終了しないこと。 | 初期共通基盤 |
| <a id="rq-11-021"></a>RQ-11-021 | 検証状態と正式support | build、VM/OS runtime、physical hardware profileの確認状態を別に公表すること。VMでの描画確認を物理GPU/boardの確認や他profileのruntime supportへ拡大しないこと。 | 初期共通基盤 |
| <a id="rq-11-022"></a>RQ-11-022 | 描画backendとfallback | 各runtime profileの描画backendと依存条件を公表すること。device/surface喪失後の復旧状態を識別でき、未指定のsoftware/framebuffer backendへ暗黙に切り替えないこと。 | 初期共通基盤 |
| <a id="rq-11-023"></a>RQ-11-023 | font・IME・Accessibility profile | Windows 11、Ubuntu 24.04、Debian 12 VM、framebuffer限定の各profileについてfont/文字対応、日本語IME条件、Accessibility provider有無を識別すること。framebuffer描画のみのprofileをIME/Accessibility対応済みと表示しないこと。 | 初期共通基盤 |
| <a id="rq-11-024"></a>RQ-11-024 | 0.0.1 support profile | 正式runtime profile候補をWindows 11 x64とUbuntu 24.04 x64とする。Debian 12 x64は特定VM/virtual GPU構成で実描画まで確認したprofileに限り追加できる。Linux ARMv7/AArch64はframebuffer描画限定の実験的経路・cross-buildを対象候補とし、代表boardでruntime確認されるまでは正式hardware supportと表示しない。 | 0.0.1候補 |


## 適用境界

承認は要求本文の範囲で完結する。API名・内部データ構造・処理方式の確定を前提にしない。既存ADRによるRust Core・versioned C ABI等の明示的な構成制約は維持する。

## 承認履歴

| 日付 | 対象版・ID / 範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-09-26 | 旧要求仕様の該当範囲 | 利用者 | Acceptedを継承。後日追記の承認日・未決範囲は旧版で確認 |
| 2026-10-12 | 要求承認管理・章分割・RQ ID化 | 利用者 | 今回の指示による管理方針。旧本文に記載済みの承認事項を要求側で管理する |
| 2026-10-12 | RQ-11全掲載ID / 再編集版0.3 | 利用者 | 要求詳細と受入確認を承認。後続targetの正式採用は本文の記載を維持 |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 旧版 | 2026-09-25 | 旧要求仕様を作成。2026-09-26にAccepted | 旧文書参照 | 旧要求仕様の承認記録 |
| 0.1 | 2026-10-12 | 章分割、RQ ID、受入確認案・設計/testリンク、要求承認履歴の集約 | Codex | 管理方針の指示に基づく再編集。版全体のレビューは未完了 |
| 0.2 | 2026-10-12 | 下流参照リンクを除去し、参照方向を明記 | Codex | 要求本文・承認状態は変更なし |
| 0.3 | 2026-10-12 | support profileの区別、分離境界、backend検証順を要求ごとに整理 | Codex | 再編集版を利用者が同日承認。後続targetの採用は別途承認 |

## 今回の承認履歴

| 日付 | 対象版・範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.4 / 全レビュー案の要求補足・意味単位移管 | 利用者 | 全レビュー案の反映指示により承認。実装・試験実行の完了とは区別する |

## 今回の改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 |
| --- | --- | --- | --- |
| 0.4 | 2026-10-17 | 要求と実現手段・検証・作業記録を分離し、正式IDと参照を対応付けた | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.5 / 要求変更 RQ-11-022, RQ-11-023, RQ-11-024 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.5 | 2026-10-17 | 要求変更 RQ-11-022, RQ-11-023, RQ-11-024を正本へ適用し、旧判断との対応を保持 | Codex |

## 0.6 承認履歴

| 日付 | 対象版・範囲 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-09 | 0.6全文 / RQ-11-001〜004、014〜018、021〜024 | 利用者 | 「ok」によりWindows 11 x64・Ubuntu 24.04 x64を対象とし、描画確認に合格したDebian 12 VMをprofile限定で加える方針、ARM実機gateの後続分離、その他Linuxのframebuffer限定範囲を含む本文全体を承認。Debian VMの実行証拠・実装・release合格は別管理 |

## 改訂履歴（0.6）

| 版 | 日付 | 変更内容 | 作成者 | 状態 |
|---|---|---|---|---|
| 0.6 | 2026-11-09 | 0.0.1 profileとframebuffer限定Linux範囲の見直し | Codex | 本文全体を利用者承認済み。実装・試験は別管理 |
