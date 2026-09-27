# MVVM UIライブラリ C++ CMake package

- 状態: BD-19承認済み（2026-09-27）
- 対応: C++ wrapperとtarget別Rust C ABI native runtimeをC++ projectへ導入する初期方式
- 根拠: Accepted ADR-0001、要求仕様第4・9・12節、BD-16～BD-18、CMake package export仕様。DD-00は未承認

## BD-19 承認された判断

**C++ wrapperはRust/C++ ABIを跨がずversioned C ABIを呼ぶ薄いC++ source wrapperとして配布し、CMake install/exportによるrelocatable packageを標準の利用境界にする。packageはpublic C++ headersとwrapper sources、対象targetのC ABI native library、`<Package>Config.cmake`とexported targetsを含め、利用側は`find_package(... CONFIG)`で接続する。初期targetはBD-16の順にWindows x64とLinux x64から始める。ARM Linux artifactは引き続きcross-compileするが、sysroot／libc profileと実機gateを確認したtargetごとにCMake install treeを生成する。vcpkg／Conan registryなどの特定package managerは本判断で必須にせず、後続でCMake packageをそれらへ登録する選択肢を残す。**

## 配布境界

| 構成要素 | 推奨案 | 詳細設計へ残す事項 |
|---|---|---|
| C++ wrapper | C ABIをRAII等のC++ APIで包むheader/source形式。C++ ABIをRust Coreへ露出させない | 最低C++ language standard、namespace、template／exception policy |
| Native runtime | BD-17／BD-18と同じtarget C ABI artifactを含める | platform別library filenameとtransitive native dependencies |
| CMake integration | install/export targetとpackage configを用意し、downstream projectは`find_package(... CONFIG)`で利用 | minimum CMake version、target名、config version policy |
| 初期target | Windows x64、Linux x64。後続でprofile検証済みARM Linux | compiler/runtime variants、Windows ARM64、macOS |
| Package manager | CMake install treeを基準とし、vcpkg／Conan固有port・recipeは別判断 | 公開registry、manifest、binary cache、credential |

## 理由と影響

- CMakeの`install(TARGETS)`と`install(EXPORT)`は、install済みtargetをdownstream projectへimportできるpackage構成を作れ、package configを通じた再配置可能な配布を支援する（[CMake: Importing and Exporting Guide](https://cmake.org/cmake/help/latest/guide/importing-exporting/index.html)）。
- `find_package`のconfig modeはpackage自身が提供するconfig fileを使ってtargetを公開できるため、CMake利用者向けにbuild/link usage requirementsを渡せる（[CMake: find_package](https://cmake.org/cmake/help/latest/command/find_package.html)）。
- 最初にregistry一つへ依存せず、Windows、Linux、Yocto cross-buildで共通するinstall treeを基準にすれば、CMake利用者に直接渡し、後からvcpkg／Conan等のdistributionへ接続できる。vcpkgにはCMake integrationとtriplet modelがあり、ConanはCMake toolchainを介したcross-buildを支援する（[vcpkg: CMake integration](https://learn.microsoft.com/en-us/vcpkg/users/buildsystems/cmake-integration)、[vcpkg: triplets](https://learn.microsoft.com/en-us/vcpkg/concepts/triplets)、[Conan: CMakeToolchain](https://docs.conan.io/2/reference/tools/cmake/cmaketoolchain.html)）。
- C++ wrapperをC++ ABI固有のbinary SDKにせずsource wrapperにすることで、MSVC／GCC／Clang間でC++ binary ABIを共有する必要を避け、安定境界をC ABIへ保てる。一方、wrapper sourceは利用者側のtoolchainでcompileする。
- Native C ABI shared libraryはtargetごとにbuild/linkする。packageに含まれるtarget artifactと利用側compiler／linker／libcが一致しなければ動かないため、別target binaryを一つのgeneric packageとして扱わない。

## 本判断で決めないこと

- vcpkg、Conan、CPM、OS package、private registry等の配布先や優先順位。
- C++ wrapperの最低standard、header-only化、exact API naming、namespace、exception contract。
- CMake minimum version、export target name、shared/static library option、Windows import libraryの命名。
- Linux glibc/musl baseline、Yocto distro、ARM board profile、compiler toolchain matrix。
- package archive extension、versioning、signing、license、support period。

## レビューで確認する点

1. C++利用者向け標準integrationをrelocatable CMake install/export packageとすること。
2. C++ wrapperをC ABIの上に置き、Rust/C++ ABIを言語間境界にしないこと。
3. 初期targetをWindows x64／Linux x64とし、ARM Linuxはprofile単位で検証してから利用保証へ加えること。
4. C++ packageのnative runtimeを他言語packageと同一target C ABI artifactから供給すること。
5. vcpkg／Conan等のregistryやexact compiler, CMake, C++ standard matrixを本判断で確定しないこと。

## 参考資料

- [CMake: Importing and Exporting Guide](https://cmake.org/cmake/help/latest/guide/importing-exporting/index.html) — install/export target、package config、relocatable package。
- [CMake: find_package](https://cmake.org/cmake/help/latest/command/find_package.html) — downstream projectからのpackage config探索。
- [Microsoft: vcpkg in CMake projects](https://learn.microsoft.com/en-us/vcpkg/users/buildsystems/cmake-integration) — CMake toolchain integration。
- [Microsoft: vcpkg triplets](https://learn.microsoft.com/en-us/vcpkg/concepts/triplets) — target environmentを表すtriplet。
- [Conan: CMakeToolchain](https://docs.conan.io/2/reference/tools/cmake/cmaketoolchain.html) — CMake cross-build toolchain integration。

## 承認記録

2026-09-27に利用者承認。C++ wrapperはC ABIを包む薄いC++ source wrapperとし、public headers／wrapper sources／target別C ABI native runtime／CMake package configとexported targetsを含むrelocatable CMake install/export packageを標準integrationとする。downstream projectは`find_package(... CONFIG)`で利用する。初期targetはWindows x64／Linux x64、ARM Linuxはprofile検証後に追加する。vcpkg／Conan registry、C++ standard、CMake minimum version、exact compiler matrix、package archive、license／supportは本判断で決めない。DD-00の承認やDD-01以降へ進む判断ではない。
