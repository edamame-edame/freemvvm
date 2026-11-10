# PresentationBackend選択API・試験対応レビュー案

- 文書版: 0.2
- 状態: 文書レビュー完了（利用者、2026-11-10）。実装は別計画の承認待ち・未着手
- 対象version: `0.0.1`
- 上位計画: [Platform別描画backend既定値・Linux framebuffer限定profile計画案](linux-framebuffer-limited-backend.md) 0.2（承認済み）
- 要求: [RQ-11 Platform](../11_requirements/RQ-11_platform.md) RQ-11-025
- 根拠設計: [Platform内部レイヤー](../13_detail_design/platform/internal-layers.md) 2.7（承認済み）、[Window・基本Control](../13_detail_design/ui/controls.md)、[共通C ABI](../13_detail_design/runtime-objects/abi.md)
- 試験根拠: [要求-テストケース対応表](../30_release_test/requirement-test-matrix.md)、[要求別試験assertion](../30_release_test/requirements-test-assertions.md)、[UI unit-test設計](../32_unit_test/ui.md)

## 1. 目的と範囲

承認済みRQ-11-025とDD 2.7の「profile既定値、利用者の明示選択、非互換・初期化失敗時のfailure、暗黙fallback禁止」を、Window API、C ABI、および試験で観測できる契約へ具体化する。

本書は提案であり、既存設計本文を承認済みのまま書き換えない。利用者レビュー後、承認された範囲をPlatform DD、UI Control DD、共通C ABI DD、要求-test対応文書へ文書単位で反映する。実装開始はその反映後の別判断とする。

## 2. 推奨するAPI境界

描画SurfaceとRendererの互換性を一つにした`PresentationBackendDescriptor`を、UI構築時にWindowへ渡す設定値から解決する。選択はWindow objectの寿命中不変とし、実行中の変更・close後の差し替えを提供しない。surface/scene資源の初期化自体は従来どおり`open`時に行う。要求IDは利用者向けAPIに持ち込まず、公開されたbackend identityを指定値に使う。

### Window契約案

Window生成時にoptionalな`PresentationBackendId`を指定できる。未指定ならprofile defaultを使う。設定値はWindow生成時にCore-owned値へcopyし、そのWindow objectを破棄するまで変更できない。

| 操作 | 提案する動作 |
|---|---|
| `Window.create(runtime, presentation_backend=None)` | UI構築時にbackendを選ぶ。未指定はprofile default。指定値をUTF-8でcopyし、Window objectの設定として固定する |
| `open()` | 固定された選択値とprofile・surface・scene capabilityを照合し、descriptorを一つ使ってsurface/rendererを初期化する |
| `close()` / 再open | native resourcesを解放・再作成する。Windowのbackend選択値は変えられず、同じ選択を再利用する |
| 別backendへ変更 | APIを設けない。別選択でUIを作る場合はWindowを破棄して作り直す |

既定backendの選択と明示backendの選択は同じdescriptor registryを通す。未指定時に利用可能backendを実行時heuristicで探索・比較しない。指定identityが未知、profile/surface/sceneに非互換、または初期化失敗の場合は`open`を失敗させ、Core Window・contentを保持する。別factory/backendを試さない。

Window生成時にIDの文字列表現を検査し、profile registryに照合する。scene capabilityやnative surfaceの実体を使った最終確認、およびdevice初期化は`open`時に行い、失敗はowned errorとして返す。診断にはrequested ID、選択可否、失敗段階を含めるが、利用可能候補の列挙APIはこの提案では追加しない。選択値がinvalidでも別backendへ自動置換しない。

### identityと公開選択肢

identityはUTF-8の安定したmachine keyとし、実装型名、OS handle、GPU indexを含めない。runtime artifactのprofile資料に、選択可能ID、対応surface、依存条件を掲載する。利用可能でない候補を列挙・受理しない。

現行profile向けのID文字列はまだ設計されていないため、次の例はレビュー用placeholderであり、互換性保証・実装済み選択肢ではない。

| profile | identity例 | 現在の範囲 |
|---|---|---|
| Windows 11 x64 | `windows.win32.d2d` | 既存Win32 + Direct2D/DirectWrite defaultの識別案 |
| Ubuntu/Debian Linux direct-display | `linux.drm-kms.gles3` | 既存DRM/KMS + GBM/EGL/GLES defaultの識別案。各profileの適合・証拠境界はDD 2.7に従う |
| Linux framebuffer限定 | `linux.framebuffer.fbdev`等 | sink方式が未承認のためidentityを確定しない。登録後にだけ公開する |

今の各artifactで代替backendが登録されているとは主張しない。将来一つのprofileに複数backendを同梱した場合にも、このWindow APIで選択できるようにする。

### artifactへのbackend収録とlink方式

- backendをcompile/linkする境界は言語wrapperではなく共通native C ABI artifactである。現行CargoではLinux/Windows adapterをtarget-specific dependencyに分ける一方、各言語wrapperは同じnative ABI artifactを使う設計になっている。したがってC++/C#/Rust wrapperだけをbuildしても、配布済みnative artifact内のbackend codeは除去できない。
- 非Python向けに「使わないbackendをcompile時に除外」する場合、`freemvvm-c-abi`をbuildするfeature/profile設定でbackend implementation/dependencyを選び、生成したnative artifactを各wrapper packageへ接続する。不要backendを含めないdefault-only artifactを作れる。将来、同一OS向け複数候補を一つのartifactで選択可能にする場合は、その候補だけを有効にしたmulti-backend feature setを別途buildし、その分のcode/data/native dependencyが増える。
- build/link構成はtarget OSごとに分ける。Windows用とLinux用の全実装を一つの通常artifactへ常時含める前提にしない。現行構成もC ABIのtarget-specific dependencyでOSごとにadapter crateを分けている。
- 一つのartifactを4言語packageで共有する現行承認済みpackage設計では、backend feature setも4言語共通になる。利用者がUI生成時に選べるのは、そのnative artifactへcompile/linkされregistry登録済みの候補だけとし、未収録backendは明示failureにする。
- Pythonの通常実行はcompile-time backend pruningを要求せず、現行どおりnative runtimeをloadする。PyInstallerでbundleする場合は選択したnative backend binary/pluginをbinary resourceとして収録できる方針を提案する。PyInstallerは`--add-binary`/specの`binaries`でnative binaryをbundleでき、frozen processは`sys._MEIPASS`でbundle directoryを得られる（[PyInstaller binary収録](https://pyinstaller.org/en/stable/usage.html)、[PyInstaller runtime path](https://pyinstaller.org/en/stable/runtime-information.html)）。現行Python loaderはsystem library lookupが主体で、PyInstaller bundle pathの解決は未実装。
- PyInstallerが一つのbackend binaryだけを収録する場合はbundle sizeを抑えられるが、実行時選択肢はその一つになる。UI構築時に複数候補から選ばせる場合、候補をすべてbundleへ含めるか、別途install/pathで解決可能にする。plugin方式を選ぶならbackendごとのshared library ABI・依存解決・所有/lifecycleを設計する。pluginを使う場合はRuntimeが存在する状態でWindow選択を受け、`open`前に選んだpluginを一度loadして保持する。backend別の完全なC ABI runtime binaryを収録する場合はCoreが重複し、どのvariantをC ABI loaderが使うかRuntime生成前に選択する別設計が必要になる。
- Pythonにruntime resourceを追加するpackage形は、「4言語が同じnative artifact bytesを使う」という既存設計を維持できるplugin方式か、Python専用artifact variantを使うかで差分が異なる。どちらを採るかは未決であり、この文書だけで承認済みにしない。
- Pythonの複数backend選択用には、共有Core/C ABI runtimeは一つのまま、backend実装を`freemvvm_backend_<id>`のようなshared libraryへ分ける案を検討できる。backend libraryはCoreのRust trait objectを直接exportせず、versioned C ABI descriptor/function tableでsurface/renderer factoryを登録する。C ABI artifactはpluginを指定pathから一度loadし、profile・surface・capabilityを照合してからWindowへ接続する。plugin handleは使用中のWindow/rendererより長く保持し、Runtime shutdown後にだけ解放する。
- このplugin案はCoreの重複を避けられる。backend shared libraryをPyInstaller bundleへ同梱する場合は`--add-binary`で配置したbinary分だけ配布bundle総量が増えるが、既存C ABI runtimeのbinary sizeにはbackend codeを加えない。選択済み一つだけを同梱する、backend packageを別途installする、bundle外の明示pathからloadする構成では、すべての候補をapp bundleへ入れる必要はない。アプリ実行時に複数候補から選ばせるなら、候補libraryはbundle内またはruntime環境のいずれかに利用可能な状態で存在する必要がある。plugin ABI/version negotiation、OS loader dependency/RPATH、署名/信頼、error mapping、登録順序、thread/lifecycle、crash境界、test matrixを新規設計する必要がある。
- 「static link」ではbackend implementation codeが既存native binaryへ入る。一方、dynamic shared library (`.dll`/`.so`) は実装を別fileに保ち、OS loaderまたは明示load APIで解決する。runtime choiceを可能にするには単純なlink-time importだけでなく、安定したC ABI function tableを通じて候補libraryを選択・loadする境界が必要になる。Windows import `.lib`はDLLの利用stubをlinkするもので、DLL本体とは別物である。
- 代替案としてbackendごとに完全なC ABI runtime variantを分ければplugin ABIは不要だが、Core/ABI codeを各variantへ重複して含み、複数variantをPyInstallerへ入れるとその分bundleが膨らむ。Python loaderはRuntime生成前にvariant一つを選ぶ必要があり、Windowごとに異なるbackendを選ぶ案とは別の意味になる。
- 暫定推奨は、Python bundleで複数backendをWindow構築時に選ばせる必要があるなら「共通Core + backend plugin library」を設計候補として優先して検討すること。ただしcross-library C ABI plugin契約は現行Core SPIから新規に作る設計であり、review approvalまでは採用確定と扱わない。1候補だけでよければbackend-specific full runtime artifactの方が単純。
- Python向けpluginをソースからbuildする配布経路として、source treeからsdistを作り、PEP 517 build backendでbackend-specific platform wheelを生成する案がある。既存pure-Python `freemvvm` wrapper wheelは維持し、native plugin wheelは別package/artifactにすることでPython ABIとPlatform plugin ABIを分ける。`setup.py`を直接install実行するのでなく、`pyproject.toml`のbuild-system frontend/backend経由でplugin build hookを動かす。
- sdistからconsumer環境でwheelをbuildする場合、Rust/C compiler、target SDK、Platformのdevelopment headers/import libraryが必要になる。実行時にはshared dependency本体がsystem runtimeにあるか、別binary package/PyInstaller bundleに含まれる必要がある。開発用package（headers/link metadata）と実行用shared libraryを混同しない。
- source buildを許せば事前配布するbackend binaryを省けるが、consumerごとのbuild artifactとなり、toolchain・sysroot・native dependency・build flagsを記録して独立identity/digestで扱う必要がある。通常利用はCIで作成・検証済みplatform wheelを配り、sdist buildは明示的なsource-build経路にする案を推奨する。これならpip installでは事前build済みwheelを展開し、一般利用者にcompilerを要求しない。
- `.whl`はbuilt distributionであり、compiled pluginを含めるとplatform-specific wheelになる。source codeを収録・管理するartifactはsdist（通常`.tar.gz`）である。PyInstallerへwheelのplugin binaryを同梱するならbinary resourceとしてコピーし、system dependencyへ依存する場合はPyInstaller targetで実ロード/依存解決まで確認する。
- Rust adapter crateをC ABI artifactへ含めることと、EGL等OS提供native libraryをdynamic linkまたはOS loader経由で使うことは別の境界である。OS loaderによる依存解決やAPI table取得は通常process起動時またはlibrary/API初期化時に行い、描画フレームごとにlibraryをload/unloadしない。個々のnative関数呼出しにloader stub/function-table経由の間接呼出しが残る場合はある。
- backend identityはWindow生成時に一度resolveする。render/present時にregistryを走査したり、backendを選び直したりしない。既存SPIのtrait-object/関数table呼出しが生む間接呼出しcostはあり得るが、静的link化だけでUI更新が速くなるとは断定しない。描画性能はtargetの代表sceneでframe time/throughputを測って判断する。

したがって、ユーザーの方針案は「Python以外はnative artifactのbuild時に不要backendを除外し、Pythonの通常実行は現状維持、PyInstallerではbackend別shared libraryをbundle resourceとして分けられる」と整理する。Pythonで複数backendをUI構築時に選ばせる場合は、共通Core + versioned C ABI pluginの案を優先してレビューし、1候補限定ならfull runtime variantも比較する。load/resolveは一度だけ行い、full runtime variantならRuntime生成前、backend pluginならWindow open前に完了させる。frameごとのplugin切替はしない。Python固有artifact/pluginの採用にはpackage設計のレビューが必要になる。

## 3. 共通C ABI提案

C ABIは既存のUTF-8 pointer/length、opaque handle、status、owned error規則を使い、既存のdefault選択用create symbolとの互換を保つ。以下は宣言レビュー用で、配布headerや実装済みsymbolではない。

```c
ui_status ui_abi_1_window_create_with_presentation_backend(
    ui_handle runtime,
    const uint8_t *backend_id,
    size_t backend_id_len,
    ui_handle *out_window,
    ui_handle *out_error);
```

- 既存`window_create(runtime, ...)`はprofile defaultを選ぶ。明示値用に新しいcreate symbolを加え、既存symbolの引数は変更しない。
- create操作はUI thread限定の同期操作とする。検査後にUTF-8をCore-owned値へcopyし、引数bufferは戻り後に保持しない。
- 必須null/長さ不整合、空ID、不正UTF-8、無効Runtime、wrong threadは既存共通検査順/status/error契約で拒否する。
- IDのsurface/scene適合性は`window_open`で最終確認する。failure時は既存Platform error domainへidentityと失敗理由を載せ、新しいstatus/error数値をこの案だけで割り当てない。
- C++/C#/Rust/Python wrapperは生成時のoptional backend設定を提供し、生成後のset/reset property/methodは持たない。各言語の型とerror idiomは各wrapper文書で承認する。

wrapper形の例: Rust `Window::create(ui, Option<&str>)`、C++ `Window::create(runtime, std::optional<std::string_view>)`、C# `UiWindow.TryCreate(runtime, string? presentationBackend = null)`、Python `Window.create(runtime, *, presentation_backend: str | None = None)`。既存default-only overloadを維持するかoptions objectを導入するかはwrapperごとのAPI一貫性レビューで決める。

Pythonではbackendをmutable propertyにしない。`Window.create(..., presentation_backend=...)`のkeyword-only引数からnative create callへUTF-8 bytesを渡し、Coreが同期copyした後はPython wrapper側に設定値を保持しない。これにより後から代入される属性と実native Windowの選択が食い違わず、GCや一時bytesの寿命にも依存しない。

この初版ではbackend一覧取得APIを追加しない。profile資料がartifactごとの選択可能IDを公開する。実行中processから動的catalog取得が必要になった場合は、catalog ownership/version/filter規則を含む独立要求・設計として扱う。

## 4. 試験対応案

### 新しい論理case

`TC-BACKEND-SELECT-01`を一つ追加し、fake `PresentationBackendDescriptor` registryとfake surface/rendererで決定的に検証する。TC-PLAT-01の一般的な責務境界検査とは別に、利用者指定値からopen結果までの選択oracleを持つ。

期待する観測:

1. ID未指定でWindowを生成した場合、profile default descriptorだけを選び、そのidentityが選択結果/診断に記録される。
2. 登録済み互換IDを生成時に指定すると、そのdescriptorだけが選ばれる。
3. 未登録IDはcreateで拒否する。surface非互換、scene capability不足はopen failureとなり、Window/contentを維持する。
4. 明示backend初期化失敗時に別factory/backendを呼び出さない。
5. close/reopen後も同じWindowのbackend選択は不変で同じidentityを再利用する。別backend選択はWindow再作成後にだけ可能。
6. create時のUTF-8 copyにより呼出元bufferの変更/解放が保存値を変えない。invalid inputはWindowを生成しない。

このcaseはfake境界の選択規則を検証し、Windows/Linuxの可視描画、GPU driver、backend profile passを代替しない。profile別の選択defaultとvisible-presentはTC-TARGET-WIN-01、TC-TARGET-LNX-01、TC-TARGET-LNX-VM-01等の環境証拠に記録する。

### unit-test候補

| test ID案 | 検証境界 | ケースとの関係 |
|---|---|---|
| `UT-CORE-BACKEND-SELECTION-001` | Core Window生成時のoptional ID、profile registry照合、open時surface/scene resolve、Window寿命中の設定固定、failure atomicity、no-fallbackをfake registryで検証 | `TC-BACKEND-SELECT-01`の主oracle |
| `UT-ABI-WINDOW-BACKEND-001` | C ABIのcreate pointer/length、UTF-8 copy、status/owned-error mapping、default-only createとの互換 | 同caseのABI表現境界 |
| wrapper個別case | 各wrapper生成時の任意引数、例外/Result/error、Python keyword-only引数とmutable propertyを持たない境界 | 共通caseをwrapper parityで補完。実装言語ごとにunit manifestへ登録 |

同一実装revisionのテスト設計・manifest・実装を一緒に更新する。unit IDとrelease case IDは試験管理用であり、設計項目IDや設計項目単位の承認には使わない。

### RQ-11-025 assertion案

- **前提・操作・観測:** profile別の登録descriptor registryを用意し、default、生成時の明示compatible、unknown、incompatible surface/capability、factory init failure、close/reopenをそれぞれ実行する。各fixtureでfactory呼出記録、create/open結果、Window/content/generation、selected identityとdiagnosticを観測する。
- **期待結果:** 未指定ならprofileに結び付いたdefaultのみを決定的に選ぶこと。生成時に明示選択なら同じidentityを満たすdescriptorだけを選ぶこと。未登録・非互換・初期化失敗は明示failureとなり、別backend factoryを呼ばず、Core状態を保持すること。同じWindowはclose/reopen後も同じ選択を使うこと。
- **後始末:** 失敗途中に確保したsurface/rendererを各ownerが一度だけ解放し、fake registry、diagnostic capture、Window/Runtimeを破棄する。
- **実行状態:** 未実装・未実行。fake test合格とprofile別可視描画記録は別証拠とする。

## 5. レビューで決める点

1. selectorをUI構築時にWindow生成引数として渡し、Window objectの寿命中は変更不可とする。
2. 既存のdefault-only C ABI createを維持し、明示選択用の`window_create_with_presentation_backend`を追加する。
3. Python wrapperはkeyword-only生成引数にし、mutable property/set/reset APIを持たない。
4. 初版には動的catalog列挙APIを追加せず、package/profile資料で選択肢を提示する。
5. `TC-BACKEND-SELECT-01`とunit-test ID案を、RQ-11-025の試験対応案として採用する。
6. 現時点で代替backendが各profileに存在するとは主張せず、placeholder IDは登録時に確定する。
7. 非Python向けbackend除外はwrapper compileではなくnative C ABI artifact build featureで行う。Python通常実行は現行native loaderを維持し、PyInstallerではbackend binary/pluginをbundle resourceにできるようにする。
8. PythonのPyInstaller bundleでbackendをUI構築時選択にする場合、共通Core + backend別plugin libraryを使う案を優先してreviewする。plugin ABIは新規設計する。pluginはCI-built platform wheelを通常経路、PEP 517経由のsdist source buildを開発/明示経路にする案とし、system development/runtime dependencyを分ける。1候補限定ならfull runtime variantとのサイズ・複雑さを比較する。
9. backend identityはWindow構築時に固定し、pluginは初回open前に一度だけload/resolveして保持する。frameごとのload/registry scanをしない。performance benefitは静的linkだけから推定せず測定で判断する。

## 6. 承認された文書と後続実装計画

以下の文書改訂版は利用者の「ok」により全文承認済み。旧承認版と実装・試験状態はそれぞれ履歴と実行台帳で管理する。

| 分野 | 承認済み文書 | 反映範囲 |
| --- | --- | --- |
| 基本設計 | [Platform抽象化 BD 1.3](../12_design/platform/abstraction.md) | Window構築時のselector設定からCore/Platform registry/surfaceへの責務flow |
| Platform DD | [internal layers 2.8](../13_detail_design/platform/internal-layers.md) | identity固定、default解決、互換性判定、open failure、close/reopen、no-fallback |
| UI Control DD | [controls 1.7](../13_detail_design/ui/controls.md) | `Window.create(..., presentation_backend=None)`の生成時設定 |
| C ABI DD | [ABI 1.7](../13_detail_design/runtime-objects/abi.md) | default create互換と明示selector用create symbol、UTF-8 buffer ownership |
| package DD | [toolchain packages 1.2](../13_detail_design/build/toolchain-packages.md) | native artifact feature set、Python plugin/sdist/wheel/PyInstaller配布の選択肢と未決境界 |
| release test | [case matrix 1.0](../30_release_test/requirement-test-matrix.md)、[assertions 1.4](../30_release_test/requirements-test-assertions.md) | `TC-BACKEND-SELECT-01`とRQ-11-025のcase/assertion/reverse mapping |
| unit test | [UI unit test 1.9](../32_unit_test/ui.md) | fake registry Core oracle、C ABI UTF-8 ownership、wrapper parity |

後続の実装範囲とbranch/unit/test/CI gateは[実装計画](backend-selector-implementation.md)へ記録した。Python plugin ABI・plugin loading lifecycle・backend artifact package方式の選択は未決として、その計画内に独立設計・承認gateを設けている。

| 日付 | 版 | 結果 |
| --- | --- | --- |
| 2026-11-10 | 0.2 | 利用者の「ok」により0.1提案を反映した設計・test文書群を承認。実装計画は別途レビュー待ち |
