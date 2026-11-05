# Linux Platform backend Docker仮検証計画

- 計画版: 0.1
- 状態: 実装済み・Debian 12 Docker仮pass（2026-10-29）。実機profile確認・正式release passは未達
- 作成日: 2026-10-28
- 対象version: `0.0.1`
- 対象branch案: `impl-rust/platform-adapter-first-backend`
- 前段: [Platform実機smoke計画](linux-platform-hardware-smoke.md)は、実機準備目処待ちで保留

## 目的とpassの意味

Debian 12 x64相当のcontainer内で、Linux Platform crateのnative dependency/build/test、および可能な範囲のGLES rendererのheadless software-renderingを確認する。成功時の状態は**container内仮pass**とし、Debian 12物理機またはRPi 4 AArch64でのDRM/KMS scanout、hardware GLES、display mode復元、正式support/release合格を示さない。

Windows runner上で既存CIと重複するだけにならないよう、Linux crateのDebian 12依存整合と、GLES実描画をsoftware Mesaでpixel-readbackする試験を中心とする。既存GitHub Actions workflowは変更しない。

## 根拠と対応範囲

- 要求: [要求仕様索引](../11_requirements/reqirements-specification.md)、[RQ-11 Platform](../11_requirements/RQ-11_platform.md) RQ-11-019/021/024。
- 設計: [Platform internal layers](../13_detail_design/platform/internal-layers.md) v1.4。Core `DeviceScale`、Core指定背景RGB、Clip、FillRect。
- 実装: `crates/platform-linux` とUNIT-PLATFORM-002の既存pure render-plan/fake worker tests。
- physical DRM/KMS evidence: 別途保留中のhardware smoke計画。本Docker planと混合しない。

### UNIT-PLATFORM-CONTAINER-001（提案）

1. `debian:12-slim` x86_64をbaseとし、image digestをreportへ記録。`rust-toolchain.toml` pinと整合するRust toolchain、`build-essential`、`pkg-config`、`libdrm-dev`、`libgbm-dev`を含む専用Dockerfileを追加する。Mesa runtime/dev dependencyはheadless testに必要なものに限定する。
2. `tools/dev.py`に明示commandを追加してimage build/runを再現可能にし、`.local/verification/linux-platform-docker.json`へimage digest、source SHA、toolchain、commands/resultsを保存する。
3. container内でCore/Linux adapter unit tests、Linux native `cargo check`、Clippyを実行。必要に応じ最小Rust `1.85.0` checkも記録する。
4. **UT-PLATFORM-GL-HEADLESS-001（提案）**: software Mesaのheadless EGL pbuffer/surfaceless contextで実GLES3 shaderを動かし、既知SceneSnapshotのscale、background clear、Clip、FillRectの代表pixelをin-memory readbackでassertする。動画・RawMovieは生成しない。必要package/driverがDocker headless modeで成立しない場合は、失敗を隠さず`blocked`記録とし、existing pure plan testsだけでGL drawing passを名乗らない。
5. Dockerでは`--privileged`や`/dev/dri` device passthroughを使わない。Cargo依存取得に通常のnetwork accessを使う。よってDRM node選択、GBM surface、KMS modeset/page-flip、real GPU/monitorは本試験のoracleに含めない。

## 仮pass受入条件

- containerがDebian 12 userlandで指定SHAをcheckoutし、Rust toolchainとLinux native dependenciesを記録する。
- Core/Linux focused tests、native Linux compile/Clippyがすべてpass。
- headless GLES testが成立した場合、software renderer名を記録し、背景色・Clip・FillRectの指定pixel assertionsがpass。成立しない場合はcontainer build/testの部分結果のみ記録し、GL実描画部分を`blocked`とする。
- reportのoverall stateは`provisional_pass`。Docker image resultを`hardware_pass`、物理Debian profile pass、RPi 4 pass、またはrelease passに昇格させない。
- 物理実機確認は別計画の保留状態を維持し、hardware preparationの目処が立った時に再開する。

## 現環境での実行可否

2026-10-28に読み取り確認したlocal Windows hostではDocker CLI 29.2.1が見えるが、Docker Linux engine named pipeがなく、`com.docker.service`はStoppedで、現行権限からのservice startは拒否された。WSL distroは`docker-desktop`だけで、通常のDebian/Ubuntu distroは見つからず、`/dev/dri`もない。このため現時点でcontainer build/runやhardware accessは実行していない。Docker Desktop/serviceが利用可能な環境またはLinux Docker hostでの実行が必要。

## 除外

- CI workflow改訂、自動release gate、formal product support判定。
- DRM/KMS/GBM physical device、monitor scanout、page-flip timing、display restoration、input/IME/Accessibility。
- RPi 4のAArch64 hardware run、hardware renderer/driver performance。
- RawMovie/video capture。GPU readbackはtest assertion用の少数pixelだけをmemory内で検証し、ファイルとして連続frame保存しない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-28 | 計画0.1 | 提案 | 利用者のDocker仮pass要望をDebian 12 container build/testとheadless GLES pixel testに具体化 |
| 2026-10-29 | 計画0.1全文 | 承認済み | 利用者の「ok」により計画を承認、実装着手。実行結果・仮passの成立とは別 |

## 実装・実行状況（2026-10-29）

- `docker/linux-platform/Dockerfile`、`tools/linux_platform_docker.sh`、`tools/dev.py linux-platform-docker`を追加。成功時だけreportを`provisional_pass`にする。
- ignored headless GLES pbuffer testを追加。8x8 pixel readbackでCore scale・背景clear・Clip・FillRectを検証し、Mesa software renderer名も確認する。動画・RawMovieは出力しない。
- Windows host上のCore 160 tests / Linux adapter 10 testsはpass。Linux targetのadapter test compileとClippyもpass。これらはDebian containerまたはheadless GLES実行の証跡ではない。
- 初回はDocker engine context file使用中でbuildが開始できなかったが、engine接続確認後に再実行し完了。
- Debian 12.12 / image `sha256:d678325a64e51872a26f1743f1c4ee07c588dad7a14e5ad816a4dea1074b5aa8`で、Core 160件、Linux adapter 14件、headless GLES pixel test 1件（llvmpipe / LLVM 15.0.6）、native check、Clippy、Rust 1.85 minimum checkがpass。reportは`provisional_pass`。
- この結果はDebian 12 x64 containerのsoftware Mesa描画・Rust/native dependency検証に限定する。物理DRM/KMS、GBM実surface、page flip、RPi 4/ARM実機、実GPU/displayを検証しておらず、正式support/release passへ昇格しない。
