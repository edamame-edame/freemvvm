# MVVM UIライブラリ text shaping／Unicode対応の上位判断案

- 状態: BD-23承認済み（2026-09-27）
- 対応: UTF-8テキスト、編集単位、bidirectional text、line breaking、font fallback
- 根拠: 要求仕様第5・6・12節、A6、A25～A28、BD-09～BD-12

## 既存の承認済み契約

- A6は文字列値をUTF-8境界で扱う。
- A25～A28は入力候補を扱い、BD-09／BD-10はIME preeditをTextBox編集状態に保持してcommit前にViewModelへ伝播しない。
- BD-11はTextとTextBoxを初期の葉Widgetに含める。BD-12は共通RendererでControlを描画する。
- この判断案はこれらの契約を変更せず、テキストの表示・編集に必要な最小限の意味論を提案する。

## 利用者が示した方針

2026-09-27、利用者は初期版の文字encodingをUTF-8とする方針を了承した。その後、初期版からmultibyte文字にも対応したい意向を示した。「初期から」の希望を取り込むが、「対応できるなら」という条件があるため、BD-23では初期版で保証するscript、font資産、各targetの成立条件を具体化してレビューする。platform別のfont dataとOS描画要件も記録する。

UTF-8は日本語などASCII以外のUnicode scalarも表現するため、文字列encodingとglyph表示・shaping・IME入力の対応は別の契約として扱う。本案では初期版の保証候補を**Latinと日本語UI文字列（ひらがな・カタカナ・日本語用漢字を含む）**に置く。中国語の簡体／繁体、韓国語、RTL/複雑script、emojiまで一律に保証するものではなく、必要なprofileを追加していく。

## BD-23 承認された判断

1. **初期版ではUTF-8を唯一のtext encodingとし、Latinと日本語UI文字列の表示・編集を最低保証候補にする。** 文字列の正本はlogical textとし、描画のためのUnicode normalizationは自動適用しない。対象漢字corpusとunsupported codepointの挙動を各targetのmatrixで確定する。
2. **Windows x64、Linux x64、embedded Linux ARMのすべてで上記最低保証を実機検証する。** font coverageをOS側の偶然のfont installだけに依存させない。OS fontsで満たせるprofileは依存条件を明記し、embedded profileは選定した日本語font assetをimageまたは別packageに含めることを推奨する。
3. **初期の多言語実装はplatform text serviceを分離する。** 推奨構成は、WindowsではDirectWriteのtext layout、font fallbackとcustom rendererを使い、LinuxではFontconfigのfont discoveryにHarfBuzz shapingとFreeType rasterization等を組み合わせる。text service境界は共通化し、metrics/rendering差はscript test corpusで検証する。実際のengine採用はAPI/ABI、binary footprint、性能を比較して確定する。
4. **Noto Sans CJK JP等の再配布可能な日本語fontをembedded用候補として評価する。** Font payload size、subsetによるcoverage、OFL notice/Reserved Font Name条件、font更新方法を確認して選定する。採用を決定したものではない。
5. **releaseごとにplatform capability matrixを持つ。** UTF-8 validation、Japanese glyph coverage、font source/fallback、shaping/rasterization stack、font assets/dependencies、IME入力経路、resource footprint、build/runtime verification、test corpusをtargetごとに記録する。
6. **他scriptの追加は段階的に行う。** Chinese Simplified/Traditional、Korean、Arabic/RTL、complex-script shaping、emoji/color glyphは独立したcoverage rowとfont/test requirementを設ける。TextBoxのextended grapheme cluster編集、bidi、line-break algorithmの採用と詳細policyは別途レビュー対象のままにする。

Unicode Standard Annexは書記素境界をUI操作のselection、cursor movement、backspacing等に関係する境界として説明する。Bidirectional Algorithmは混在方向テキストの方向配置を定め、Line Breaking Algorithmは折返し可能位置の候補を定める。実際にどの候補で折り返すかは幅などを扱う上位Layout側が選ぶ。これらの初期採用は今回確認されたUTF-8 encoding方針とは別にレビューする。

## Platform capability matrix案

次の行は要求項目と候補層を示す。挙げたOS API／Linux libraryは選定済み実装ではない。release前に各targetの値とverification evidenceを記入する。

| Target | Font discovery / fallback | Shaping / rasterization | Font assets / distribution | Profile verification |
|---|---|---|---|---|
| Windows x64 | Installed font collection、fallback、rendererへのfont/glyph accessを確認。DirectWrite候補 | DirectWrite text layout/shape/renderまたは共通serviceとの境界を確定 | OS搭載Japanese fontsのcoverageを列挙し、font package不要の成立条件を確認 | supported Windows profile上で日本語UI corpus、input/IME、missing-glyph casesを実行 |
| Linux x64 | distro内font discovery/fallback providerを確認。Fontconfig候補 | HarfBuzz shapingとFreeType rasterization等の組合せ、shared/static linkとABIを確認 | 日本語font packageをruntime dependencyにするか同梱するか明記。license/noticeを維持 | distro/profile別に日本語font coverage、IME、fallbackを実行検証 |
| Embedded Linux ARMv7/AArch64 | board image/sysroot内font discovery/fallbackを確認。fontを同梱する場合は選定・順序を固定 | cross-target shaping/rasterization library、sysroot、renderer backend、ABIを確認 | Japanese font subset/packageの導入経路、flash/RAM使用量、license、update方法を記録 | cross-buildとboard上runtime verificationを分け、日本語corpusを実機検証 |
| Other targets | target追加時にfont serviceとfallback sourceを選ぶ | backend capabilityとdependencyを個別記録 | font assetsと配布条件を個別記録 | 実機／runtime確認範囲を個別記録 |

capabilityはcodepoint範囲だけでなく、Latin/basic combining、Japanese/CJK、Korean、Arabic/RTL、complex-script shaping、emoji/color glyph、fallback successを分けて記録する。初期版の日本語corpusは表示、selection/caret、IME commit、line wrappingを含める。対応対象を増やす際はtest corpusとfont packageを同時に定義する。

## 理由と制約

- UTF-8 byte単位で編集すると、複数scalarからなる利用者認識上の一文字を途中で分割する可能性があるため、編集単位の設計では書記素境界を評価する。
- logical orderを保持すれば、ABI値、ViewModel binding、IME commit値を描画上の並び替えから独立させられる。
- Core LayoutとPlatform rendererの既存境界を保ちつつ、shaping engineの選択を初期の意味論契約から分離できる。
- 日本語対応は実現可能だが、font資産とruntime testを含む。font fallbackが利用できないglyphはmissing-glyph表示になり得るため、採用fontのcorpus coverageを全初期targetで検証する。
- Windows DirectWriteは国際テキストのlayout、font fallback、glyph shaping、custom rendererを提供する。Linux側ではFontconfigのfont discoveryだけではrasterizationされないため、shaperとrasterizerを別途選び、embedded Linux imageに入れる必要がある。
- 同じ日本語font assetを全targetで使えばglyph coverageのばらつきを減らせる一方、payload footprintとfont license/update責務が生じる。OS font依存を許せばpackageは軽くなるが、profile間の見え方・coverageを保証しにくい。

## この判断では決めないこと

- HarfBuzz等のshaping engineやOS-native shaping APIの採否、静的／動的リンク方式。
- Unicode／CLDR dataの採用versionと更新方針、tailoring dataの配布。
- Noto Sans CJK JPを含むexact font family、fallback order、font subset/payload budget、font update policy。
- grapheme単位のcaret移動におけるvisual/logical arrow key policy、word navigation、selection affinity、hit-testing。
- TextBoxのIME候補window placement、font variation、ligature、OpenType feature、emoji presentation。
- 縦書き、ruby、justification、advanced typography、全OS間pixel-identical rendering。

## Platform候補の根拠

- Windows DirectWriteはfont enumeration、fallback、font cachingを提供し、custom rendererでglyph drawingを接続できるとMicrosoftが説明している（[Introducing DirectWrite](https://learn.microsoft.com/en-us/windows/win32/directwrite/introducing-directwrite)）。BD-23では採用APIとせず、必要capabilityの候補根拠とする。
- LinuxのFontconfigは利用可能fontの検索・substitutionを担当する一方、font自体のrasterizationは行わない（[Fontconfig](https://wiki.freedesktop.org/www/Software/fontconfig/)）。したがってLinux profileではfont discoveryとshaping/rasterization dependencyを別々に記録する。
- HarfBuzzはUnicode codepoint列からfont glyphを選び、positioning／script layoutを行うshaping engine候補である（[What is HarfBuzz?](https://harfbuzz.github.io/what-is-harfbuzz.html)）。採否とfont discovery／glyph rasterizerとの組合せは未決。

## 承認記録

利用者は2026-09-27にBD-23を承認した。初版のUTF-8 text boundaryに加え、Latinと日本語UI文字列を初版から表示・編集できることを最低保証候補として採用し、Windows／Linux x64／embedded Linuxごとにfont、shaping/rasterization、配布依存、IMEとruntime verificationを管理する。Windows DirectWrite、Linux Fontconfig＋shaper/rasterizer、Noto Sans CJK JP等は実装・font候補とし、exact corpus、font packageとversion、license notice、他script coverage、caret/navigation policyは後続で確定する。

## 参考資料

- [Unicode Standard Annex #29: Unicode Text Segmentation](https://www.unicode.org/reports/tr29/) — extended grapheme clusterとtext boundary。
- [Unicode Standard Annex #9: Unicode Bidirectional Algorithm](https://www.unicode.org/reports/tr9/) — bidirectional textの方向処理。
- [Unicode Standard Annex #14: Unicode Line Breaking Algorithm](https://www.unicode.org/reports/tr14/) — line-break opportunities。
- [Microsoft DirectWrite: Introducing DirectWrite](https://learn.microsoft.com/en-us/windows/win32/directwrite/introducing-directwrite) — international text、font fallback、glyph shaping、custom renderer。
- [Fontconfig Developers Reference](https://fontconfig.pages.freedesktop.org/fontconfig/fontconfig-devel/) — Linux font matching/discovery。
- [HarfBuzz Manual](https://harfbuzz.github.io/) — Unicode inputからpositioned glyphを生成するshaping library。
- [Noto CJK fonts](https://github.com/notofonts/noto-cjk) — JP/KR/SC/TC/HK別font family/release。Noto CJK Sans/SerifはSIL Open Font License 1.1。

BD-23は上位判断として承認済み。exact Japanese corpus、font package/version、他script coverage、詳細text editing policyは後続判断とする。DD-00は未承認のため、DD-01以降へ進まない。
