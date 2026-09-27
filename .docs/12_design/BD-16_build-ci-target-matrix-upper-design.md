# MVVM UIライブラリ 初期Build／CI target matrixに関する上位判断案

- 状態: BD-16承認済み（2026-09-27）
- 対応: BD-12のWindows → Linux x64 → 組み込みLinux → その他の順を、初期build・test gateへ落とす
- 根拠: Accepted ADR-0001、要求仕様第8～10節、承認済みBD-12、A1～A32、L1～L4。DD-00は未承認

## BD-16 承認された判断

**初期CIはWindows x64、Linux x64、組み込みLinux ARMv7／AArch64を段階的に加える。Windows x64で最初のnative buildと縦断smoke testを行い、次にLinux x64のnative build/testを加える。Linux x64 hostからARMv7とAArch64のtarget artifactをcross-compileし、両targetのbuild/link検査をCI gateにする。組み込みLinuxの実機試験は代表boardで別に実施し、cross-compile成功だけでARM runtime support済みとは判定しない。Windows ARM64とmacOS等はこの初期matrixの後に加える。**

| 段階 | Build target / host | CIで行うこと | 扱い |
|---|---|---|---|
| 1 | Windows x64 host → Windows x64 native target | Rust Core、C ABI、必要なplatform integrationをbuildし、Core/ABIのunit・integration・4言語smoke testを実行する。最初の描画・Accessibility検証もこの環境から始める。 | 最初の開発・CI target |
| 2 | Linux x64 host → Linux x64 native target | native build、Core/ABI test、Renderer/backend smoke、wrapper parityを追加する。Linux x64をARM cross-buildのhostとしても使う。 | 2番目のCI target |
| 3a | Linux x64 host → embedded Linux ARMv7 target | 対象board profileのcompiler/linker/sysrootでCore、C ABI、依存backendをcross-compileし、link/import・artifact inspectionを行う。 | build verified。実機動作確認まではruntime supportを宣言しない |
| 3b | Linux x64 host → embedded Linux AArch64 target | 同様にAArch64向けartifactをcross-compileし、target-specific依存・ABIを検査する。 | build verified。実機動作確認まではruntime supportを宣言しない |
| 3c | Linux ARM representative board | 最初に選定した実機でwindow/surface、描画、input、IME、Accessibilityの基本経路と性能指標を検証する。GPU等の変更を含むCI gateの頻度は後続詳細設計で定める。 | hardware verifiedは当該board/profileの範囲に限定 |
| 4 | Windows ARM64、macOS x64/ARM64等 | 先行targetのbuild/testが安定した後、別段階で追加する。 | 初期matrix対象外。対応順はBD-12を維持 |

## 推奨するBuild／Runtime判定の区別

- `build verified`: target向けcompile/linkが成功し、ABI symbol/versionとnative dependencyの検査を通過した状態。
- `runtime verified`: 実target上でUI起動・描画・入力等の定めたsmoke testを実行した状態。
- `hardware profile verified`: 特定board、OS image、sysroot、GPU/input driverの組み合わせで実機検査を通過した状態。
- ARMv7とAArch64の両方はCIでcross-compileするが、最初の実機boardは一つを選んで開始する。未検証architecture／boardへ実機保証を拡張する場合は別hardware gateを追加する。
- Linux x64から共有するのはRust source、C ABI、共通Renderer、build/test scripts等であり、x64 artifact自体ではない。ARM artifactはtarget compiler、ABI設定、sysroot、native libraryを使って個別に作る。

## 選択理由と制約

- Windows x64を最初にすれば、開発環境で早くbuild・UI smokeを回せる。次にLinux x64を加えることで、embedded cross-buildより先にLinux host上でplatform backendとnative dependencyを確認できる。
- Linux x64 hostはLinux ARM targetのtoolchain／sysrootを呼び出すbuild hostである。Rust target triple、linker、C library ABI、graphics/input依存はARM用に一致させる必要がある。
- ARM向けcompile testはtarget boardのGPU driver、display mode、IME、input device、電源管理を実行しない。実機検証を別gateとして残す。
- ARMv7のCPU/FPU/float ABI、Linux libc、GPU/backendの具体profileとboardはADR-0001／要求仕様に沿って別途選定する。

## この判断では決めないこと

- Windows/Linuxの最低OS version、Linux distribution/glibc baseline、compiler version、exact Rust target triple。
- ARMv7 CPU feature/FPU/float ABI、Yocto release、sysroot package lock、C/C++ toolchain version。
- Windows x64／Linux x64／ARMのCI provider、runner image、job graph、cache key、test parallelism。
- ARM board model、OS image、GPU driver、input device、IME framework、performance threshold。
- NuGet/PyPI/Cargo/vcpkg等のpackage formats、native library bundling、license。
- Windows ARM64/macOS等の追加時期・正式release support matrix。

## 承認記録

2026-09-27に利用者承認。初期CI targetをWindows x64、Linux x64、組み込みLinux ARMv7/AArch64の順に追加する。Linux x64 hostからARM target artifactをcross-compileし、両アーキテクチャのbuild/link検査を行う。cross-compile成功を実機Runtime supportと同一視せず、実機hardware profile検証を別gateとする。最初の実機boardは一つを選んで開始し、未検証architecture／boardへ検証結果を広げない。Windows ARM64、macOS等は後続段階に置く。exact compiler、sysroot、board、package、正式release matrixは後続判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
