# 0.0.1 Windows TextService・DirectWrite font catalog 接続計画

- 計画版: 0.1
- 状態: 対象実装unit完了（2026-11-01、同revision Windows/Linux CI pass）。Control/Scene producer接続とGUI実機表示は別残課題
- 対象version: `0.0.1`
- 対象: Windows profileのTextServiceからDirectWrite font sourceを解決し、生成GlyphRunのfaceを同じdigestでrendererへ登録する内部接続
- 実装branch案: `impl-rust/platform-adapter-windows-text-service`
- 親branch案: `impl-rust/platform-adapter-windows-text`
- branch起点: push済み `0d5d88bac581294a849f5bc9e34a1b3e79bce0be`（`impl-rust/platform-adapter-windows-text`）

## 背景と現在地

Windows DirectWrite rendererは、GlyphRunの`font_digest`から登録済み`IDWriteFontFace`を引いて描画する。明示TTFの読み込みとsystem faceの登録関数も`crates/platform-windows/src/renderer.rs`から呼べるが、現時点では後続のTextService/profile接続用であり、呼び出し元はない。face未登録のGlyphRunはframe failureになる。

承認済みDDには`TextService.shape(text_snapshot, font_profile, width_constraint) -> GlyphRunSnapshot`という責務境界があるが、push済みbranchのCoreにはTextService、text snapshot、font profileの実装型や、UIからGlyphRunを生成する接続がない。したがって、rendererの未使用helperを単に呼ぶだけでは完了しない。まず既存のCore/Platform境界とthread/resource ownerを照合し、承認済みDD内で実装できる内部接続を確定する。新しいCore/C ABI公開面や要求上の保証が必要と判明した場合は、その変更を実装へ混ぜず、該当DD/RQの差分案を提示して承認を待つ。

## 上位根拠と承認境界

- 要求: [UI機能範囲](../11_requirements/RQ-09_ui-scope.md)のRQ-09-053（Latin・日本語UI textとfont依存条件）、RQ-09-054（初期対象外文字）、RQ-09-072（初期表示範囲）、[Platform・target](../11_requirements/RQ-11_platform.md)のRQ-11-023（font/文字対応profile）。
- ADR: [ADR-0001](../04_approval/adr-0001-rust-core.md)。Rust Coreを動作の正本、versioned C ABIを言語境界とし、Platform実装を差替え可能に保つ。
- 基本設計: [プラットフォーム抽象化](../12_design/platform/abstraction.md) 1.1（TextServiceとfont discovery/shaping/renderingの責務境界）。
- 詳細設計: [プラットフォーム内部レイヤー](../13_detail_design/platform/internal-layers.md) 1.7（2026-11-01全文承認済み）。GlyphRunの黒色既定値、OS system font解決、明示TTF source、face digest一致、失敗、backend ownerによるresource解放に従う。
- 試験: [UI単体試験](../32_unit_test/ui.md) 1.1（2026-11-01全文承認済み）の`UT-PLATFORM-WINDOWS-TEXT-001`、[unit-test strategy](../32_unit_test/unit-test-strategy.md)、[implementation-test inventory](../32_unit_test/implementation-test-inventory.md)。

この計画は要求や承認済み設計を変更しない。Windows family名の固定、固定Fonts path走査、OSへのfont install、描画時fallback、未登録faceを成功扱いする挙動は採用しない。TextService/renderer間のCOM object受け渡しやthread apartmentはDDのowner契約に照合し、未決なら実装前に差分を出す。

## 実装範囲

1. DirectWriteのsystem font collection/fallbackまたは明示TTF sourceから、対象textを描画するfaceを解決する。
2. 解決faceを使ってglyph shapingし、glyph ID・placement・size・bounds・承認済み既定色を持つGlyphRunを作る。複数faceに分かれる場合はfaceごとにrunを分ける。
3. shapingで使ったfaceとGlyphRunのdigestを対応付け、Windows rendererのfont catalogが同じfaceをそのdigestで解決できるようにする。rendererでの別face fallbackは行わない。
4. face/file reference/cacheのownerと解放時点を明確にし、renderer close後に一度だけ解放する。失敗途中の生成物も漏らさず解放する。
5. `UT-PLATFORM-WINDOWS-TEXT-001`を用いて、fake font source/resolver・fake DirectWrite boundaryで登録から描画までを検証する。特定OS family名やpixel outputをCI oracleにしない。

実装では、system default familyをWindowsの`NONCLIENTMETRICS`から取得し、user localeをDirectWriteへ渡してsystem collectionのlanguage/script fallbackを使う。明示TTFはDirectWrite font set/custom collectionへ追加する。GlyphRun digestは選ばれたfaceのfont file bytesとface indexからSHA-256で生成し、renderer所有workerのcatalogへ登録する。OS family/pixelを判定するnative headless smokeは手動専用で、CI unit oracleには含めない。

## 実装unit案

| Unit | 責務 | 対応試験と完了条件 |
|---|---|---|
| `UNIT-PLATFORM-WINDOWS-004` | Windows DirectWrite TextService/profile resolverとRenderer font catalogを内部接続する。system default・明示TTF sourceからfaceを選び、同じface digestでGlyphRun生成と描画を行う。公開ABI追加やLinux変更を含めない | `UT-PLATFORM-WINDOWS-TEXT-001`を実装し、system default/TTF source、複数face分割、digest一致・不一致、source欠落・不正、shape/render failure、renderer close時の資源解放を検証。Windows target build、Rust minimum check、既存workspace gateが同一revisionで成功 |

試験名・単位名は既存の承認済みunit/test管理規約に従う。実装開始時に`.docs/32_unit_test/implementation-test-inventory.md`と`tools/units.yaml`へ実際のsource/test path、command、依存pathを登録し、既存UTのoracleに不足があれば試験設計を改訂してから実装する。

## 段階

### 1. 接続境界の事前確認

- push済みbranchの実ソース、Core text/layout seam、DirectWrite resource owner、worker threadとCOM使用条件を照合する。
- `GlyphRunSnapshot`生成責務、profileからのfont source入力、face catalogへの登録とscene submitの順序を決め、既承認DDのクラス/ownership契約内に納める。
- 承認済みDDにないAPI/外部保証やCore変更が必要な場合は、実装を止めてDD/RQ差分をレビューに出す。

### 2. TextServiceとfont catalogの接続

- system default時はDirectWrite system font collectionとOS language/script fallbackを使い、選ばれた実faceをshapingとrenderingの両方に使う。
- 明示TTFはDirectWrite file reference/custom collectionからロードする。OS Fonts directoryの走査やfont installationをしない。
- source failure、face registration failure、digest mismatchはshape/frame failureとして伝え、Presented成功にしない。
- COM/native resourceの所有threadとclose順序をDDに合わせ、途中失敗時を含む一回解放を実装する。

### 3. 同revision検証と記録

- focused UT、Windows target build、minimum Rust toolchain、Clippy/format/workspace checksを実行する。
- 可視WindowやOS font pixel差をCI成功条件にしない。system fontの実選択・表示品質は後続のWindows実機profile smokeとして未完了記録に残す。
- unit-test inventoryと`tools/units.yaml`へ実行結果と対象SHA/CI evidenceを記録し、unit完了後に日本語commit・pushする。

## ブランチ・作業管理

- 計画承認後、最新のpush済み`impl-rust/platform-adapter-windows-text`から`impl-rust/platform-adapter-windows-text-service`を作る。
- 現在の計画用worktreeは`112b65e`でdetached、実装branchより古い。実装・commit対象にせず、承認後に上記branchへ切り替えた新しいworktreeで作業する。
- 変更がCore API/C ABIに広がると判明した場合は、ユーザー指示の層順に従いCore APIs branchを親にして切り直し、その上にWindows platform branchを作る。wrapper変更は本計画に含めない。
- 対応unitの試験合格後に日本語commit・pushする。関連TODOが完了するまでmerge PRは作らない。

## 対象外・残課題

- Core Control treeからtext snapshotを作り、layout/Scene producerへGlyphRunを供給してWindowに表示する縦断接続。Core側にTextService入力/Control表示生成の実装がないため別計画で扱う。
- TextBox編集、IME/TSF、Accessibility、文字編集位置の接続。
- Windows GUI実機でのsystem font選択・実描画品質確認。CI passで代替しない。
- Linux Fontconfig/HarfBuzz/FreeType、別OS、GPU backend、公開C ABI・言語wrapper、font package配布。
- 設計未決事項や外部契約を実装都合だけで決めること。

## 完了条件

- Windows TextServiceが、system defaultと明示TTFの双方で、描画に使うfaceとGlyphRun digestを一貫して生成・登録する。
- rendererは登録faceのみでGlyphRunを描き、欠損・不一致・不正glyph・backend失敗をPresented成功にしない。
- face/font file/cacheが合意したownerに保持され、normal closeと途中failureで漏れ・二重解放がないことをunit-testで確認する。
- 対象unitのWindows CI/target checksが同revisionで成功し、inventoryにSHA・command・結果を記録する。
- Core Control/Scene producer接続およびGUI実機表示は別残課題として識別された状態である。

## 実装・検証状態

- `UNIT-PLATFORM-WINDOWS-004`は実装・local検証完了。Windows crate focused tests 3件、Windows crate 15 passed / 2 ignored、workspace tests、Clippy、Rust 1.85 check、formatがpass。
- 手動専用headless DirectWrite smokeはこのWindows環境でsystem default/明示TTFともpass。同revision `4bf0373baa2d24baaad780491bbcfd34892a9182` の[Windows/Linux CI run 41275875074](https://github.com/edamame-edame/freemvvm/actions/runs/41275875074)もsuccess（Linux bootstrap・Linux Platform device tests・Windows bootstrap・Windows Platform adapter tests）。
- Control/Scene producerがTextServiceを呼ぶCore縦断経路はまだないため、このunitの成功をUI上のtext表示完了とは扱わない。

## 承認・改訂履歴

| 日付 | 対象版 | 結果 |
|---|---|---|
| 2026-11-01 | 計画0.1 | 利用者が「OK」と承認。実装・試験合格は別管理 |
