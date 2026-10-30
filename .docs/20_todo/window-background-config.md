# Window背景色の規定値・利用者設定

- 計画版: 0.1
- 状態: 計画0.1および要求・BD/DD・全wrapper・requirements test・unit test全文承認済み（利用者、2026-10-30）。Core/C ABI `UNIT-CORE-079` local完了、wrapper実装待ち
- 作成日: 2026-10-30
- 対象version: `0.0.1`
- 対象branch案: `change/window-background-config` → `impl-rust/core-apis-window-background` → 各言語wrapper branch
- 依存する作業: `impl-rust/core-scene-producer` branch上の`.docs/20_todo/core-scene-production.md`および承認済みUI Tree/Layout DD 1.4・Platform DD 1.6・UI unit-test 0.8。これらは本change branchのbaseには未統合。

## 目的

Window背景色について、利用者が値を指定でき、未指定なら製品側の規定値になるという外部契約を要求・設計・全言語wrapperへ追加する。Coreからrendererへ値を明示して渡し、renderer固有の暗黙色に依存しない。

## 提案する動作

- 設定単位はWindowごと。値を一度も指定していない場合は規定値を使用し、明示的に指定した場合はその色を使用する。指定解除操作が成功すると規定値へ戻る。
- 初期色は不透明な白 `RGB(1.0, 1.0, 1.0)`（`#FFFFFF`）を提案する。値の色空間はscene契約に合わせた正規化RGB各成分0.0～1.0、alpha/gradient/子Controlの色設定は本計画の対象外とする。
- NaN/Infinity/範囲外成分は拒否し、既存値・scene/frame状態を変更しない。設定変更はLayoutではなくpaint/sceneをdirtyにし、次回のscene生成・frame提出へ反映する。
- 規定値はCore側で解決して`SceneSnapshot`へ必ず実値を格納する。rendererのOS/GPU依存defaultへ委ねない。

## 既存要求・設計との差分

- 現行RQ-09にはWindow背景色の指定・未指定時の結果がない。RQ-09-073は次段階Style/Theme/Resource一般を扱うため、初期版のWindow背景色一項目を独立要求として追加し、Style/Theme全体を前倒ししない案とする。採番候補は次の未使用ID `RQ-09-074`。
- 承認済みPlatform DD 1.4はrendererに背景色を推定させない一方、「未指定時の暗黙色を持たない」とする。Coreが未指定値を規定色へ解決する要求を加え、DD/UTを整合させる必要がある。
- 公開ABI・全wrapperにWindow背景色のread/set/resetをどう表現するかは未設計。既存のWindow/property APIとC ABI型集合を照合し、公開surfaceを増やす場合はRQ/ABI/DD/testを同時にレビューする。

## 作業案

1. この計画の外部動作・初期色案をレビュー・承認する。特に初期色を`#FFFFFF`とするかを確定する。
2. RQ-09-074案を作り、RQ-09本文・承認履歴、要求索引の件数、必要なrequirements test caseを同じ要求改訂として更新する。RQ-09-073の次段階範囲を黙って変更しない。
3. UI Control、Platform、ABIおよび各wrapper DDを文書単位で更新する。未指定/指定/解除、component validation、Window lifetime、dirty propagation、scene renderingまでの責務を記述する。API名・signatureは既存ABIと各wrapper設計を確認して提案する。
4. Unit-test設計を更新し、default / explicit set / reset / invalid input atomicity / paint-only invalidation / Window reopen・renderer forwardingを実装unitに対応させる。要求test matrixとのcase対応も更新する。
5. 設計とtest文書全体の利用者承認後、承認済みchange branchの下にRust Core/C ABI branchを作り、unitごとに実装する。言語が変わるたびにCore branchから各wrapper branchを切り、全wrapperで未指定default・set/reset semanticsを揃える。
6. 各unitを同一revisionで検証し、requirements/implementation-test ledgerと実行記録を更新する。Linux Docker結果はprovisional、実機確認は別扱いとする。

## 対象外

- Global Theme/Style/Resource、Window以外のControl背景、alpha blending、gradient、visual state別palette。
- renderer別の自動fallback、GPU/OSが返す暗黙default色。
- 未承認のAPI signatureやC ABI feature declaration。
- Platform Adapter実機hardware smoke。

## 承認境界

- 計画0.1は利用者承認済み。提案default `#FFFFFF` と要求ID候補 `RQ-09-074` を用いてrequirements/design/test改訂案を作成する。API signatureは既存ABIとwrapper設計の照合後に提案し、未承認のまま確定仕様にしない。
- 要求・設計・test文書は全体レビュー・承認後に実装へ進む。計画承認・要求承認・設計承認・実装開始・unit完了・commit/pushを別状態として記録する。
- 計画0.1は承認後にcommit/pushする。後続の新規・改訂計画は利用者レビュー前にcommit/pushしない。実装unitのcommitは承認済み実装計画とrepository Git規則に従う。

## 現在の進捗

- 要求: `RQ-09-074`をRQ-09 0.6改訂案へ追加。承認済み280件の承認状態は維持し、今回案は未承認として索引件数に混ぜない。
- 設計: BD UI Tree/Layout/Control 1.1、Platform abstraction 1.1、DD UI controls 1.2、ABI 1.4、C++ 1.4、C# 1.5、Python 1.2、Rust 1.7をレビュー案として更新。
- Test: requirement-test matrix 0.8 / assertions、および UI unit-test 0.4に候補caseを追加。未実装・未実行。
- Core/C ABI: `impl-rust/core-apis-window-background` にて `UNIT-CORE-079` を実装。WindowLifecycleがWindowごとのoverrideを保持し、規定値white、reset、finite/range validation、paint full-damage強制、SceneSnapshotへの反映を行う。C ABI get/set/reset export・header・atomic getter testも追加した。
- local検証: Core/ABI focused tests、workspace全target test、workspace Clippy、Rust 1.85 check、fmt、`.venv/Scripts/python.exe tools/dev.py check`、docs link check、manifest JSON、`git diff --check`がすべてpass。Linux Platform Adapter実機確認と同revision CIは未実施。
- commit/push: 実装unitとしてcommit可能。pushは依頼されていないため未実施。
- dependency integration: `impl-rust/core-scene-producer` を親へmerge commit `5cb737d` で統合。別エージェントの `impl-rust/ui-webview-plugin` は統合対象外で、変更も含めていない。
- wrapper branch: Core API unitの全検証・commit完了後、各対象言語ごとにこのCore branchから独立して作成する。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-10-30 | 計画0.1 | review draft | 未指定時に規定値、Windowごとの色指定、解除時の規定値復帰を追加する変更範囲を提案。初期色`#FFFFFF`はレビュー待ち。 |
| 2026-10-30 | 計画0.1 | 利用者承認 | 「ok」により、規定値`#FFFFFF`案を含むWindow背景色設定の要求・設計・test改訂案作成を承認。実装は設計全文承認後。 |
| 2026-10-30 | 要求・設計・test各改訂案 | review draft | RQ-09-074、各層の背景色状態/API/scene flow、test traceabilityを案として作成。各文書全体のレビュー・承認待ち。実装・test未開始。 |
| 2026-10-30 | 要求・設計・test各全文 | 利用者承認 | 「ok」により更新対象文書全体を承認。実装・試験状態とは区別し、Core/C ABI実装へ進む。 |
