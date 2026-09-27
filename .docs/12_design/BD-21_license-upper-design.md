# MVVM UIライブラリ first-party licenseに関する上位判断案

- 状態: BD-21承認済み（2026-09-26）
- 対応: Rust Core、C ABI、language wrapper、source packageとbinary SDK distributionのfirst-party license
- 根拠: 要求仕様第9・12節、ADR-0001、BD-17～BD-20、Open Source Definition、Apache License 2.0公式条文。DD-00は未承認

## ユーザーが示した優先順位

当初はcompiled artifactのみの再配布を許可し、改変sourceの再公開を禁止する意向だった。その後、利用者はこの制限よりも「オープンソースであること」を優先する意向を示し、Apache-2.0を採用するBD-21を承認した。したがって、本判断は改変sourceおよび改変binaryの再配布を許可する。

## BD-21 承認された判断

**first-party library source、C ABI/header、C++／C#／Rust／Python wrapper、source package、target-specific binary SDKにApache License 2.0（SPDX `Apache-2.0`）を適用し、sourceを公開可能な形で提供する。利用者は本ライブラリのsourceを改変し、改変sourceおよび改変binaryを再配布できる。アプリ実行ファイルと、compiled `.dll`等・public headersを含むbinary SDKも再配布できる。改変物にはApache-2.0のlicense/notice条件を適用し、third-party dependencyは各上流licenseのまま取り扱う。**

OSI Open Source Definitionはsource配布とmodified/derived worksの再配布を要求する。Apache-2.0は改変物をsource formおよびobject formで配布可能にし、再配布時のlicense、変更通知、attribution/NOTICE等の条件を定める（[OSI Open Source Definition](https://opensource.org/osd)、[Apache License 2.0, sections 2–4](https://www.apache.org/licenses/LICENSE-2.0)）。

## 理由とトレードオフ

- Apache-2.0はpermissiveなOSI-approved licenseで、商用利用・binary再配布を含む用途を許可しつつ、Contributorの一定範囲の特許請求に対する明示的なlicense grantを含む。
- section 3には、Workに対する特許侵害を主張する訴訟を提起した場合に当該Workのpatent licenseが終了する条件がある。採用時にはLICENSE本文で利用者に明示する。
- 利用者の以前の案にあった「改変sourceを再配布できない」という制限は、オープンソース優先の判断により撤回する。公式版と第三者forkを区別するには、licenseによるfork禁止ではなく、商標・公式名称の方針、署名済み公式release、checksumとbuild provenanceを使う。
- ライセンスだけでは、第三者が脆弱性を含む改変binaryやappを作成・再配布することを防げない。artifactの真正性と公式sourceの識別には、別途release/security controlsが必要になる。
- 単一のApache-2.0は4言語のsourceと複数distributionでlicense metadataを揃えやすい。`MIT OR Apache-2.0`のdual optionよりlicense file、NOTICE、patent termsを一つに統一することを推奨する。

## Distribution時の最低方針

1. 公開source repository/source archiveにApache-2.0 LICENSEを含め、改変sourceの再配布を許可する。
2. NuGet、wheel、CMake SDK、Cargo crateおよびbinary SDKにはlicense textと必要な上流noticeを含める。
3. Apache `NOTICE` fileを配布する場合はApache-2.0 section 4に沿って派生distributionへ引き継ぐ。改変source filesには変更した旨を記載する。
4. Third-party source/binary dependencyをApache-2.0へ再標識せず、license inventoryとnoticeをreleaseごとに検証する。
5. 公式binary artifactは署名・checksum・provenanceで識別し、third-party forksが公式releaseと混同されないよう商標・release naming policyを別途設計する。

## 本判断で決めないこと

- 公開repository hosting、registry公開順、release owner/credential。
- Contributor License Agreement (CLA)、Developer Certificate of Origin (DCO)、copyright ownershipの収集方式。
- 商標使用、fork naming、公式supportやsecurity responseの条件。
- Documentation、fonts、icons、sample assetsの独立license。
- Exact third-party license allowlist、SBOM format/tool、notice aggregation pipeline。
- version support period、security response window、ABI major EOL policy。

## 承認記録

利用者は2026-09-26にBD-21を承認した。上記のApache-2.0採用、sourceと改変物の再配布許可、配布notice方針を承認済み上位判断として扱う。実際のLICENSE／NOTICE本文作成、第三者dependency inventory、署名・provenanceの詳細、repository/registry公開は別作業であり、この判断だけでは実施・確定しない。

## 参考資料

- [Open Source Definition](https://opensource.org/osd) — source code、modified/derived works、patch-only restrictionのcriteria。
- [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0) — copyright/patent grantとsource/object redistribution条件。
- [Apache Licensing and Distribution FAQ](https://www.apache.org/foundation/license-faq.html) — license noticeの適用方法。
- [OSI Approved Licenses](https://opensource.org/licenses) — OSI-approved license一覧。

BD-21は上位判断として承認済み。DD-00は未承認のままであり、DD-01以降へ進まない。
