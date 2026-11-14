# Linux Wayland / Window初期サイズ API・build設計レビュー案

- 案版: 0.1
- 状態: API/build方式と7つの正本DD全文は利用者承認済み（2026-11-14）。実装・依存追加・試験文書変更は未着手
- 親計画: [Linux Wayland Window実装計画](linux-wayland-window-implementation.md) 0.1（承認済み）
- 対象要求: [RQ-09-076](../11_requirements/RQ-09_ui-scope.md#rq-09-076)、[RQ-11-003/014/015/024](../11_requirements/RQ-11_platform.md)、[RQ-12-010](../11_requirements/RQ-12_build-distribution.md#rq-12-010)
- 承認済み設計: [UI controls DD 1.8](../13_detail_design/ui/controls.md)、[Platform DD 2.9](../13_detail_design/platform/internal-layers.md)、[ABI DD 1.7](../13_detail_design/runtime-objects/abi.md)
- 参考: [C++](../13_detail_design/wrappers/cpp.md)、[C#](../13_detail_design/wrappers/csharp.md)、[Python](../13_detail_design/wrappers/python.md)、[Rust](../13_detail_design/wrappers/rust.md)、[build/package DD](../13_detail_design/build/toolchain-packages.md)

## この案の位置づけ（承認済み）

RQ-09-076は初期寸法の指定・省略時800×600 DIP・無効値拒否を定め、UI/Platform DDは希望値とOS/compositorが返す実寸の境界を定めている。本案のAPI/ABIとbuild profileへの写像は利用者に承認され、列挙した7つの詳細設計正本へ反映済みである。実装開始前に、別途実装計画0.2のレビュー・承認を受ける。

記載するC宣言・wrapper API・feature名・Cargo feature名はレビュー候補であり、配布header、実装済みAPI、確定依存ではない。

## 公開API案

### C ABI

既存の`ui_abi_1_window_create`と`ui_abi_1_window_create_with_presentation_backend`は維持し、新しいsymbolを一つ加える。

```c
ui_status ui_abi_1_window_create_with_initial_client_size(
    ui_handle runtime,
    const uint8_t *presentation_backend_utf8,
    uint64_t presentation_backend_byte_length,
    double width_dip,
    double height_dip,
    ui_handle *out_window,
    ui_handle *out_error);
```

- このsymbolは初期サイズを必ず明示する。backend指定は任意とし、`presentation_backend_utf8 == NULL && byte_length == 0`はprofile defaultを意味する。backendを明示する場合は有効なUTF-8かつ長さ1以上とする。pointer/lengthの片側だけがnull/zero、または空backend IDは`UI_INVALID_ARGUMENT`。
- 幅・高さはdoubleでDIPを表し、両方が有限かつ正の場合だけ受理する。NaN、±infinity、0、負数はWindowを生成せず`UI_INVALID_ARGUMENT`とする。
- 成功時にbackend identityと初期希望寸法をCore-owned Window設定へcopyする。入力pointerを保持しない。close/reopenでも同じ値を使う。Window open後のLayoutにはpeerから返る実client areaを使う。
- 必須outputを既存ABI規則に従って検査し、失敗時は`out_window = 0`。error outputは既存の任意owned handle規則に従う。部分Windowを公開しない。
- `has_feature(10)`を`core.ui.window.initial_client_size.v1`へ割り当てる案。feature 10はこのsymbolと契約が完全に提供される場合のみtrueとする。既存feature 8/9の範囲・意味は変更しない。ABI minorは互換追加として既存versioning規則に沿って更新する。
- wrapperは初期サイズが指定されたときにfeature 10を確認する。未対応runtimeなら`UI_UNSUPPORTED`相当の既存error表現にし、Windowを生成しない。初期サイズ未指定の既存呼出しは既存symbolを使う。

この一symbolで次の組み合わせを表せる。

| wrapperの引数 | 呼ぶC ABI | backend | 初期サイズ |
|---|---|---|---|
| backendなし、sizeなし | `window_create` | profile default | Core default 800×600 DIP |
| backendあり、sizeなし | `window_create_with_presentation_backend` | 指定値 | Core default 800×600 DIP |
| backendなし、sizeあり | `window_create_with_initial_client_size` | profile default | 指定値 |
| backendあり、sizeあり | `window_create_with_initial_client_size` | 指定値 | 指定値 |

### 言語wrapper

既存の簡便な生成methodと引数は保つ。追加するoption APIはC ABIの検証を再実装せず、指定値をmarshalしてCoreへ委譲する。wrapper内に別のWindowサイズ既定値は持たない。

| 言語 | 追加API候補 | 境界 |
|---|---|---|
| Rust | `WindowOptions<'a> { presentation_backend: Option<&'a str>, initial_client_size: Option<WindowInitialClientSize> }`と`Window::create_with_options(ui, options)`。`WindowInitialClientSize { width_dip: f64, height_dip: f64 }`はCopy可能な値型 | 既存`create`と`create_with_presentation_backend`を維持。sizeなしは既存ABIを使い、sizeありはfeature 10経路へ送る。UI thread/Runtime条件は既存`UiContext`に従う |
| C++ | `WindowCreateOptions`と`WindowInitialClientSize{double width_dip, height_dip}`を追加し、`Window::create(runtime, options)`を提供 | optionsはcall中だけ借用する`std::string_view`と`std::optional`値を保持。既存`create(runtime)`/backend overloadを維持し、statusは既存`Result<Window>`へ写す |
| C# | immutable `UiWindowCreateOptions`と`WindowInitialClientSize(double WidthDip, double HeightDip)`を追加し、`UiWindow.TryCreate(runtime, options)`を提供 | 既存`TryCreate`/`TryCreateWithPresentationBackend`を維持。status/errorは既存`UiResult<T>`へ写す。P/InvokeのdoubleはC ABI `double`と対応させる |
| Python | `Window.create(runtime, presentation_backend=None, *, initial_client_size=None)`を提案。sizeは2要素の`(width_dip, height_dip)`として受ける | 既存の第2 positional/keyword `presentation_backend`を保ち、新しいsize引数だけkeyword-onlyにする。tuple長、要素数値型、有限/正値を検査し、bool・文字列・その他の型は拒否して`UiError`へ写す。最終の型受理規則はPython DDへ記録 |

既存native runtimeにfeature 10がない場合の扱いは、sizeを指定した呼出しに限りunsupportedとして返す。C++/Pythonのsymbol tableやC#のimport解決でfeature 10 symbolを必須化して既存constructorの利用まで失敗させない。Rust `-sys`は外部symbolへ直接linkするため、対応するnative artifactを同じ版で配布し、wrapperの最低runtime版を明示する。古いnative libraryと新しいRust wrapperを組み合わせる保証は追加しない。

## Linux backend / build profile案

### backend選択と依存分離

- 既存backend identity `linux.drm-gbm-egl-gles`を維持し、承認済みPlatform DD 2.9で使っているWayland identity `linux-wayland-egl`を引き継ぐ。WaylandとDRM/KMSは別build profile/artifactとし、1つのartifactで両方の実装を有効にしない。
- `freemvvm-platform-linux`に`drm-gbm-egl-gles`と`wayland-egl-gles`のCargo featureを作り、backend専用依存をoptionalにする。`freemvvm-c-abi`側の既存DRM featureは従来どおりdefaultを維持し、新しい`presentation-backend-linux-wayland-egl-gles` featureからWayland featureだけを有効にする。
- Wayland artifactは`--no-default-features --features presentation-backend-linux-wayland-egl-gles`相当でビルドする。DRMとWayland両featureの同時選択はbuild時に診断して拒否する。通常のWindow実行時backend hot-swap、backend fallback、plugin library探索は導入しない。
- `glow`/EGL等、両backendが本当に共有するRust/runtime依存だけ共通化し、DRM/GBMとWayland client/protocol/EGL-window依存はfeature graph上で分離する。`cargo tree`・artifact metadataでWayland単独profileからDRM専用crateが除外されることを検証する。
- Rust crateとfeatureはCargo.lockで固定し、Rust 1.85を最低compiler gateとする。Wayland client/protocol/egl crateのversionはlock graph全体のMSRV・license・native link条件を確認した後に選ぶ。現時点で特定versionは確定しない。

### runtime profileとnative library

`freemvvm-c-abi/build.rs`のartifact profile許可表を、承認済みRQ-11/12と一致させる。Wayland desktopは現DRM profile名を再利用せず、次のprofile identityを候補とする。

| `FREEMVVM_TARGET_PROFILE`候補 | 対象OS/runtime | 既存DRM/KMS profileとの関係 |
|---|---|---|
| `debian-13-vm-wayland-x64` | Debian 13 x64 / Wayland desktop session | 新規profile |
| `debian-12-vm-wayland-x64` | Debian 12 x64 / Wayland desktop session | `debian-12-vm-x64`と分離 |
| `ubuntu-24.04-wayland-x64` | Ubuntu 24.04 x64 / Wayland desktop session | `ubuntu-24.04-x64`と分離 |
| `ubuntu-26.04-wayland-x64` | Ubuntu 26.04 x64 / Wayland desktop session | 新規profile |

特定profile用artifactは`FREEMVVM_TARGET_PROFILE`で明示し、generic Linux x64 buildをいずれかのOSでのruntime evidenceとして扱わない。各VM/OSのrelease記録にはexact OS/kernel、compositor/session、hypervisor/virtual GPU、driver、backend、artifact SHAを併記する。build.rsはWayland featureと上記Wayland profileだけの組合せを許可する。

開発時には`libwayland-dev`、EGL/GLES headers、pkg-config相当を確認し、実行時にはWayland client/EGL/GLES system shared librariesをtarget OSが提供することをpackage metadataに示す。Wayland backendのRust codeをartifactへ選択的に含めることと、OSのshared librariesをアプリへbundleするかは別項目として記録する。通常のWayland desktop sessionが持つsystem librariesを前提にし、特定distributionのpackage versionをここで固定しない。

Pythonは現在の共通native artifact + pure-Python wrapper packaging方針を維持する。Wayland対応native runtimeをprofile別にbuildしてwheel/既存distribution経路から選ぶ。backend別の追加plugin、全backend libraryのPython process内同時load、PyInstaller専用package方式はこの案で新設しない。

## 設計適用対象

この案が承認された場合、次の設計正本へ提案を適用し、各文書の全体レビュー・承認を得る。現行の承認baselineは承認履歴へ維持する。

| 文書 | 改訂案 | 適用内容 |
|---|---:|---|
| [共通C ABI](../13_detail_design/runtime-objects/abi.md) 1.8改訂案 | 1.7承認済み | Window初期サイズsymbol、feature 10、validation/error/compatibility、API表 |
| [Platform internal layers](../13_detail_design/platform/internal-layers.md) 3.0改訂案 | 2.9承認済み | Wayland identity/build profileと依存分離、Linux artifact metadata |
| [build/toolchain packages](../13_detail_design/build/toolchain-packages.md) 1.3改訂案 | 1.2承認済み | Linux target profiles、Cargo feature選択、native build/runtime library・Python artifact境界 |
| [Rust wrapper](../13_detail_design/wrappers/rust.md) 1.9改訂案 | 1.8承認済み | `WindowOptions`とsize値型、native runtime minimum |
| [C++ wrapper](../13_detail_design/wrappers/cpp.md) 1.6改訂案 | 1.5承認済み | options値型と`Result` facade |
| [C# wrapper](../13_detail_design/wrappers/csharp.md) 1.7改訂案 | 1.6承認済み | options/size値型と`UiResult` facade |
| [Python wrapper](../13_detail_design/wrappers/python.md) 2.3改訂案 | 2.2承認済み | keyword-only size tuple、strict validation/native feature gate |

UI DD 1.8とPlatform DD 2.9の承認済み外部意味は変更しない。build/runtime依存の実地確認で別の前提が必要になった場合は、上表の詳細設計案へ混ぜず、その差分を先に提示する。

## 受入確認案

- C ABI header C/C++ compile probeと既存symbol compatibilityを保ち、新しいsymbol/feature 10の完全契約を確認する。
- Core/ABI fake testsでdefault・backend・sizeの4組合せ、invalid dimensions/UTF-8/backend、out handle 0、partial Windowなし、close/reopen時の希望値保持、peer実寸優先を確認する。
- Rust/C++/C#/Pythonのpublic consumer testで4組合せとinvalid inputを同じ意味で確認する。旧C ABI callerは新runtimeの既存symbolを継続利用できる。wrapper packageは配布するnative artifactとのminimum versionを明示し、feature未提供時のsize指定はunsupportedにする。
- Wayland-only Cargo graph/buildとDRM-only graph/buildを別々に確認し、各profile artifact metadataにbackend identityとprofileが一つだけ記録される。minimum Rust 1.85でlockされたWayland dependency graphをcheckする。
- VM visible testは親実装計画のDebian 13 → Debian 12 → Ubuntu 24.04 → Ubuntu 26.04順で実行し、window present、configure resize/redraw、close/reopenを各profileの証拠として個別に記録する。

## レビューしてほしい内容

1. 初期サイズを指定する新C ABI symbolを一つ追加し、backendを`NULL/0`でprofile defaultにする案。
2. feature ID 10 `core.ui.window.initial_client_size.v1`を新設し、wrapperはsize指定時だけ必要featureを確認する案。
3. 4 wrapperのoption API形状と、既存生成API/default動作を維持する案。
4. Wayland/DRMのCargo featureとartifactを分離し、Wayland desktop artifactで未使用DRM依存を除外する案。
5. Debian 13/12、Ubuntu 24.04/26.04をprofile metadataへ明示し、Pythonは共通native artifact配布方針を使う案。

## 承認境界

- API/build方式および7つの正本DD全文は利用者承認済み。設計変更が必要になった場合は、影響する文書全体の再レビューを受ける。
- 実装計画0.2の承認前にunit catalog/test文書変更、依存追加、source変更、実装、commit/pushを行わない。

## レビュー履歴

| 日付 | 案版 | 状態 | 備考 |
|---|---:|---|---|
| 2026-11-14 | 0.1 | 利用者レビュー待ち | 承認済みRQ/UI/Platform契約をC ABI、4 wrapper、build profileへ写す具体案。正本適用・実装・依存追加は未着手 |
| 2026-11-14 | 0.1 | API/build方式承認 | 利用者の「ok」により提案の方向性を採用。7つの正本DDへ改訂案を適用し、文書全体レビュー待ち |
| 2026-11-14 | 0.1 | 7つの正本DD全文承認 | 利用者の「ok」によりABI 1.8、Platform 3.0、build 1.3、Rust 1.9、C++ 1.6、C# 1.7、Python 2.3の各文書全体を承認 |
