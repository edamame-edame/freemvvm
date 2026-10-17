# RQ-11: Platformと組み込みLinux

- 文書版: 0.5
- 状態: 承認済み（利用者、2026-10-17）
- 更新日: 2026-10-17
- 入口: [要求仕様索引](reqirements-specification.md)

## 概要

対象platform、Rust Coreから分離するplatform依存機能、組み込みLinux backendの検証順を定める。target向けbuild成功、target上でのruntime確認、特定board/profileでの確認は別々の結果として扱う。

本章は提供機能・利用条件・外部から観測できる結果を定める。各IDの対象段階を区別し、将来候補の列挙を採用承認とはしない。

## 要求本文

| 要求ID | 項目 | 要求詳細 | 対象段階 |
| --- | --- | --- | --- |
| <a id="rq-11-001"></a>RQ-11-001 | Linux ARMv7 | 組み込みLinuxを製品の正式対象に含め、Linux ARMv7向けtargetを用意する。正式support範囲はABI majorとtarget profileの組で示す。 | 現行 |
| <a id="rq-11-002"></a>RQ-11-002 | Linux AArch64 | 組み込みLinuxを製品の正式対象に含め、Linux AArch64向けtargetを用意する。正式support範囲はABI majorとtarget profileの組で示す。 | 現行 |
| <a id="rq-11-003"></a>RQ-11-003 | Linux x64 | Linux x64を初期開発・CI targetに含め、native build/testとARM cross-buildのhostとして利用する。 | 現行 |
| <a id="rq-11-004"></a>RQ-11-004 | Windows x64 | Windows x64を初期開発・CI targetに含め、native buildと初期UI縦断確認を行えるようにする。 | 現行 |
| <a id="rq-11-005"></a>RQ-11-005 | Windows ARM64 | Windows ARM64を後続target候補として保持する。初期CI matrixへの追加や正式supportは本要求だけでは確定しない。 | 現行 |
| <a id="rq-11-006"></a>RQ-11-006 | macOS x64 / ARM64 | macOS x64およびARM64を後続target候補として保持する。初期CI matrixへの追加や正式supportは本要求だけでは確定しない。 | 現行 |
| <a id="rq-11-007"></a>RQ-11-007 | bare metal / MCU | bare metalおよびMCUは通常のLinux targetと同一視せず、独立したplatform profile候補として扱う。 | 現行 |
| <a id="rq-11-008"></a>RQ-11-008 | renderer | Controlの状態と意味は共通Coreを正本とし、特定OSの描画APIやnative control状態を正本としないこと。描画backendを交換してもCoreのControl意味を保てる構成とすること。 | 現行 |
| <a id="rq-11-009"></a>RQ-11-009 | platform backend | Window/surface、OS event loop、device接続等のplatform backendをRust Coreから分離する。 | 現行 |
| <a id="rq-11-010"></a>RQ-11-010 | text engine | text engineをRust Coreから分離する。Coreの文字列・Text意味論を特定OSのtext engine型に依存させない。 | 現行 |
| <a id="rq-11-011"></a>RQ-11-011 | image codec | image codecをRust Coreから分離する。Coreの共通UI機能が特定codec実装に直接依存しない構成とする。 | 現行 |
| <a id="rq-11-012"></a>RQ-11-012 | input backend | OS/device由来の入力backendをRust Coreから分離する。入力は共通処理へ渡す境界を持つ。 | 現行 |
| <a id="rq-11-013"></a>RQ-11-013 | accessibility backend | Accessibility backendをRust Coreから分離する。Core意味情報をOS固有Accessibility機構へ接続する責務をbackendに置く。 | 現行 |
| <a id="rq-11-014"></a>RQ-11-014 | 描画backendの検討順 | DRM/KMS + EGL/OpenGL ESを最初に検証する。 | 現行 |
| <a id="rq-11-015"></a>RQ-11-015 | 描画backendの検討順 | Waylandを第2候補として検証する。 | 現行 |
| <a id="rq-11-016"></a>RQ-11-016 | 描画backendの検討順 | X11を第3候補として検証する。 | 現行 |
| <a id="rq-11-017"></a>RQ-11-017 | 描画backendの検討順 | framebufferを第4候補として検証する。 | 現行 |
| <a id="rq-11-018"></a>RQ-11-018 | 描画backendの検討順 | software rendererを第5候補として検証する。 | 現行 |
| <a id="rq-11-019"></a>RQ-11-019 | 実機依存機能の隔離 | GPU、input device、IME、display rotation、power management、window systemなどの実機依存要素はPlatform Adapterに閉じ込める。Coreの共通意味論や共通状態が特定実機・driverの表現を正本として持たないこと。 | 現行 |
| <a id="rq-11-020"></a>RQ-11-020 | Platform資源喪失と終了 | 表示資源の喪失でCoreの確定済みUI状態を失わず、失敗した描画を成功として扱わないこと。OSのWindow close要求だけでRuntime全体を暗黙終了しないこと。 | 初期共通基盤 |
| <a id="rq-11-021"></a>RQ-11-021 | 検証状態と正式support | build成功、Runtime動作確認、hardware profile確認の範囲を区別して公表すること。未検証profileや他architectureへ実機supportを拡大表示しないこと。 | 初期共通基盤 |
| <a id="rq-11-022"></a>RQ-11-022 | 描画backendとfallback | 初期profileの描画backendと依存条件を公表すること。device喪失後の復旧可否を識別でき、software rendererへの暗黙切替を行わないこと。 | 初期共通基盤 |
| <a id="rq-11-023"></a>RQ-11-023 | font・IME・Accessibility profile | 各profileのfont/文字対応、日本語IMEの導入条件、Accessibility bus/provider有無を識別できること。desktop LinuxのAT-SPI profileはbusを必要とし、DRM profileはIME brokerを明示すること。 | 初期共通基盤 |
| <a id="rq-11-024"></a>RQ-11-024 | 初期support profile | 初期profileをWindows11 x64、Debian12 x64、ARMv7/AArch64 cross-buildと代表RPi4 AArch64とすること。初期ARMv7はbuild検証表示に限定し、実機未確認を明示すること。 | 初期共通基盤 |


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
