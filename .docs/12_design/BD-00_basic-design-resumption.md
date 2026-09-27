# MVVM UIライブラリ 上位設計再開計画

- 状態: 作業計画。BD-01～BD-35承認済み（2026-09-27）。DD-00は未承認
- 作成日: 2026-09-26
- 対象: DD-00の承認・DD-01以降の詳細設計より前の上位判断

## 1. 出典と承認境界

- ADR-0001はAccepted。Rust Core、versioned C ABI、4言語wrapper、組み込みLinuxを前提とする。
- `runtime-contract.md`のR1～R4、`abi-spec.md`のA1～A32、`mvvm-semantics.md`のP1～P4・B1～B4・C1～C4・D1～D4・V1～V8、`language-bindings.md`のL1～L4はそれぞれの本文の範囲で承認済み。配布用Cヘッダや未記載の実装方式まで承認したものではない。
- `00-decision-map.md`はDD-00の未承認レビュー案であり、本文も新しい判断を行わないと明記する。そのDD-01割当てをこの計画の承認済み前提としない。
- 利用者が要求仕様と`design-overview.md`のAccepted状態を確認した（2026-09-26）。手元の両文書の状態表記をAcceptedへ訂正した。この確認はDD-00や本計画のBD-01の承認を意味しない。

## 2. 要求仕様第12節の未決事項と現在の照合

「既に判断済み」は列挙された論点の基本契約が既存の承認済み判断にある場合に限る。「一部判断済み」は外縁・詳細・対象範囲が残る場合。「未判断」は検討候補や要求の記載だけの場合。

BD-34承認後の内訳は18項目中「既に判断済み」4件、「一部判断済み」14件、「未判断」0件。部分判断済み14件は、すべてが新たな上位判断を必要とする意味ではなく、多くは個別Control・platform・packageの詳細設計に送った内容を含む。

| 第12節の項目 | 分類 | 承認済みの範囲／残る判断 |
|---|---|---|
| 対象OSの正式な優先順位 | 一部判断済み | BD-12で対応・検証順をWindows→Linux x64→組み込みLinux ARMv7/AArch64→その他とし、Linux x64からARMへcross-compileする方針を承認。正式な公開順位・CI matrixは未決。 |
| native controlか独自描画か | 既に判断済み | BD-12承認。初期Controlは共通Rendererで独自描画し、native controlを意味・状態のsource-of-truthにしない。 |
| RendererのAPIと描画モデル | 一部判断済み | BD-12でCore Visual Tree／LayoutとPlatform renderer/backendを分離する境界を承認。Scene、描画命令、更新領域、backend APIは未決。 |
| text shaping、Unicode、RTL対応範囲 | 一部判断済み | BD-23承認。初版UTF-8、Latin＋日本語UI文字列の表示・編集、Windows／Linux x64／embedded Linuxごとのfont/shaping/dependency/verification管理を決定。exact corpus/font package/version、他script coverage、caret/navigation policyは後続。 |
| IME compositionの詳細 | 一部判断済み | V5～V8、A25～A28とBD-09／BD-10／BD-24でpreedit保持・commit検証、FocusLost、明示cancel時の復元、session identity、commit/FocusLost順序を決定。Undo/Redo、caret・selection細則、OS APIは後続。 |
| binding循環の扱い | 一部判断済み | D1～D4は変換なし同型Propertyの循環を制限し、BD-03は有限作業枠・UIへの譲り・保留・診断と静的警告を決定。上限値、静的解析の適用範囲、非収束判定・復旧の詳細は未決。 |
| validationの標準モデル | 一部判断済み | V1～V8、A17～A20・A25～A28で同期Property検証とView入力検証を規定。BD-25承認済み: 初版のCore validatorは短時間同期のみ。remote/long-running checkはViewModel AsyncCommand＋状態Propertyで表し、stale resultはアプリ側で識別する。複数Property、共通表示・再検証、将来のCore async APIは後続。 |
| async Commandのキャンセル仕様 | 一部判断済み | C1～C4、A21～A24およびBD-26で協調的キャンセル・状態遷移・二重起動拒否・shutdown、中間進捗の最新値集約、終端結果の独立一回適用とoperation IDによる古い通知破棄を規定。画面による所有の具体API、結果値、進捗型・閾値・複数channelは未決。 |
| UI transactionの有無 | 既に判断済み | BD-02承認。初期版に明示transactionは設けず、各setterで値を確定し、複数setterの自動rollbackは保証しない。複数Propertyの原子的更新は、必要性が確認された場合に別判断。 |
| error objectの詳細構造 | 一部判断済み | A2、A15、A18およびBD-27でstatus、domain/code/message、snapshotと寿命、machine-readable境界、causeとdiagnosticの分離を決定。全code catalog、ABI表現とdiagnostic payloadは未決。 |
| ABI互換性ポリシー | 一部判断済み | A3・BD-28でMAJOR.MINOR.PATCH、major付きsymbol、minor互換追加、patch互換fixを規定。初回stable番号・pre-release policy・各wrapper package制約は後続。旧headerによる検証とsupport期間も決定済み。 |
| Python wrapperの最終方式 | 既に判断済み | BD-14承認。初期Python 3 wrapperは標準ライブラリctypesでversioned C ABIを動的ロードし、compiled extensionを必須にしない。package、wheel、interpreter matrixは未決。 |
| Python lint/type check | 一部判断済み | BD-15承認。library compiler共通diagnosticsをMCP/CLI/CIから利用し、Temp cacheで増分解析。Pythonの同一Control Property警告は静的に解決できる範囲。checker製品・MCP schema・cache policyは未決。 |
| 配布形式 | 一部判断済み | ADR-0001はnative libraryと言語wrapperを分離しtarget別artifactを規定。BD-17～BD-20は各言語配布境界、BD-29～BD-30はimmutable署名artifact、公式catalog、既定package manager、明示target・version/digest pinning、BD-31は独立trust root・key rotation/revocation・offline trust bundle、BD-32はmanifest schema evolutionを承認。暗号形式、registry実装、fetch/cache運用は後続。 |
| ライセンス | 既に判断済み | BD-21承認。First-party source、C ABI/header、各language wrapper、source package、target-specific binary SDKにApache-2.0を適用し、改変source／改変binary／app binaryの再配布を許可する。third-party dependencyは各上流licenseを維持。公開repo、CLA/DCO、docs/assets、SBOM等は未決。 |
| サポート期間 | 一部判断済み | BD-22承認。最新安定majorの通常保守、直前majorを新major安定版後24か月security／重大修正のみ、profile非推奨は原則12か月前通知、ABI/profile単位のmatrixを決定。security response SLA、LTS、exact target matrixは未決。 |
| CIで検証するコンパイラとtarget matrix | 一部判断済み | BD-16承認。初期matrixはWindows x64、Linux x64、ARMv7/AArch64 cross-build。実機gateとbuild verified/runtime verifiedを区別。exact compiler/sysroot/board/正式release matrixは未決。 |
| accessibility APIの対象範囲 | 一部判断済み | BD-13承認。Core Widgetの共通role/name/state/focusability/focus/enabled/keyboard意味情報、初期Controlのvalue/action、Logical Tree基礎のAccessibility treeを決定。OS別写像、name fallback、ABI/API詳細は未決。 |

## 3. この回の小さな設計課題と停止条件

1. 完了: `../12_design/value-types-upper-design.md`のBD-01でBOOL/I64/UTF8にF64とscale 5ビットの独自8バイトDECIMALを追加する上位判断を2026-09-26に承認。カスタム、一般的なnull許容値、OBJECTは後続、Collectionは別Core機能とした。
2. 完了: A5の初期公開値型の種類数がBD-01で改訂されたことを`abi-spec.md`へ記録。F64/DECIMALの具体ABIは未設計。BD-33でCollection/GridのCore model境界、stable ID、change set、visible-cell virtualizationを承認。
3. DD-00の承認やDD-01以降の詳細設計には進まない。要求仕様と設計骨格のAccepted表記は利用者の確認に従い更新済み。
4. 完了: 要求仕様第12節「UI transactionの有無」を`../12_design/ui-transaction-upper-design.md`のBD-02で承認し、初期版に明示transactionを設けず、複数setterの自動rollbackを保証しないと決定。
5. 完了: `../12_design/runtime-notification-upper-design.md`のBD-03を承認。通知配信の有限な作業枠、保留とUIへの譲り、非収束連鎖の停止と診断、同一Control Propertyアクセスの静的警告（Pythonを除く）を決定し、`mvvm-semantics.md`と要求仕様第12節へ反映。
6. 完了: `../12_design/ui-tree-lifetime-upper-design.md`のBD-04を承認。親子所属によるCore内部強参照、Core objectとPlatform peerの寿命分離、公開handleが残るDetached object、複数親・循環の禁止を決定。Logical/Visual Treeの役割と具体APIは後続に分ける。
7. 完了: `../12_design/logical-visual-tree-upper-design.md`のBD-05を承認。Logical TreeとVisual Treeを別概念とし、前者をアプリのControl包含・所有寿命、後者をlayout・描画・hit-test対象とした。template内部要素はLogical Treeに露出させない。
8. 完了: `../12_design/layout-upper-design.md`のBD-06を承認。Measure/ArrangeによるLayout計算をCoreのVisual Tree側に置き、Platformへ最終boundsを渡す。Sizer／Locationerと`Align`等の調整可能要素を含む。独自アルゴリズムの公開拡張は別判断。
9. 完了: `../12_design/input-event-boundary-upper-design.md`のBD-07を承認。Platform Adapterがraw inputを正規化し、Core InputがUI threadで共通意味論を処理する責務境界を決定。focus、IME、event routingは後続判断。
10. 完了: `../12_design/focus-upper-design.md`のBD-08を承認。同一Runtimeのactive Windowとactive focusを一元管理し、非active Windowの復帰候補と区別する。
11. 完了: `../12_design/ime-focus-transition-upper-design.md`のBD-09を承認。Focus移動時の未確定composition文字列と編集位置をTextBox編集値として保持し、IME session再開やVM伝播は行わない。
12. 完了: `../12_design/ime-platform-boundary-upper-design.md`のBD-10を承認。Platform Adapterがnative IME sessionとcandidate UIを所有し、Core TextBoxが編集文字列・selection/caret・入力状態を所有する。preeditはVMへ流さず、commit時にA25～A28へ接続する。
13. 完了: `../12_design/initial-control-contract-upper-design.md`のBD-11を承認。共通抽象はWidget、子を持ちうる分類はContainerとし、Window／StackPanelをContainer、TextBox／Text／Buttonを葉Widgetにする。
14. 完了: `../12_design/rendering-platform-upper-design.md`のBD-12を承認。初期Controlは共通Rendererで独自描画し、対応・検証順はWindows→Linux x64→組み込みLinux→その他。Linux x64からARM向けへcross-compileする。
15. 完了: `../12_design/accessibility-upper-design.md`のBD-13を承認。Core Widgetに共通Accessibility意味情報を持たせ、Platform AdapterがOS APIへ写像する。初期treeはLogical Treeを基礎とし、Visual template部品は既定で公開しない。
16. 完了: `../12_design/python-wrapper-upper-design.md`のBD-14を承認。初期Python 3 wrapperは標準ライブラリctypesでversioned C ABIを動的ロードし、compiled extensionを必須にしない。
17. 完了: `../12_design/python-lint-mcp-upper-design.md`のBD-15を承認。強いPython lint/type check、Temp cacheによる差分解析、MCP/CLI/CI共通diagnosticsを採用。静的に解決できるPython event callbackもBD-03の同一Control Property警告対象とする。
18. 完了: `../12_design/build-ci-target-matrix-upper-design.md`のBD-16を承認。初期CIはWindows x64、Linux x64、ARMv7/AArch64 cross-buildの順。cross-buildと実機hardware profile検証を区別する。
19. 完了: `../12_design/python-package-distribution-upper-design.md`のBD-17を承認。Python wrapperとtarget native runtimeを別distributionとし、Windows x64／Linux x64のnative wheelから開始。ARM Linuxは互換性確認前にstandard wheel tagを付けない。
20. 完了: `../12_design/csharp-nuget-native-assets-upper-design.md`のBD-18を承認。C# managed wrapperとRID別native C ABI assetを同一NuGet packageにまとめ、初期RIDを`win-x64`／`linux-x64`とする。
21. 完了: `../12_design/cpp-cmake-package-upper-design.md`のBD-19を承認。C++ wrapperをCMake install/export packageとして供給し、Windows x64／Linux x64の共通target C ABI native artifactを接続する。
22. 完了: `../12_design/rust-cargo-package-upper-design.md`のBD-20を承認。raw FFI／safe wrapper crateを分け、共通native artifactを別配布する。別environment artifact import featureを用意し、Cargo `--target`とnamed profileでtargetを指定、artifact metadata不一致を拒否する。
23. 完了: `../12_design/license-upper-design.md`のBD-21を承認済みとして記録。Apache-2.0を採用し、source／改変source／改変binary／app binaryの再配布を許可。公式artifactの署名・provenance、商標方針とsupport lifecycleは別判断に残す。
24. 完了: `../12_design/support-lifecycle-upper-design.md`のBD-22を承認。最新安定majorの通常保守、直前majorの24か月security／重大修正保守、ABI major・target profile別matrix、原則12か月前の非推奨通知、公式保守終了としてのEOLを決定。
25. 完了: `../12_design/text-shaping-upper-design.md`のBD-23を承認。初版UTF-8、Latin＋日本語UI文字列の表示・編集、target別font/shaping/runtime verification管理を決定。exact corpus、font package/version/license、他script coverage、詳細caret/navigationは後続。
26. 完了: `../12_design/ime-edit-session-upper-design.md`のBD-24を承認。explicit cancelは開始snapshotへ戻し、FocusLostはpreedit保持、IME session identityとcommit/FocusLost順序、stale event破棄を決定。
27. 完了: `../12_design/async-validation-upper-design.md`のBD-25を承認。初版は短時間同期Validatorのみとし、remote/long-running validationはViewModel AsyncCommand＋状態Propertyで表す。stale result識別はアプリ側。
28. 完了: `../12_design/async-command-progress-upper-design.md`のBD-26を承認。中間進捗の集約を許し、終端結果を独立して一度だけ適用する。処理順序とoperation IDで古い／終端後通知を破棄し、具体頻度・型は詳細設計へ送る。
29. 完了: `../12_design/error-object-upper-design.md`のBD-27を承認。domain/code/statusを機械判定用に安定させ、messageは説明用、causeとdiagnosticを分離する。
30. 完了: `../12_design/abi-release-version-upper-design.md`のBD-28を承認。安定版native runtimeをMAJOR.MINOR.PATCHとし、MAJORをA3 symbol majorへ一致させる。wrapper package versionは独立可能で、対応ABI major/minimum runtimeを示す。
31. 完了: `../12_design/artifact-provenance-upper-design.md`のBD-29を承認。immutable artifact identityと署名付きmanifest/digestを用い、利用前にtarget/profile、ABI、feature、署名、digestを照合し、不適合や検証不能は拒否する。
32. 完了: `../12_design/artifact-distribution-upper-design.md`のBD-30を承認。公式native artifact catalogをsource of truthとし、language wrapperを既定package managerで配布する。cross-targetはtarget/profileを明示し、buildではimmutable version/digestを固定する。
33. 完了: `../12_design/artifact-signing-trust-upper-design.md`のBD-31を承認。trust rootはcatalogと独立にprovisionし、未知・失効・検証不能keyを拒否。rotationは認証済みkeyset、offline importは明示trusted bundleと失効情報を使う。
34. 完了: `../12_design/artifact-manifest-schema-upper-design.md`のBD-32を承認。schema versionをruntime versionから分け、同一major minorを互換とする。unknown required/critical field、major、曖昧表現は拒否し、署名対象に全manifest contentを含める。
35. 完了: `../12_design/collection-grid-upper-design.md`のBD-33を承認。CollectionModel/SelectionModelを値型・UI treeから独立させ、stable item/column ID、change set、Grid visible cell virtualizationを採用。
36. 完了: `../12_design/selection-model-upper-design.md`のBD-34を承認。SelectionModelにSingle/Multiple modeを設け、current itemはselectionとkeyboard focusから分離する。削除itemのselection/currentを除き、暗黙の後継選択はしない。
37. 完了: `../12_design/collection-change-batch-upper-design.md`のBD-35を承認。Collection batchはModel整合性についてall-or-nothingとし、Selectionをcommit前に整合させる。独立入力欄は各自validation後に個別commitでき、別入力を待たない。DD-00は未承認。

## 4. 次の上位判断の順序

「後回し」は最初の値型判断より後に回せるかを示す。縦断シナリオ／製品版の入口では別途必要性を確認する。

| 順 | 上位判断 | 既存の根拠 | 決める内容 | 影響する詳細設計フェーズ | 後回しにできるか |
|---:|---|---|---|---|---|
| 1a | UI transaction（BD-02承認済み） | P2～P4、B2、D3、R4 | 初期版は明示transactionなし。複数Property原子的更新は必要性が出た場合の別判断 | DD-03、DD-07 | 初期版の判断済み。拡張は必要性確認後 |
| 1b | Runtime通知上限・診断・静的警告（BD-03承認済み） | R4、P3、D3、Runtime契約第5節 | 再配信が止まらない場合の上限・診断・イベントループへの譲り方、同一Control Propertyアクセスの静的警告 | DD-03、DD-04、DD-07、DD-16 | 上位判断済み。詳細設計で閾値・検出方式を定める |
| 2a | UI treeの所有・寿命（BD-04承認済み） | Runtime契約第3節、設計骨格第3～6節、A29～A32 | 親子所属のCore所有参照、detachと公開handleの寿命 | DD-11、DD-17 | 上位判断済み。Logical/Visual Tree設計へ進む |
| 2b | Logical/Visual Treeの役割（BD-05承認済み） | 設計骨格第3～6節、調査資料「画面基盤」、BD-04 | アプリのControl包含とtemplateを含む表示構造の分離 | DD-11～DD-13、DD-17 | 上位判断済み |
| 2c | Layout責務境界（BD-06承認済み） | 設計骨格第3～6節、要求仕様第4・6節、BD-05 | CoreとPlatformのMeasure/Arrangeおよびboundsの責任 | DD-12、DD-13、DD-17 | 上位判断済み |
| 2d | Input event境界（BD-07承認済み） | 設計骨格第3～6節、R2～R4、A25～A28 | Platform input adapterとCore Inputの正規化・処理境界 | DD-14～DD-16 | 上位判断済み |
| 2e | Focus（BD-08承認済み） | R2～R4、BD-05、BD-07、要求仕様第5節 | Runtimeのactive Window／Focus対象、keyboard入力先、Logical/Visual対応 | DD-14～DD-16 | 上位判断済み |
| 2f | Focus移動時のIME composition（BD-09承認済み） | V5～V8、A25～A28、BD-08 | 未確定compositionの保持・伝播抑止・再編集 | DD-14～DD-16 | 上位判断済み |
| 2g | IME Platform境界（BD-10承認済み） | 設計骨格第4・8節、R2～R4、A25～A28、BD-09 | native IME session／candidate UIとCore編集状態の責務 | DD-14～DD-16 | 上位判断済み。詳細はTextBox実装前に必要 |
| 2h | 初期Control契約（BD-11承認済み） | 要求仕様第5・6節、A21～A32、BD-04～BD-10 | Widget／Container分類とWindow、StackPanel、TextBox、Text、Buttonの意味上の責務 | DD-12～DD-17 | 上位判断済み。画面縦断前に詳細化 |
| 3 | 描画方式と対象OS（BD-12承認済み） | ADR-0001の組み込みLinuxとbackend候補、要求仕様第8・12節、開発環境Windows | Windows → Linux x64 → 組み込みLinux → その他、Linux x64資産からARM cross-compile | DD-12、DD-14～DD-16、DD-22～DD-23 | 上位方針判断済み。backend詳細は着手前に必要 |
| 4 | Accessibility（BD-13承認済み） | 要求仕様第5・7・12節、BD-05、BD-08、BD-11～BD-12 | Core共通意味情報とLogical Tree基礎の初期Accessibility tree | DD-13～DD-17、DD-23 | 上位意味論判断済み。初期縦断でnameを検証 |
| 5a | Python wrapper方式（BD-14承認済み） | ADR-0001、承認済みL1～L4、A3 | ctypesでC ABIを呼び出すPython wrapper | DD-18～DD-20 | 上位判断済み |
| 5b | Python lint/type checkとMCP連携（BD-15承認済み） | BD-03、L1～L4、Python向けwrapper、View言語要求 | compiler共通diagnostics、Python型解析、MCP公開、Temp cache増分解析 | DD-18～DD-20 | 上位方針判断済み。checker/schema/cache詳細は後続 |
| 5c | Build/CI target matrix（BD-16承認済み） | ADR-0001、BD-12、要求仕様第8～10節 | Windows x64/Linux x64/ARM cross-buildと実機verificationの区別 | DD-21～DD-23 | 上位方針判断済み。exact toolchain/profileは後続 |
| 5d | Python package/native distribution（BD-17承認済み） | ADR-0001、BD-14、BD-16、要求仕様第9節 | pure-Python wrapperとtarget別native runtime、初期native wheel coverage | DD-21～DD-23 | 上位方針判断済み。index、tag、Python matrixは後続 |
| 5e | C# NuGet native assets（BD-18承認済み） | ADR-0001、要求仕様第9節、BD-16 | managed wrapperとRID別native assetを同じNuGet packageに含め、初期RIDを選ぶ | DD-21～DD-23 | 上位方針判断済み。TFM、feed、loader等は後続 |
| 5f | C++ CMake package（BD-19承認済み） | ADR-0001、要求仕様第4・9節、BD-16 | C++ wrapperと共通C ABI targetのCMake integration | DD-21～DD-23 | 上位方針判断済み。package manager、toolchain detailは後続 |
| 5g | Rust Cargo package（BD-20承認済み） | ADR-0001、A3、要求仕様第9・12節 | raw FFI／safe wrapper crate、別配布native runtime、別環境artifact import feature | DD-21～DD-23 | 上位境界判断済み。registry/fetch/provenanceは後続 |
| 5h | First-party license（BD-21承認済み） | 要求仕様第9・12節、各language distribution | Apache-2.0、改変source/binary再配布、notice条件 | DD-21～DD-23 | 上位判断済み。配布前にlicense/notice実装が必要 |
| 5i | Support lifecycle（BD-22承認済み） | ADR-0001、要求仕様第9・12節、ABI A3、BD-16 | ABI major別保守期間、security-only期間、target/profile matrix、非推奨とEOL | DD-21～DD-23 | 上位判断済み。詳細なSLA／LTSは後続 |
| 6a | Text shaping／Unicode／RTL（BD-23承認済み） | A6、A25～A28、要求仕様第5・6・12節、BD-09～BD-12 | 初版UTF-8＋Latin/Japanese UI text、target別font/shaping/dependency/runtime verification matrix | DD-12～DD-17 | 上位契約済み。exact corpus/font assetは詳細前に固定 |
| 6b | IME edit-session semantics（BD-24承認済み） | V5～V8、A25～A28、BD-09、BD-10、BD-23 | composition update/commit/cancelとFocusLost競合時のCore編集値・Binding伝播 | DD-14～DD-17 | 上位契約済み。Undo/Redo・OS APIは詳細設計 |
| 6c | 非同期Validationの初版範囲（BD-25承認済み） | V1～V8、C1～C4、A21～A28、R2～R4 | 初版Core validatorを同期限定とし、remote/long-running checkをViewModel AsyncCommand＋状態Propertyで表す | DD-07、DD-17～DD-18 | 初版境界は判断済み。共通async APIは後続判断 |
| 6d | Async Command進捗配信（BD-26承認済み） | C1～C4、R2～R4、BD-25 | 中間進捗の集約・破棄、終端結果との順序と一度限りの反映 | DD-07、DD-17～DD-18 | 上位保証は判断済み。頻度・閾値・型は詳細設計へ送れる |
| 6e | Error object安定性境界（BD-27承認済み） | A2、A15、A18、V1～V4、C1～C4、BD-03 | 機械判定codeと説明message、causeとdiagnosticの境界 | DD-03～DD-07、DD-18～DD-20 | 上位境界は判断済み。全catalog・表現詳細は後続可能 |
| 6f | ABI release version scheme（BD-28承認済み） | A3、BD-22、要求仕様第9・12節 | 公開runtime versionとsymbol major、minor/patch互換性規則の対応 | DD-02～DD-03、DD-21～DD-23 | 安定版規則は判断済み。個別wrapper package versionは後続可能 |
| 6g | Native artifact provenance（BD-29承認済み） | BD-16～BD-22、BD-28、Rust Cargo artifact import feature | artifact manifestの識別情報、署名・digest・metadata検証 | DD-21～DD-23 | 上位検証方針は判断済み。鍵管理・registry実装詳細は後続可能 |
| 6h | Native artifact distribution route（BD-30承認済み） | BD-17～BD-20、BD-22、BD-28～BD-29 | 公式catalog、各wrapper package manager、明示target、version/digest pinning | DD-21～DD-23 | 上位責務は判断済み。URL/schema/各package manager細部は後続可能 |
| 6i | Artifact signature trust policy（BD-31承認済み） | BD-29～BD-30、BD-21 | trusted publisher identity、key rotation/revocation、offline verification境界 | DD-21～DD-23 | 上位信頼境界は判断済み。鍵運用・具体暗号仕様は後続可能 |
| 6j | Artifact manifest schema evolution（BD-32承認済み） | BD-29、BD-31 | schema versioning、署名対象範囲、unknown required/optional fieldの扱い | DD-21～DD-23 | 上位互換規則は判断済み。encoding/struct詳細は後続可能 |
| 7a | CollectionModel／Grid virtualization（BD-33承認済み） | 要求仕様第4・6節、BD-01、R2～R4 | Collection/Selection object境界、stable IDs/change set、Gridの可視cell生成・再利用 | DD-02～DD-03、DD-12～DD-17、DD-21～DD-23 | 上位境界は判断済み。差分API・性能値は詳細設計可能 |
| 7b | SelectionModel semantics（BD-34承認済み） | BD-33、要求仕様第4～6節、BD-08 | single/multiple selection mode、current itemとkeyboard focusの区別、item削除時のstate更新 | DD-13～DD-17 | 上位意味論は判断済み。Control既定値やnavigation細則は後続可能 |
| 7c | Collection change batch atomicity（BD-35承認済み） | BD-33、R2～R4、BD-34 | Model整合性のbatch事前検証、全件適用／全件拒否、通知一貫性。独立入力欄の値／業務validationを相互に待たせない | DD-02～DD-03、DD-17～DD-18 | 上位判断済み。具体ABI・再入細則は後続可能 |

初期値型と別論点のCollection/Selectionは要求仕様第4節のCore必須範囲。値型として公開するかと、初回縦断版・要求を満たす製品版のどの時点で提供するかを区別し、後者を勝手に必須範囲から外さない。
