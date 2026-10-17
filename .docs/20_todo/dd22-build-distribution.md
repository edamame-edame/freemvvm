# DD-22 Buildと配布 作業計画

- 状態: 完了（DD-BUILD-01～11利用者承認済み、2026-10-12）
- 着手日: 2026-10-12
- 対象: target別native artifactのbuild/profile、4言語package接続、artifact identity/catalog、manifest・署名・trustの詳細境界
- 上位出典: Accepted ADR-0001、Accepted要件、承認済みA1～A32、BD-16～22、BD-28～32
- 先行詳細設計: 承認済みDD-01～21
- 対応する詳細設計: `.docs/13_detail_design/22-build-distribution.md`

## 目的

Rust Coreとversioned C ABIをtargetごとに一度buildし、4言語wrapperの上位package managerへ接続しながら、native artifactのidentity・署名・manifest・信頼検証を一致させる。build verified、runtime verified、hardware profile verifiedを分ける。承認済みBD判断を具体化し、exact toolchain、package ID、registry運用、OS/board matrixなど上位が残した事項を暗黙に確定しない。

## 作業手順

1. ADR、要件、A1～A32、BD-16～22/28～32、DD-06/16/18～21、release/version test方針の状態と承認範囲を本文で照合する。
2. Windows x64、Linux x64、Linux hostからのARMv7/AArch64 cross-build、実機profile gate、後続targetのbuild profile区分とbuild/runtime/hardware verification stateを提案する。compiler/sysroot exact versionは未決境界を保つ。
3. Python wheel、NuGet RID native asset、CMake install/export package、Cargo `-sys`/safe wrapperとprebuilt-native経路の内容・artifact参照関係を表にする。4言語が同一target C ABI artifactを使い、独自再build/relabelしない制約を保つ。
4. canonical release catalog、immutable artifact identity、target/profile明示、version/digest pin、cache key、explicit fetch/import、検証前link/load禁止、offline artifact/trust bundleの関係を提案する。catalogの製品名・URL・transport・cache storageは決めない。
5. manifest必須情報とschema/runtime versionの独立性、署名対象、unknown field/majorのfail-closed境界をBD-29/32に沿って整理する。encoding、canonicalization、hash/signature algorithmは根拠ある候補を調査し、未承認選択肢としてレビューに出す。
6. trust rootの独立provision、known/revoked key検証、rotation/緊急失効、offline stale trust扱いをBD-31に沿って提案する。key custody、HSM、CI identity、release approval workflowは未決に分ける。
7. ABI runtime MAJOR.MINOR.PATCH、ABI symbol major、wrapper package version、manifest schema version、target profile/support/EOLの別個の識別・対応を示す。BD-21 license/notice境界もpackage成果物へ反映する。
8. build/release pipeline上のartifact生成・検査・署名・catalog登録・package assembly/export・consumer validationの順序と失敗時のfail-closed境界をレビュー案にする。release test/version testとの責務を混ぜない。
9. `DD-BUILD-*`判断ID付きレビュー案を `.docs/13_detail_design/22-build-distribution.md` に保存し、上位決定・未決・対象外・検証観点と相互参照を点検した。利用者確認前は承認済み仕様にしない。

## 対象外と停止条件

- BD-16～22/28～32、A3、ADR-0001の承認済み判断をDD単独で変更すること。
- exact compiler/linker/Rust toolchain/SDK/sysroot/Yocto release、board model、GPU/input stack、minimum OS/libc/CPU baseline、正式support matrix。
- package/crate/NuGet ID、PyPI/crates.io/private registry、catalog service/URL/API、credential、release owner。
- ABI headerの完全なsymbol宣言、language wrapperのAPI/loader、各package managerの完全なinstall hook/command実装。
- signature/hash algorithm、canonical serialization、manifest wire schemaを利用者承認前に確定仕様と扱うこと。
- root key保管、HSM、threshold signing、CI identity、trust metadataの有効期間/SLA、失効告知運用。
- build attestation、SBOM形式・集約pipeline、実測性能閾値、全OS/boardの実機検証仕様。必要な基盤契約と後続担当だけを示す。
- 要求case、4言語縦断機能試験、実行証跡の共通契約を本書へ含めること（Requirements Test Manifestと要求test対応表の担当）。

## レビューで確認する点

1. Build target順、ARM cross-buildとruntime/hardware verificationの区別がBD-16/22を保つ。
2. Python/NuGet/CMake/Cargo package境界がBD-17～20に一致し、native Coreを言語ごとに再buildしない。
3. 公式catalogとimmutable manifest/digestがnative artifactのcanonical identityを維持する。
4. target/profile/version/digestは明示・固定され、適合しないartifactへ推測fallbackしない。
5. 署名・trust・schema validation完了前にartifactをlink/load/buildへ渡さず、失敗時はfail closedとする。
6. runtime version、ABI major、wrapper version、manifest schema version、target profileの役割が混同されない。
7. build verified、runtime verified、hardware profile verifiedを別状態として表す。
8. Apache-2.0、third-party license維持、support matrix/EOLの承認範囲をpackageとrelease記録へ引き継ぐ。
9. exact toolchainやservice等の未決事項を実装都合だけで確定せず、該当するtest成果物または将来のrelease/implementation designへ担当を送る。

## 承認記録

2026-10-12: 利用者はDD-BUILD-01～11を承認。承認状態を詳細設計、作業計画、フェーズ一覧、進捗メモへ反映した。本文で後続へ送ったtoolchain、package/catalog運用、key custody、正式support matrix等の未決は維持する。
