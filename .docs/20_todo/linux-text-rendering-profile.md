# Linux初期Text描画profileの確定

> **旧候補の扱い:** 以下の初期調査・レビュー履歴には、固定font assetや自動fallbackを採らない旧案が残る。未指定fontはPlatformのsystem defaultを使い、fallbackを解決結果とともに記録する方針がRQ-09-075 / Platform DD 1.9で承認されたため、旧案の「OS設定に追従しない」「fallbackしない」は superseded。具体的なLinux profile・依存・fallback順は[実装計画0.3](linux-text-rendering-implementation.md)のレビュー対象であり、まだ未承認。

- 計画版: 0.1
- 状態: 計画0.1/0.3、Platform DD 1.9、UI test 1.2、RQ-09-075全文承認済み。UNIT-CORE-084は`b6c64ea`でcommit/push済み。Linux依存追加・platform実装は未着手
- 作成日: 2026-10-31
- 対象version: `0.0.1`
- 対象branch案: `impl-rust/core-scene-rendering`
- 関連計画: [Core UI Scene生成の次工程](core-scene-production.md)
- 直前完了unit: [Window DeviceScale transport](core-device-scale-transport.md)、UNIT-CORE-083、commit `cf3ce05`

## 監査結果

- 要求[RQ-09-004](../11_requirements/RQ-09_ui-scope.md#rq-09-004)、[RQ-09-006](../11_requirements/RQ-09_ui-scope.md#rq-09-006)、[RQ-09-072](../11_requirements/RQ-09_ui-scope.md#rq-09-072)はText表示・Button label・単一paragraph表示とclient clippingを要求する。soft-wrap、ellipsis、selection/copyは初期必須範囲ではなく、Button labelはAccessibleNameと別値。
- [RQ-11-023](../11_requirements/RQ-11_platform.md#rq-11-023)はprofileごとのfontと文字対応の識別を要求するが、配布font asset、family/version/hash、依存、missing-font時の扱いを選定していない。
- [UI controls DD](../13_detail_design/ui/controls.md)はText/TextBox/Buttonの表示snapshotをTextServiceへ渡し、同じmetricsをLayoutと描画で使う方針を持つ。
- [Platform DD](../13_detail_design/platform/internal-layers.md)は`TextService.shape`、Linux Fontconfig/HarfBuzz/FreeType、Noto系fontを候補として挙げるが、依存・profile asset・GlyphRun colorは未決。`GlyphRunSnapshot`にcolorとfont asset/profile解決契約がなく、Linux GLESもGlyphRunを成功扱いせず拒否する。
- Coreはfake Visual descriptorでScene構造を生成できるが、Text/Button/TextBoxのproduction descriptor/providerは存在しない。固定placeholder、未承認色、未承認fontを足しても要求を満たしたことにはならない。

## 目的

Linux初期profileで要求されたText系Control表示を実装可能にするため、必要最小限の可視appearance、font asset/profile、TextService出力、Layout metrics、GlyphRun rendering/failure境界をDDとtest designで提案する。依存・配布asset・外部から観測できる初期appearanceは、設計文書全体の利用者承認前に確定しない。

## 提案作業

1. Platform DD改訂案を作る。対象はLinux初期profileに限定し、font resolver/shaper/rasterizerの候補、font asset識別/配布、GlyphRunのcolor/asset参照、単一paragraphとclient clip、TextService failure時のLayout/Scene atomicityをひとつの実現契約として記述する。
2. Text/Button/TextBoxで既に確定している要求・Property snapshot・AccessibleName分離を再掲し、新しい利用者向けstyle API、soft-wrap、ellipsis、IME、Accessibility scopeを追加しない。
3. 初期appearanceの選択が必要な箇所（少なくともtext color、font family/profile、font sizeの由来）を具体候補と可視差分つきで提示する。OS theme追従やsystem font fallbackを暗黙の規定値としない。
4. `.docs/32_unit_test/ui.md`へ、font profile/asset lock、同一metricsのMeasure/Scene利用、glyph color/clip、missing/unsupported operation、invalid text/font/profile failureのtest caseを提案する。Docker/CIと実機profile evidenceを分離する。
5. DD/test文書全体を提示して利用者レビュー・承認を受ける。承認後にだけ依存を追加し、実装unitとunit manifestを確定する。

## 実装候補（設計承認後）

- Core: immutable `TextPaintSnapshot`または同等の内部値からTextServiceへ渡す中立契約とmetrics/GlyphRun identityを接続。公開C ABI/wrapper APIは増やさない。
- Linux: 承認されたprofileだけを使うTextService/font realizationとGLES GlyphRun描画を追加。未対応文字、font欠落、shaping/rasterizer/GPU failureは明示失敗とし、空描画やPresented成功に読み替えない。
- Controls: 承認済みappearance/profileからText/Button/TextBox display snapshotを生成し、measureとSceneで同一snapshot/metrics revisionを使う。Controlごとの描画責務は適度な単位にまとめ、個別ファイルやtypeへ過剰分割しない。
- 実機DRM/KMS描画確認はDocker仮passやfake surface suiteと別gateにする。

## 対象外

- Windows DirectWrite実装、renderer選択APIやlanguage wrapper変更。
- ユーザー公開Style/Theme/Font API、soft-wrap/ellipsis/selection/copy、IME/Accessibility実装。
- font auto-fallbackを利用者が識別できない形で行うこと。
- RQにないplaceholder表示を合格扱いすること。

## 承認境界

- 本計画案の承認は、Linux text profileを具体候補と可視影響を含めて設計・test proposalへまとめる許可であり、特定font/color/dependencyの採用や実装開始を意味しない。
- 設計・test文書の全文承認、依存追加、実装開始、unit合格、commit/push、PRは別状態として管理する。
- Rust Core/Linux Platform作業なので`impl-rust/core-scene-rendering`を継続する。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-10-31 | 計画0.1 | 利用者承認 | Linux profile設計・test proposalを先に作り、具体font/color/dependency採用と実装を別承認にする手順を承認 |
| 2026-10-31 | Platform DD 1.8 / UI UT 1.1 | 文書全体レビュー待ち | font source A/B/C、Noto Sans CJK JP候補の表示影響、black/14 DIP案、profile/test failure boundaryを提示 |
| 2026-10-31 | Platform DD 1.8 / UI UT 1.1全文 | 利用者承認 | 「ok」により提案契約・test case全文を承認。font asset/source/digest、color/size、coverage corpusの採用値と実装は別レビュー |

具体profile値およびimplementation unitは[次の実装計画](linux-text-rendering-implementation.md)で別途レビューする。現時点で依存追加・実装は未着手。

利用者が未指定fontにPlatform system defaultを使う方針を追加したため、固定font source候補案をそのまま実装しない。RQ-09-075、Platform DD 1.9、UI UT 1.2、UI Controls DD 1.4および実装計画0.2の改訂案をレビューする。
