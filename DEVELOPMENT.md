# 開発環境

現段階では、各言語のビルド、C/C++環境probe、文書検査を行える土台を用意している。C++ Runtime最小wrapperとCMake install/export packageは実装済み。handleの各種wrapper・UIは後続実装で追加する。

## 基準と固定版

要求のminimumは[RQ-03-014](.docs/11_requirements/RQ-03_language-support.md#rq-03-014)、target範囲は[RQ-11-024](.docs/11_requirements/RQ-11_platform.md#rq-11-024)、実現方針は[build DD](.docs/13_detail_design/build/toolchain-packages.md) 1.1を根拠とする。

| 対象 | 今回の開発設定 | 管理ファイル |
| --- | --- | --- |
| Rust | 開発1.90.0、minimum検証1.85.0、edition2024、matching rust-analyzer component | rust-toolchain.toml / Cargo.toml / Cargo.lock |
| C/C++ | C11の環境probe、C++17、CMake3.28.4、Ninja package1.11.1.4 | CMakePresets.json / tools/requirements-dev.txt |
| C# | SDK10.0.401、net10.0、C#14 | global.json / FreeMvvm.csproj / packages.lock.json |
| Python | 標準CPython3.13、CIは3.13.12、repository内venv | pyproject.toml / tools/requirements-dev.txt |
| 検査 | Rust fmt/clippy、Python Ruff0.11.13、pytest8.3.5を導入 | tools/dev.py / tools/requirements-dev.txt |

Python開発依存はtransitiveもexact版へ固定している。現在のpinは開発用で、release artifactの署名済みprofile lockではない。Rust/C#/Pythonのpackage version0.0.0は未公開の骨格を示す。現在exportされるC ABI関数を宣言するheaderとC/C++ compile-only probeがあり、C++は`FreeMvvm::cpp`と`FreeMvvm::runtime`をexportするrelocatable CMake packageを持つ。正式release packageと他言語のnative loaderは未実装。

## Windowsで開始

MSVC x64 C++ Build ToolsとWindows SDK、rustup、標準CPython3.13、.NET SDK10.0.401を用意する。MSVC環境はvswhere/VsDevCmdから子processへ取り込むため、通常のPowerShellから実行できる。既存のOS PATHやrustup defaultを変更しない。

```powershell
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy --component rust-analyzer
rustup toolchain install 1.85.0 --profile minimal
py -3.13 -m venv .venv
.\.venv\Scripts\python.exe -m pip install -r tools/requirements-dev.txt
.\.venv\Scripts\python.exe tools/dev.py doctor
.\.venv\Scripts\python.exe tools/dev.py check
.\.venv\Scripts\python.exe tools/dev.py minimum
```

venvのactivateは不要。既定Python3.15や既存CMake RC版ではなく、venv内の実行ファイルを使う。非ASCII文字を含むcompiler環境はUTF-16で取り込み、環境変数一覧をログへ出さない。ユーザーが既にdeveloper shellを開いている場合はそのINCLUDE/LIBを使用する。

## Linuxで開始

標準CPython3.13、rustup、.NET SDK10.0.401、C/C++ compilerを事前導入する。今回のCI runnerはUbuntu24.04で、Debian12の正式profile検証とは別である。

```sh
rustup toolchain install 1.90.0 --profile minimal --component rustfmt --component clippy --component rust-analyzer
rustup toolchain install 1.85.0 --profile minimal
python3.13 -m venv .venv
.venv/bin/python -m pip install -r tools/requirements-dev.txt
.venv/bin/python tools/dev.py check
.venv/bin/python tools/dev.py minimum
```

Linux/ARMのsysroot・compiler・container digestを固定するrelease profileと、代表board実行は後続phase。今回のWindowsローカル結果からLinux/ARMの成功や正式supportを推定しない。

## コマンドと確認範囲

| コマンド | 内容 |
| --- | --- |
| `tools/dev.py doctor` | venv・Python・Rust/CMake/Ninja/.NETと依存pinの確認 |
| `tools/dev.py format` | Rust/Pythonのformat |
| `tools/dev.py abi-smoke` | Cargo C ABI native libraryをC consumerへlinkし、version queryとRuntime lifecycleを実行 |
| `tools/dev.py abi-command-smoke` | Link a C11 consumer against the native C ABI and verify Command start, UI-posted completion, state Properties, and close |
| `tools/dev.py abi-smoke-cpp` | Cargo C ABI native libraryをC++17 consumerへlinkし、version queryとRuntime lifecycleを実行 |
| `tools/dev.py cpp-runtime-test` | C++ Runtime fake-ABI unit testと`BUILD_TESTING=OFF`のwrapper build |
| `tools/dev.py cpp-package-test` | CMake packageをinstall・移動し、独立consumerのconfigure/link/Runtime smokeを実行 |
| `tools/dev.py check` | 文書リンク/表/fence、Python format/lint、Rust fmt/clippy/buildとtest harness compile、C/C++ CTest、relocated C++ package consumer、.NET locked restore/build、Python wheel build/install/import |
| `tools/dev.py minimum` | Rust1.85.0でworkspace/all-targetsをlocked check |
| `tools/dev.py python-input-test` / `python-input-consumer-test` | Python Input eventのfake ABI unitと公開consumer試験を実行する |
| `tools/dev.py python-window-peer-smoke-test` | 実機用の手動確認。Linux DRM/KMS上で背景色のvisible present、redraw、close/reopenを確認する。画面の色変化は人が観測する。`check`/CIでは実行しない |

`python-window-peer-smoke-test`はLinux DRM/display実機が準備できた場合にだけ明示実行する。赤・青のredraw後、close/reopenして緑のframeをpresentする。実行ログのpassだけでは可視表示の証拠にならず、画面の色変化を観測してprofile reportへ記録する。headless runnerのfailureを許容passにしない。背景色やC ABI値を検証するheadless-safe consumerは`check`/CIに含める。

実行は常に上記venvのPythonで行う。checkは初期化済みvenvへ開発wheelを導入し直す。C++ Runtimeとpackage consumerは製品unit/integration testとして実行し、他のpytest製品suiteは未作成である。C/C++環境probeはtoolchain確認で、UI全体の振る舞いの証明ではない。

結果はGit除外の `.local/verification/check.json` と `minimum.json` に保存する。git HEAD、dirty状態、入力hash、host、tool版、command、終了codeを記録する。未commitのローカル結果は、同じHEADだけを根拠にcommit済みsource/testの試験証拠と扱わない。

## 配置と依存方向

| 配置 | 責務 |
| --- | --- |
| crates/core-runtime | 内部Rust Core |
| crates/c-abi | Coreへ依存するversioned C ABI境界 |
| wrappers/rust/freemvvm-sys | 共通native artifactへ接続するraw FFI。Core依存なし |
| wrappers/rust/freemvvm | sysへ依存するsafe facade |
| wrappers/cpp / csharp / python | 各言語のthin facadeとbuild構成 |
| tools | CLI entrypoint、責務別command module、文書検査、unit catalog、環境probe、依存pin |

共通native runtimeをwrapperへ独立複製しない。現段階のC++ INTERFACE targetやPython importは、未実装の公開APIが使えることを意味しない。

### 開発CLIとunit catalog

`tools/dev.py`は既存名を保つCLI entrypointとして残し、commandの実処理は`tools/dev_support/commands/`へ責務別に置く。bootstrap、ABI、Linux Platform、Rust/C++/C#/Python wrapperごとにcommand registry・handler・report scopeを管理する。新しいcommandを追加するときは対象moduleとfocused testを更新し、entrypointへ個別処理を増やさない。

`tools/units.yaml`はschemaとshard patternを持つroot manifestで、unit record本体は`tools/units/`の責務・owner別shardに置く。新しいunitは対応するshardへ追加し、`python tools/dev_support/unit_catalog.py`で重複名・依存関係・case形式を検証する。wrapper言語ごとのunitは、それぞれの言語shardへ記録する。

## CIと次の実装単位

[bootstrap workflow](.github/workflows/bootstrap.yml)はpush/PRでWindows2022・Ubuntu24.04の全bootstrap gateを実行し、同じcheckout SHAの入力を使う。Actionsはfull SHAへpinし、read-only権限で起動する。未pushのローカル編集をCI実行済みとはしない。CIの合否は対象branch・commit SHAのActions runを確認する。

bootstrap全件を常時実行する初期構成であり、製品unitのpath mapping/依存逆閉包選択はまだ実装していない。次は[handle単位の試験設計](.docs/32_unit_test/core-runtime.md)をもとに実source/test/commandを台帳へ登録し、対応unit-testと一緒に実装する。各製品unit着手時にCI選択と同一revisionの証拠も追加する。[設計接続gap](.docs/32_unit_test/README.md)があるCommand進捗・TextBox通知等は、該当単位をreadyにする前に整合する。

利用した設定仕様: [Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html)、[CMake presets](https://cmake.org/cmake/help/v3.28/manual/cmake-presets.7.html)、[.NET global.json](https://learn.microsoft.com/en-us/dotnet/core/tools/global-json)。
