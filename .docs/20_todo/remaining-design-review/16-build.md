# 未設計レビュー / 最低版・target matrix・toolchain lock・保守

- 文書版: 0.2
- 状態: 全判断承認済み・正本適用済み（2026-10-17）
- 判断ID: REM-BUILD-01～09
- 入口: [全件索引](index.md)
- 根拠RQ: [RQ-11-001](../../11_requirements/RQ-11_platform.md#rq-11-001)、[RQ-11-002](../../11_requirements/RQ-11_platform.md#rq-11-002)、[RQ-11-003](../../11_requirements/RQ-11_platform.md#rq-11-003)、[RQ-11-004](../../11_requirements/RQ-11_platform.md#rq-11-004)、[RQ-11-021](../../11_requirements/RQ-11_platform.md#rq-11-021)、[RQ-12-009](../../11_requirements/RQ-12_build-distribution.md#rq-12-009)、[RQ-12-010](../../11_requirements/RQ-12_build-distribution.md#rq-12-010)、[RQ-12-017](../../11_requirements/RQ-12_build-distribution.md#rq-12-017)、[RQ-12-018](../../11_requirements/RQ-12_build-distribution.md#rq-12-018)、[RQ-12-019](../../11_requirements/RQ-12_build-distribution.md#rq-12-019)、[RQ-12-020](../../11_requirements/RQ-12_build-distribution.md#rq-12-020)
- 具体化するBD: [BD-12](../../12_design/BD-12_rendering-platform-upper-design.md)、[BD-16](../../12_design/BD-16_build-ci-target-matrix-upper-design.md)、[BD-22](../../12_design/BD-22_support-lifecycle-upper-design.md)
- 適用対象DD/SD: [DD-22](../../13_detail_design/22-build-distribution.md)

本書は承認したレビュー記録である。推奨案と代替理由は提案時点の比較として保持する。現行の方式・正式判断IDは[DD-37: 最低版・target matrix・toolchain lock・保守](../../13_detail_design/37-build-support-profiles.md)を正本とする。採用済みでも本文に将来候補と記した範囲は初期必須へ昇格しない。

## 判断案

| 判断ID | 推奨案・具体的な選択 | 代替と選択理由 | 承認区分 |
| --- | --- | --- | --- |
| REM-BUILD-01 | 初期正式profile候補はWindows11 x64、Debian12 glibc x64。ARMv7/AArch64は両方cross-build、代表実機はRaspberry Pi4 ModelB 4GBのAArch64を選ぶ。 | 一実機の合格をARMv7や他boardへ拡大しない。製品対象ARMは維持し段階状態を公表する。 | RQ・REM-RQ-27 |
| REM-BUILD-02 | Linux nativeはx86_64-unknown-linux-gnu、AArch64はaarch64-unknown-linux-gnu、ARMv7はarmv7-unknown-linux-gnueabihf。ARMv7 CPU=cortex-a7/NEON/VFPv4 hard-floatをprofileへ固定。 | target tripleだけではCPU/FPU/libc互換条件が足りない。 | DD |
| REM-BUILD-03 | minimumはC++17/CMake3.28、Rust1.85 edition2024、C#14/net10.0、CPython3.13。release compiler/SDKはlockfileのexact値を使う。 | minimumとrelease compilerの最新値を混同しない。 | RQ・REM-RQ-21 |
| REM-BUILD-04 | release lockはtoolchain/config commit、compiler/linker/Rust/SDK、sysroot digest、CPU flags、libc、font/graphics依存、container image digestを必須にする。 | 未取得のartifact digestを資料中で作り上げず、採取commandと入力元を確定する。 | DD/運用 |
| REM-BUILD-05 | 再現用候補はRust1.90.0、GCC12系/Debian12 SDK、MSVC v143/Windows SDK10.0.26100系。patch版はimageから採取しlock生成で固定、floating tagでreleaseしない。 | 数値をlatestと主張せず再現profileの候補として審査する。 | DD・REM-RQ-27 |
| REM-BUILD-06 | 新stable初版はruntime1.0.0/ABI1、pre-releaseは1.0.0-rc.N、wrapper versionは独立だがminimum Runtime/featureをmanifestに持つ。 | Gitのrelease branch規約とruntime ABI SemVerを別に対応付ける。 | 運用/RQ・REM-RQ-28 |
| REM-BUILD-07 | majorごとのnative filenameを分け、同process内はRuntime作成後に別majorへの暗黙switchを禁止。別majorの併存はloader handleを分離する。 | globalな曖昧libfreemvvm名だけに依存しない。 | DD |
| REM-BUILD-08 | support registryはmajor×profileのbuild/runtime/hardware/maintenanceを別columnにし、検証SHAと日付から公開表を生成する。 | 手作業のSupported一語で証拠区分を消さない。 | 運用 |
| REM-BUILD-09 | EOLの公開artifact・metadata・告知は継続保存。test logはEOL+24か月、security監査logは2年を初期運用候補とする。担当はReleaseMaintainer/SecurityMaintainerの役割で指定する。 | 未指定の実在担当者を作らない。公開EOL参照と内部log保管を区別する。 | 運用・REM-RQ-29 |

<a id="rem-build-01"></a>

<a id="rem-build-02"></a>

<a id="rem-build-03"></a>

<a id="rem-build-04"></a>

<a id="rem-build-05"></a>

<a id="rem-build-06"></a>

<a id="rem-build-07"></a>

<a id="rem-build-08"></a>

<a id="rem-build-09"></a>

## 実現設計案

### Profile候補とgate

| profile ID案 | OS/architecture | 初期gate | 公開表示の条件 |
|---|---|---|---|
| win11-x64-abi1 | Windows11 x64、MSVC CRT | build/ABI/4wrapper/縦断/TSF/UIA | 当該OS image・SHAの実行成功後だけRuntime verified |
| linux-x64-glibc236 | Debian12 x64、glibc2.36 | build/ABI/4wrapper/WaylandまたはX11/IBus/AT-SPI | desktop dependencyを含むprofile単位で公表 |
| linux-armv7-a7-hf | Linux ARMv7 hard-float | cross-build/link/artifact inspection | 初期はBuild verified。実機未実施と明記 |
| rpi4-a64-drm | RPi4 4GB、AArch64、DRM/EGL/GLES、明示IME broker | hardware/描画/入力/IME/性能 | board image・kernel・Mesa・依存hashを固定した実測後 |

代表boardの選定は価格/購入の依頼ではなくtest profileの案。未保有・未接続の実機で合格を作らない。ARMv7の実機gateを後から追加してもAArch64結果を転記しない。

`profiles/<profile>.lock.json`のschemaは`profile_id, target_triple, host_image_digest, rustc_version, cc_version, linker_version, sdk_id, sysroot_sha256, cpu_flags, libc_abi, dependencies[{name,version,digest,license}], source_revision`。採取はrustc -Vv、compiler -v、SDK manifestとsysrootのfile inventory digestから行う。lock generatorは全fieldを埋めるまでrelease jobを起動しない。実デバイスimageやprivate SDKを取得しない今回の資料では、digest欄を有効なlockとして生成しない。

公式根拠: [Rust1.90.0](https://blog.rust-lang.org/2025/09/18/Rust-1.90.0/)、[.NET support](https://learn.microsoft.com/en-us/dotnet/core/releases-and-support)、[Debian12 release information](https://www.debian.org/releases/bookworm/index)、[Raspberry Pi4仕様](https://www.raspberrypi.com/products/raspberry-pi-4-model-b/)。これらは候補の存在・制約の根拠で、本ライブラリの実機support証拠ではない。

## 適用結果

全判断を[DD-37](../../13_detail_design/37-build-support-profiles.md)へ正式IDで適用した。関連要求の変更もRQ本文へ反映した。実装・試験実行は未完了。[全200件の適用表](../remaining-design-application-result.md)で旧新対応を確認できる。

## 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 本書の案作成 | 利用者 | 一括生成指示。方式採用・正本適用は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 現行の未設計箇所を実現案・代替・interfaceへ具体化 | Codex |

## Build baselineの補足

[CMake3.28 release notes](https://cmake.org/cmake/help/v3.28/release/3.28.html)をminimum候補の照合先とする。C#14はlibraryを構築するsource基準案で、consumerのcompilerを不要に拘束しないようpublic facadeにC#14専用の構文依存を持ち出さない。TFM/net10.0の利用条件とcompiler source基準は別fieldで管理する。

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.2 / 本書の全推奨案と補足の採用・正本適用 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.2 | 2026-10-17 | 本書の全推奨案と補足の採用・正本適用を正本へ適用し、旧判断との対応を保持 | Codex |
