# MVVM UIライブラリ Python package/native distribution

- 状態: BD-17承認済み（2026-09-27）
- 対応: BD-14のctypes wrapperとBD-16のtarget別native libraryをPython利用者へ配布する初期方式
- 根拠: Accepted ADR-0001、要求仕様第9節、BD-12、BD-14、BD-16、PyPA wheel/platform tag仕様。DD-00は未承認

## BD-17 承認された判断

**Python向けはPython wrapper distributionとRust C ABI native runtime distributionを別packageにする。WrapperはPython標準ライブラリ`ctypes`だけで動くpure-Python wheelとし、native runtimeはOS／architecture別binary wheelまたはtarget向けnative artifactとして配布する。最初はWindows x64とLinux x64のnative wheelsを用意する。組み込みLinux ARMv7/AArch64はBD-16どおりnative artifactをcross-compileするが、対象boardのlibc・GPU・input依存が標準Linux wheelの互換条件を満たすと確認できるまでmanylinux/musllinux wheelとして表示しない。条件を満たさない場合は同じnative artifactをYocto／system packageやtarget-specific archiveから導入する。**

概念上のdistribution分割は次のとおりとする。実際のpackage名は決めない。

| Distribution | 内容 | wheel/platform tagの扱い |
|---|---|---|
| Python wrapper | Pythonic API、ctypes宣言、`.pyi`等の型情報。C ABIのnative binaryは含めない。 | Pure Python wheel。Python version supportを定めた後に`py3-none-any`等のtagを選ぶ。 |
| Native runtime | target別Rust Core、Renderer/backend、versioned C ABI shared library。 | Binary wheelを作る場合、Python tagだけでなく実際のOS/architecture/libc互換性に合うplatform tagを付ける。Python extensionではないため、CPython C ABIを不要に依存させない。 |

Wrapperは互換native runtime major versionを依存条件として宣言する。Installerで適合native wheelを取得できないtargetでは、wrapper install自体を誤成功させず、native artifactを別経路で配置・検出する手順を後続詳細設計で用意する。C++／C#／Rust wrapper用にもビルドした同一target C ABI artifactを再利用できるようにし、Python専用にRust Coreを再compileしない。

## 初期の配布順

1. **Windows x64:** pure-Python wrapper wheelとWindows x64 native runtime wheelを分ける。
2. **Linux x64:** wrapper wheelは共通で再利用し、Linux x64 native runtimeを別のplatform wheelとして出す。manylinux tagは依存libraryが要件を満たす時だけ使う。
3. **組み込みLinux ARMv7/AArch64:** cross-compiled native artifactをboard profileのsysroot／libcに合わせて用意する。標準wheel互換性を確認できたprofileだけwheel化し、他はYocto/system packageまたはtarget-specific archiveで配布する。
4. **その他:** BD-12／BD-16の順序に沿って、Windows ARM64やmacOS等を追加する。各targetにnative runtime artifactが必要。

## 理由とトレードオフ

- ADR-0001の「native libraryとlanguage wrapperを分ける」方針をPython packageでも保ち、同じtargetのC ABI libraryを他言語wrapperと共有できる。
- Wrapperがpure PythonならctypesのためだけのPython ABI別extension wheelを作らずに済む。一方でnative runtimeはOS／architectureごとの配布物が必要であり、wheel platform tagでinstallerに適合可否を伝える必要がある。
- Linuxの組み込みtargetはvendor sysroot、libc、graphics/input libraryが異なり得る。desktop/server向けmanylinux/musllinux tagを付けるだけで互換とみなさず、各binary dependencyを検査してから該当tagを使う。
- Python wrapperのversionが適合しないnative runtimeをロードしないよう、ABI majorとpackage dependencyを対応させる。minor互換範囲やpatch更新のversion constraintはA3とpackage toolの契約に合わせて詳細設計する。

## この判断では決めないこと

- PyPI公開、private index、社内artifact registryのどれを使うか。package名、owner、release credentials。
- Windows/Linux wheelの最低Python version、CPython/PyPy対応、exact wheel tag。
- manylinux/musllinux baseline、glibc version、bundled shared libraries、auditwheel相当の検査手順。
- 組み込みLinuxごとのYocto recipe、RPM/DEB/ipk package、installer script、board-specific archive構成。
- 各language packageの形式、license、support期間、正式release matrix。
- native library discovery path、ABI mismatch時のPython exception/detail text、security/SBOM signing。

## レビューで確認する点

1. pure-Python ctypes wrapperとtarget native runtimeを別distributionとして扱うこと。
2. 初期native wheelsをWindows x64とLinux x64から開始し、BD-16の対応順を保つこと。
3. 組み込みLinux向けnative artifactはcross-compileしても、標準wheel tagの互換性を確認するまではcustom target packageとして扱うこと。
4. Pythonと他言語で同じtarget C ABI artifactを共有し、Python用native Coreを重複buildしないこと。
5. package index、names、wheel tag、license、release policyを本判断で確定しないこと。

## 参考資料

- [PyPA: Platform compatibility tags](https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/) — wheelのPython／ABI／platform tagと互換性選択。
- [PyPA: Binary distribution format](https://packaging.python.org/en/latest/specifications/binary-distribution-format/) — wheelの配布形式とplatform tagを含むfilename。
- [PyPA: manylinux](https://github.com/pypa/manylinux) — Linux向けportable wheelのtag、architecture対応、外部依存の制約。

## 承認記録

2026-09-27に利用者承認。Python wrapperとRust C ABI native runtimeを別distributionとし、wrapperはpure-Python wheel、native runtimeはtarget別binary wheelまたはnative artifactとして配布する。初期native wheel対象はWindows x64とLinux x64。組み込みLinux ARMv7/AArch64はcross-compileするが、対象profileの互換性が確認できるまでは標準manylinux/musllinux tagを付けない。package名、index、exact wheel tag、Python対応範囲、license、support期間は本判断では確定しない。DD-00の承認やDD-01以降へ進む判断ではない。
