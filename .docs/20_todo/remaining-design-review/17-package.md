# 未設計レビュー / package名・loader・artifact取得・catalog

- 文書版: 0.2
- 状態: 全判断承認済み・正本適用済み（2026-10-17）
- 判断ID: REM-PACKAGE-01～08
- 入口: [全件索引](index.md)
- 根拠RQ: [RQ-12-004](../../11_requirements/RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](../../11_requirements/RQ-12_build-distribution.md#rq-12-005)、[RQ-12-006](../../11_requirements/RQ-12_build-distribution.md#rq-12-006)、[RQ-12-007](../../11_requirements/RQ-12_build-distribution.md#rq-12-007)、[RQ-12-008](../../11_requirements/RQ-12_build-distribution.md#rq-12-008)、[RQ-12-011](../../11_requirements/RQ-12_build-distribution.md#rq-12-011)、[RQ-12-012](../../11_requirements/RQ-12_build-distribution.md#rq-12-012)、[RQ-12-013](../../11_requirements/RQ-12_build-distribution.md#rq-12-013)
- 具体化するBD: [BD-17](../../12_design/BD-17_python-package-distribution-upper-design.md)、[BD-18](../../12_design/BD-18_csharp-nuget-native-assets-upper-design.md)、[BD-19](../../12_design/BD-19_cpp-cmake-package-upper-design.md)、[BD-20](../../12_design/BD-20_rust-cargo-package-upper-design.md)、[BD-29](../../12_design/BD-29_artifact-provenance-upper-design.md)、[BD-30](../../12_design/BD-30_artifact-distribution-upper-design.md)
- 適用対象DD/SD: [DD-18](../../13_detail_design/18-wrapper-cpp.md)、[DD-19](../../13_detail_design/19-wrapper-csharp.md)、[DD-20](../../13_detail_design/20-wrapper-rust.md)、[DD-21](../../13_detail_design/21-wrapper-python.md)、[DD-22](../../13_detail_design/22-build-distribution.md)

本書は承認したレビュー記録である。推奨案と代替理由は提案時点の比較として保持する。現行の方式・正式判断IDは[DD-38: package名・loader・artifact取得・catalog](../../13_detail_design/38-package-loader-tool.md)を正本とする。採用済みでも本文に将来候補と記した範囲は初期必須へ昇格しない。

## 判断案

| 判断ID | 推奨案・具体的な選択 | 代替と選択理由 | 承認区分 |
| --- | --- | --- | --- |
| REM-PACKAGE-01 | native filenameはfreemvvm_abi1.dll / libfreemvvm_abi1.so。package ID案はPython freemvvm、native freemvvm-runtime-<profile>、NuGet FreeMvvm、CMake FreeMvvm、crate freemvvm/freemvvm-sys。 | 名称のregistry利用可否は公開時確認。未所有namespaceを確保済みとしない。 | DD/運用 |
| REM-PACKAGE-02 | CMake exportはFreeMvvm::cppとFreeMvvm::runtime、relocatable Config package。静的Core同梱variantは初期非提供。 | static/dynamic混在で4言語が別Coreを利用する構成を避ける。 | DD・REM-RQ-30 |
| REM-PACKAGE-03 | Python pure wheelはpy3-none-any、native Windowsはpy3-none-win_amd64、Linux x64は検証済みmanylinux baseline候補。ARMはprofile archive/Yocto recipeを先に使う。 | 未検証ARMをmanylinux/musllinuxへ分類しない。 | DD |
| REM-PACKAGE-04 | NuGetはruntimes/win-x64/nativeとlinux-x64/nativeにnative+manifestを同梱。single-file publishでも抽出後bytesを再検証する。 | default probingだけで別profileの同名libraryを利用しない。 | DD |
| REM-PACKAGE-05 | Cargoはnative-prebuilt featureで明示importを許可し、build.rsはnetwork禁止、artifact.lockとprefixを検査してlink metadataだけを出す。 | featureを--targetの代替にしない。 | DD |
| REM-PACKAGE-06 | freemvvm-tool fetch/import/verify/lockの4commandを作り、fetchはversion/profile指定必須、upgradeは別の明示lock更新操作。 | 通常build/importでlatestを暗黙選択しない。 | DD |
| REM-PACKAGE-07 | canonical catalogは署名済み静的indexとcontent-addressed blobsのHTTPS/file backend。service URLは運用configに置き、consumerへの任意外部URLは初期無効。 | 実在しない公式domainを設計で取得済みとしない。 | DD/運用 |
| REM-PACKAGE-08 | cacheはOS user cache配下のfreemvvm/v1、credentialはOS storeまたはCI secret参照。manifest/lock/logへsecret値を保存しない。 | cacheの有無をtrust rootにしない。 | SD/運用 |

<a id="rem-package-01"></a>

<a id="rem-package-02"></a>

<a id="rem-package-03"></a>

<a id="rem-package-04"></a>

<a id="rem-package-05"></a>

<a id="rem-package-06"></a>

<a id="rem-package-07"></a>

<a id="rem-package-08"></a>

## 実現設計案

### Artifact tool protocol

```text
freemvvm-tool fetch --catalog <configured-origin> --version 1.0.0 --profile <id>
freemvvm-tool import --manifest <file> --artifact <file> --trust-bundle <file>
freemvvm-tool verify --lock artifact.lock.json
freemvvm-tool lock --manifest <verified-manifest> --output artifact.lock.json
```

すべて案のcommandで実在toolではない。fetchはmetadata→trust→bytes→digest→private staging→atomic publish→lock候補の順。importは同じ検証器へlocal filesを渡す。lock更新は利用者が差分を確認しcommitする操作へ分ける。

| 配布 | wrapperとnativeの接続 | 誤適合時 |
|---|---|---|
| Python | import wrapper時にprototype登録、Runtime.createでselected artifactを検証/load | native未取得はMissingRuntimeError。ネット取得しない |
| C# | 専用interop assembly resolverへverified absolute pathを登録 | profile/RID mismatchを明示してloadしない |
| C++ | find_packageのimported targetがnative prefixを参照 | host pathをtarget libraryとしてlinkしない |
| Rust | build.rsがtarget/profile/ABI/features/lockを検証 | lock不足/不適合はcompile時error、fetchしない |

Linux wheelのbaselineはglibc2.36の候補profileと、auditwheel相当のdependency検査結果からtagを決定する。tagの文法は[PyPA公式仕様](https://packaging.python.org/en/latest/specifications/platform-compatibility-tags/)に照合し、単にdistribution OS名からmanylinux互換と判断しない。publisherはPython/C#/C++/Rust packageへ同じnative digestを収録し、圧縮やlayout変更以外の再compileを行わない。

Release assemblyはartifact manifest、license/notice inventory、wrapper minimum constraints、target profile、provenanceをbundleへ入れる。CLA/DCO・商標・docs/assets別license・public registryへのpublishは今回の採用対象に含めず、必要なら要求変更案により扱う。

## 適用結果

全判断を[DD-38](../../13_detail_design/38-package-loader-tool.md)へ正式IDで適用した。関連要求の変更もRQ本文へ反映した。実装・試験実行は未完了。[全200件の適用表](../remaining-design-application-result.md)で旧新対応を確認できる。

## 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 本書の案作成 | 利用者 | 一括生成指示。方式採用・正本適用は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 現行の未設計箇所を実現案・代替・interfaceへ具体化 | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.2 / 本書の全推奨案と補足の採用・正本適用 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.2 | 2026-10-17 | 本書の全推奨案と補足の採用・正本適用を正本へ適用し、旧判断との対応を保持 | Codex |
