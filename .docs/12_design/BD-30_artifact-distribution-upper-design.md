# MVVM UIライブラリ Native artifact配布経路に関する上位判断案

- 状態: BD-30承認済み（2026-09-27）
- 対応: 公式native artifact catalogとlanguage wrapper package managerの責務、取得時のversion固定とcache
- 根拠: BD-17～BD-20、BD-22、BD-28～BD-29、要求仕様第9・12節。DD-00は未承認

## 既存の承認済み契約

- BD-17～BD-20はPython wheel、C# NuGet、C++ CMake package、Rust wrapperと別配布native runtimeの境界を定める。Rustにはtarget指定のartifact import featureがある。
- BD-28によりstable native runtimeはMAJOR.MINOR.PATCHで識別し、MAJORはABI symbol majorに一致する。wrapper package versionは独立でき、ABI majorとminimum runtimeを宣言する。
- BD-29によりnative artifactはimmutable identity、署名付きmanifest、digestを持ち、取得・利用前にtarget/profile、ABI、feature、署名、digestを照合する。不一致・検証不能は拒否する。
- registry、fetch/cache方式、package managerごとのruntime接続、更新公開経路は未決である。

## BD-30 承認済み判断

1. **native runtime artifactのcanonical release catalogを一つ設ける。** 公式manifestとimmutable artifact identityの権威ある取得先はこのcatalogとする。各language wrapper registryで配る同一native binaryは同じmanifest identity/digestを参照し、独自に再ビルド・再ラベルしない。
2. **language wrapperは各言語の既定package managerで配布する。** Python wheel、NuGet、CMake package、Cargo crateはBD-17～BD-20の形を保つ。package managerが対象target向けnative assetを同梱・取得する場合も、BD-29の公式identity・manifest・digestと一致させる。
3. **通常のnative asset選択はbuild/runtime targetと明示されたprofileから決める。** host上のpackage installはhost targetを使えるが、cross-compilationや別環境importではtarget triple/profileを明示する。推測で別target artifactへfallbackしない。
4. **再現可能なbuildではruntime versionとdigestへ解決結果を固定する。** `latest`等の可変aliasをCI/正式release buildの入力として残さない。lock/manifest等に解決済みidentityを記録し、cacheはidentity＋digestをkeyにする。通常package installでの暗黙更新は行わず、wrapper dependency更新か明示的なruntime upgradeで更新する。
5. **artifact fetch/importは明示的な操作として扱う。** native artifactがローカルまたはcacheに無い場合、package managerによる依存取得、またはRust等のtarget import featureが明示した解決を行う。検証が済む前にbuild/link/loadへ渡さない。offline利用では既に検証済みのcache/artifact bundleを使えるようにする。

## 理由とトレードオフ

- canonical catalogは言語別registryに分散したruntime binaryの版・target・digestを照合する基準になる。
- 既定package managerを維持すると、各言語利用者の既存のdependency workflowを使いながら、同じnative runtime provenanceを参照できる。
- targetを明示することで、組み込みLinux向けcross-buildで開発host用artifactを誤って選ぶのを防ぎやすい。
- digestでlockするため、同一version labelの差し替えや可変aliasの変化をbuildへ持ち込まない。更新操作は明示的になる。
- offline環境での取得を継続できる一方、bundle準備時に必要なtarget/profileとruntime versionを指定する必要がある。

## この判断では決めないこと

- canonical catalogの具体サービス、URL、API、registry hosting、認証、proxy動作。
- lockfile/manifestの具体形式、resolver CLIのコマンドと各wrapper package managerとのplugin実装。
- wrapper packageにnative assetを同梱するtarget範囲、wheel/RID/tripleの完全対応表。
- install時にnetwork fetchするか、native runtime用packageを別途要求するかの言語別詳細。
- cacheの保存場所・容量制限・GC、offline bundle形式、署名鍵のprovisioning/rotation。
- SBOM、build attestation、release approval workflowの詳細。

## 承認記録

- 利用者承認（2026-09-27）。上記1～5を上位判断として採用する。
- native artifactのcanonical identityとmanifestは公式catalogをsource of truthとする。
- 各language wrapperは既定package managerで配布し、native binaryを含める場合もcanonical artifact digestを保つ。
- cross-target artifact選択は明示的target/profileを使い、自動fallbackしない。
- 再現可能なbuildではimmutable versionとdigestをlockし、通常installで意図しないruntime更新をしない。
- URL、具体package-manager取得方法、cache運用は後続設計で決める。

URL、各package-manager取得方法、lockfile/manifest形式、cache運用の実装詳細は後続設計で決める。DD-00は未承認のため、DD-01以降へ進まない。
