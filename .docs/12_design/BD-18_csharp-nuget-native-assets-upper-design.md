# MVVM UIライブラリ C# NuGet native asset

- 状態: BD-18承認済み（2026-09-27）
- 対応: C# P/Invoke wrapperとtarget別Rust C ABI native runtimeをNuGetで配布する初期方式
- 根拠: Accepted ADR-0001、要求仕様第9・12節、BD-16、Microsoft NuGet native file／RID資料。DD-00は未承認

## BD-18 承認された判断

**C#向けは managed wrapper assembly と target別 native runtime を一つのNuGet packageにまとめ、各RIDのRust C ABI libraryを `runtimes/{rid}/native/` に格納する。NuGet/.NET SDKのRID asset選択とpublish時のnative library配置を利用し、初期RIDは Windows x64 (`win-x64`) と Linux x64 (`linux-x64`) から始める。ARMv7/AArch64はBD-16のcross-compile対象を維持するが、Yocto board profileごとのlibc・外部依存の互換性が確認できるまでは一般RID packageへ含めず、検証済みprofile別の配布方法を後続判断とする。**

## 配布構成

| 項目 | 推奨案 | 後続へ送る詳細 |
|---|---|---|
| NuGet package | C# managed wrapperとRID別native assetを同じpackageに含める | package ID、repository、version convention |
| native asset | `runtimes/{rid}/native/`に同一targetのRust C ABI shared libraryを置く | OSごとのfilename、複数依存library、loader search behavior |
| 初期RID | `win-x64`、`linux-x64` | 最低OS/runtime versionと具体CI images |
| embedded Linux | ARM artifactをbuild/link verificationし、hardware profile確認後にRID package可否を判断 | `linux-arm`／`linux-arm64`適用範囲、Yocto package/archive、libc baseline |
| ABI共有 | C#用にCoreを再compileせず、同一target C ABI artifactをC++／Rust／Python向け配布にも利用 | release pipeline内のartifact identity/provenance |

## 理由と影響

- 要求仕様第9節はC#でRIDごとのnative assetをNuGet packageに含めるとしている。NuGetのnative asset選択はRID別directoryを使い、publish先に選択したassetを配置する仕組みがある（[Microsoft: Native files in .NET packages](https://learn.microsoft.com/en-us/nuget/create-packages/native-files-in-net-packages)、[Microsoft: .NET RID catalog](https://learn.microsoft.com/en-us/dotnet/core/rid-catalog)）。
- managed wrapperとnative libraryを一緒にinstallすることで、C#利用者がnative packageを別途探してversionを合わせる手順を減らせる。代わりにNuGet package内に複数target binaryが入るため、package sizeとRIDごとのbuild/release検証が増える。
- RIDはOS／architectureを表すが、組み込みLinuxごとのsysroot・libc・GPU/input依存の互換性全てを保証しない。ARM向けbinaryがbuildできたことと、そのNuGet RIDで動作保証できることを分ける。
- PackageReferenceによるpublishだけでなく、開発時のbuild/run、self-contained publish、single-file等でnative library loadingと配置が成立するかはC# wrapper詳細設計の縦断試験で確認する。

## 本判断で決めないこと

- NuGet.org/private feed、package ID、signing、release credential。
- 正式な.NET target frameworkと最低runtime version。
- Linux glibc/musl baseline、追加RID、ARM board profile、Windows ARM64。
- native libraryのロードAPI、明示的resolver方式、配布binaryの全platform filenameと依存library bundle。
- license、support期間、SBOM、security update policy。

## レビューで確認する点

1. managed wrapper assemblyとRID別native libraryを初期は一つのNuGet packageに含めること。
2. native libraryはRID別のNuGet native assetとして扱い、最初は`win-x64`と`linux-x64`から始めること。
3. ARM向けcross-compileを行っても、board profileとのruntime互換性を確認する前に一般RIDの動作保証へ含めないこと。
4. C ABI native artifactは他言語と同一targetで共有し、C#専用Coreを作らないこと。
5. package naming、NuGet feed、TFM、exact support matrixを本判断で固定しないこと。

## 参考資料

- [Microsoft: Including native libraries in .NET packages](https://learn.microsoft.com/en-us/nuget/create-packages/native-files-in-net-packages) — RID別native assetの配置とbuild/publish動作。
- [Microsoft: .NET Runtime Identifier (RID) catalog](https://learn.microsoft.com/en-us/dotnet/core/rid-catalog) — NuGet packageのplatform-specific assetを選択するRID。

## 承認記録

2026-09-27に利用者承認。C# managed wrapper assemblyとRID別Rust C ABI native runtimeを同一NuGet packageに含め、`runtimes/{rid}/native/`を使う。初期RIDは`win-x64`と`linux-x64`。組み込みARM向けはcross-compileを続けるが、board profileごとのlibc・外部依存を確認してから一般RID packageへの収録可否を決める。NuGet feed、package ID、TFM、exact support matrix、library loading API、license等は本判断で確定しない。DD-00の承認やDD-01以降へ進む判断ではない。
