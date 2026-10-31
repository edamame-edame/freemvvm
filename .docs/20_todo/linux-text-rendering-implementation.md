# Linux TextService・GlyphRun実装

- 計画版: 0.3
- 状態: RQ-09-075、Platform DD 1.9、UI Controls DD 1.4、UI UT 1.2、計画0.2全文承認済み。Linux resolver/dependency/test profileと実装unit分割0.3は利用者承認済み。UNIT-CORE-084はcommit/push済み（`b6c64ea`）。UNIT-PLATFORM-003はLinux Dockerで実装・検証完了
- 作成日: 2026-10-31
- 対象version: `0.0.1`
- 対象branch: `impl-rust/core-scene-rendering`
- 上位計画: [Linux初期Text描画profileの確定](linux-text-rendering-profile.md)

## 目的

承認済み[RQ-09-004/006/072/075](../11_requirements/RQ-09_ui-scope.md)に沿って、Text・TextBox本文・Button labelの単一paragraph表示をCore-owned paint/metricsとLinux GLES GlyphRunまで実装する。PlatformDefaultではsystem fallbackを許し、解決後のfont identityを保持する。fallback chainで解決できない表示・Layout/Scene/frameは明示失敗とする。

## Linux initial profile・implementation unit案（0.3レビュー対象）

| 項目 | 推奨候補 | 根拠・要確認 |
| --- | --- | --- |
| Font供給・候補 | Linux TextServiceはFontconfigのsystem configurationを使用し、generic family `sans-serif`, normal weight, current profile's language contextから既定faceを決める | headless DRM/KMSでありGNOME/KDE等desktop UI toolkitの設定は読まない。異なるhost/profile間でfamily/pixel一致は要求しない。resolved faceごとにfamily/style/file SHA-256/face index/resolver identityを保持 |
| Resolution/fallback | request patternへsystem `sans-serif`を設定し、`FcConfigSubstitute` + `FcDefaultSubstitute`後に`FcFontSort(trim=false)`でpriority-ordered candidate fontsとUnicode coverageを得る。TextServiceはshaping cluster全体を最初にcoverageを満たす候補へ割当て、HarfBuzzでrunごとにshape | Fontconfigのgeneric matching自体はexact family guaranteeでなく、TextServiceが結果とfallback run identitiesを記録する。候補全体で未解決cluster、設定/DB不備は明示error。Fontconfig docsによれば`FC_LANG`がなければdefault languageはcurrent locale由来 |
| Core接続 | Core-neutral `PlatformDefault` request → PlatformAdapterが保持する`TextService` → immutable `ResolvedTextSnapshot` (metrics + glyph runs + per-run resolved identity) → Coreが同一revisionをLayout/Sceneで共有 | CoreはOS font name/path/APIを知らない。Rendererは同じPlatformRuntimeのresolved identity/resource registryを使い、default fontを選び直さない |
| Rust text engine bindings | `fontconfig = 0.11.0` + `dlopen` feature、`harfbuzz = 0.8.0` (`default-features=false`, `std` + `freetype`)、`freetype-rs = 0.38.0`を候補採用。HarfBuzzはsystem buildを使いbundled featureは無効 | binding feature/buildをUNIT-PLATFORM-003の最初のcompile gateで確認する。Windows wrapper/APIには依存させない。Fontconfig runtime欠落はTextServiceの明示初期化error |
| Debian 12 verify profile | `fontconfig` / `fontconfig-config` / `libfontconfig1` `2.14.1-4`; `libharfbuzz-dev` `6.0.0+dfsg-3`; `libfreetype-dev` `2.12.1+dfsg-5+deb12u4`; `fonts-noto-cjk` `1:20220127+repack1-1`をDocker test imageだけに追加する案 | Fontconfig crateは`dlopen`を使うためdev headersは不要候補。CJK font packageは約55.2 MB download / 91 MB installed。repoへfont assetは同梱しない。Docker corpusはDebian system default+fallbackを検証し、hardware profileが同じfaceを選ぶとは主張しない |
| 文字appearance | opaque black `#000000`、normal/Regular、`14 DIP`を初期Core text paint候補として維持 | system family/default face選択とは別のCore-owned value。system resolverからsizeを取得する契約は今回のLinux headless profileにはなく、DeviceScaleをfont sizeへ二重適用しない。dark backgroundでもautomatic contrastはしない |
| 初期test corpus | Latin `FreeMvvm 0123`、Japanese `日本語表示`、mixed `FreeMvvm 日本語`をLinux Docker integration corpusにする案。fake resolverで複数face fallback identityとuncovered clusterも検証 | corpusはLinux initial verification setであり、将来全Unicode coverageを保証しない。resolved faces/coverageをtest reportへ記録し、exact pixel outputを異platform比較しない |

上表のFontconfig family/language/fallback policy、dependency/crate versions、CJK test image追加と表示色/size候補は、本計画0.3のレビューで承認済みである。`fonts-noto-cjk`はCI/Docker用のみで製品binary/repositoryへ同梱しない。各実装unitの範囲に応じて依存追加・Docker image変更・oracle実装を行い、未指定fontを特定faceへ固定しない。

## 実装unit案

1. **UNIT-CORE-084 — Core TextService request/result seam**: Core-neutral `PlatformDefault` requestとresolved font-run identity/metrics snapshotを導入。fake resolverでplatform別identity差、Measure/Scene同revision利用、failure atomicityを確認。C ABI/wrapperは変更しない。
2. **UNIT-PLATFORM-003 — Linux Fontconfig/HarfBuzz/FreeType realization**: system `sans-serif` + Fontconfig ordered coverage fallback、HarfBuzz shaping、FreeType metrics/maskを実装。Latin/Japanese/mixed corpus、複数face、missing config/font、uncovered cluster、invalid textを検証。Docker test profileでは`FC_LANG=ja-jp`を明示し、実機のFontconfig language settingは変更しない。
3. **UNIT-CORE-085 — Text Control/Layout/Scene wiring**: Text/TextBox editing/Button label snapshotsをresolved metricsと結び、same revisionをLayout/Visual sceneまで通す。AccessibleName分離と失敗時complete snapshot保持を検証。
4. **UNIT-PLATFORM-004 — Linux GLES glyph atlas/render**: glyph mask atlas, HarfBuzz positions, Core color, DeviceScale, Clipを合成し、atlas/upload/draw/device-loss failureをframe failureへ伝える。欠落/空GlyphRunをsuccessful Presentedにしない。

各unit開始前に`tools/units.yaml`へpath/requirements/design/test mappingを登録し、対応testと同じ変更revisionで検証する。unit test passごとに日本語commit/pushする。Linux hardware display確認はDocker/fake surface passと分離し、保留中の実機確認gateに残す。

## 対象外

- Windows DirectWrite、C ABI/Rust/C++/C#/Python wrapper変更。
- public Style/Theme/Font API、Button/TextBox chrome、caret/selection、IME、Accessibility。
- soft-wrap/ellipsis/automatic contrast。Platform system defaultとしての明示的に追跡可能なfallbackは対象に含むが、silent fallbackは禁止。
- unit完了前のmerge PR。関連TODOがまとまって完了した後にPRを作る。

## 承認境界

- 承認済み計画0.3に沿ってunit mappingを`tools/units.yaml`へ登録した。UNIT-CORE-084はcommit/push済み、UNIT-PLATFORM-003はLinux Docker focused/workspace Rust tests、Clippy、Rust 1.85 check、fmtをlocal passした。以後も各unitの対応testを同一実装revisionに含める。
- profileの確定値・依存導入・実装着手・各unitのtest合格・commit/push・PRは別々に記録する。
- RQ-09-075、DD 1.9、UI Controls 1.4、UI UT 1.2のみの承認と、exact Linux dependency/profile valuesを含む計画0.3の承認は別記録である。計画0.3は利用者承認済みだが、実装・test pass・commit/pushはunitごとに別記録する。

## UNIT-PLATFORM-003 実装・検証記録

- Linux `freemvvm-platform-linux`へFontconfig `sans-serif`候補順、grapheme単位coverage fallback、HarfBuzz run shaping、FreeType 14 DIP metrics/gray mask cacheを実装。各runはresolved family/style/file SHA-256/face indexをCore-neutral `ResolvedTextSnapshot`へ記録する。C ABIへText APIは追加しない。
- `abi_support`のhidden Rust SPIからTextService request/result型をplatform crateへ公開し、metrics値は検証済みconstructor/getter経由とした。C ABIの既存Control統合testはLinuxで実DRM/KMS `Window.open`を呼ばないよう分離し、実機確認を保留する。
- Debian 12 Docker (`FC_LANG=ja-jp`)で `cargo test --workspace --all-targets --locked` 成功。Linux Platform 19 passed / 1 ignored（headless GLES pixel testは別条件）、Core 180 passed。`cargo clippy --workspace --all-targets --locked -- -D warnings`、`cargo +1.85.0 check --workspace --all-targets --locked`、`cargo fmt --all -- --check` 成功。
- 実行結果はDocker/ソフトウェア側のtext shapingとraster mask生成を確認したもの。live Control/Layout/Scene接続はUNIT-CORE-085、GLES glyph atlas/presentはUNIT-PLATFORM-004、DRM実機のopen/displayはhardware確認であり完了を意味しない。

## 実装計画0.3の参照資料

- Fontconfig language context and config environment: [Fontconfig user documentation](https://fontconfig.pages.freedesktop.org/fontconfig/fontconfig-user.html)
- Ordered matching/candidate coverage: [Fontconfig developer reference: `FcFontSort`](https://fontconfig.pages.freedesktop.org/fontconfig/fontconfig-devel/)
- Debian 12 packages: [fontconfig-config 2.14.1-4](https://packages.debian.org/bookworm/fonts/fontconfig-config), [libharfbuzz-dev 6.0.0+dfsg-3](https://packages.debian.org/bookworm/libdevel/libharfbuzz-dev), [libfreetype-dev 2.12.1+dfsg-5+deb12u4](https://packages.debian.org/bookworm/libdevel/libfreetype-dev), [fonts-noto-cjk 1:20220127+repack1-1](https://packages.debian.org/bookworm/fonts/fonts-noto-cjk)
- Rust bindings: [fontconfig 0.11.0](https://docs.rs/fontconfig/0.11.0/fontconfig/), [harfbuzz 0.8.0](https://docs.rs/harfbuzz/0.8.0/harfbuzz/), [freetype-rs 0.38.0](https://docs.rs/freetype-rs/0.38.0/freetype_rs/)

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-10-31 | 計画0.2 | 利用者承認 | Platform defaultフォントとCore-neutral TextService接続方針を承認。exact Linux profile/unit mappingは後続計画へ分離 |
| 2026-10-31 | 計画0.3 | 利用者承認 | 「ok」によりFontconfig `sans-serif`/locale/fallback、Rust binding、Debian package versions、Docker Japanese corpus、4-unit splitを承認。profile実装・各unit合格は別管理 |
