# 0.0.1 Platform対象範囲の変更計画

- 計画版: 0.1
- 状態: 計画と関連する要求・設計・試験文書の改訂版は承認済み（利用者、2026-11-09）。VM確認・framebuffer実装計画・実装・release判断は未完了
- 対象version: `0.0.1`
- 上位計画: [0.0.1実装・リリース計画](release-0.0.1.md)
- 根拠要求: [Platformと組み込みLinux](../11_requirements/RQ-11_platform.md)、[ビルドと配布](../11_requirements/RQ-12_build-distribution.md)
- 根拠設計: [Platform abstraction](../13_detail_design/platform/abstraction.md)、[Platform internal layers](../13_detail_design/platform/internal-layers.md)
- 試験: [共通UI縦断・profile・配布物](../30_release_test/system-scenarios.md)、[requirements-test matrix](../30_release_test/requirement-test-matrix.md)

## 目的

実機準備の見通しが立たないARM/組み込みLinuxの本番機器固有確認を0.0.1のrelease gateから外す。Windowsに加え、Ubuntu x64とDebian 12 x64をVMware/VirtualBox等の仮想環境で、対象の描画・UI経路まで確認できる場合は0.0.1の対象候補に含める。その他のLinux targetはframebuffer経由の描画に限る狭い提供範囲を提案し、未確認profileを正式support済みと誤表示しない。

## 提案する対象区分

| 区分 | 0.0.1での扱い | 保証の境界 |
|---|---|---|
| Windows x64 | 正式対象候補 | Windows 11 x64でUI縦断・package consumer・可視描画を確認する |
| Ubuntu x64 | 正式対象候補 | Ubuntuの対象versionとVM/実機条件を固定し、UI縦断・package consumer・描画経路を確認する |
| Debian 12 x64 | 正式対象候補 | VMware/VirtualBox等のVMでLinux backendのdevice/surface取得から可視描画まで到達できるか確認する。VM上で確認できた範囲をprofile証拠にし、物理GPU固有の保証は含めない |
| その他のLinux target | framebuffer描画に限定した実験的な経路を0.0.1対象候補とする案 | 対象アーキテクチャ、pixel format、表示初期化、failure境界、文字描画、入力・IME・Accessibilityの対象外範囲を明記する。profileごとの実機証拠がないものは正式runtime support passと表示しない |
| ARMv7 / AArch64本番機器 | framebuffer限定経路とcross-build/fake testを0.0.1候補に含め、board runtime gateは後続へ分離する案 | hardware evidenceなしでは正式runtime supportと表示しない。framebuffer限定の実装・build/test範囲とboard上の可視動作を別statusにする |
| Linux DRM/KMS+GBM/EGL/GLES | 既存実装・試験証拠を保持しつつ、0.0.1の正式profile採用可否を再判断 | framebuffer限定案と同一視しない。採用しない場合もコード削除はこの計画だけで決めず、後続計画へ残す |

「Ubuntu x64」と「Debian 12 x64」の具体的な実行profileおよび「その他Linuxのframebuffer限定」の接続方式は未確定であり、既存Linux AdapterがUbuntu/Debianの通常desktop sessionを提供するとは仮定しない。framebufferがLinux fbdevを指すのか、DRM/KMS dumb buffer等も含むのかを、要求・設計変更案で定義する。VMで正式profile確認とみなす条件は、単なるguest OS起動やcross-buildではなく、対象artifactからWindow/surface生成、描画、可視出力までを確認できることとする。VM固有driverの制約と物理GPU/boardで未確認の範囲は別記する。

## 変更が必要な文書・要求

承認済み要求と設計を計画だけで上書きしない。以下を差分案として作成し、要求・設計・試験文書単位でレビューを受ける。

1. `RQ-11-001`、`RQ-11-002`、`RQ-11-003`、`RQ-11-014`〜`RQ-11-018`、`RQ-11-021`〜`RQ-11-024`を照合し、0.0.1正式対象、将来候補、framebuffer限定範囲を再定義する。
2. `RQ-12-010`と関連するtarget/package要求を更新し、Windows/Ubuntuのruntime gateと他targetのbuild-onlyまたは限定描画gateを区別する。
3. BD/DDのPlatform profile・backend選択・依存条件・未対応機能・failure境界を、承認されたRQ変更に合わせて改訂する。
4. `release-0.0.1.md`、`system-scenarios.md`、requirement-test matrix、CI/profile説明、README/導入資料を新しい区分へ揃える。VMでのx64 Linux profile確認と、ARM board本番環境でのhardware確認を別gateにする。ARM board実機確認、desktop LinuxのIME/Accessibility等、対象外へ移す項目は削除せず後続versionの残課題へ移管する。
5. Linux framebuffer backendが現行実装に存在しないため、実装の要否・責務・対象profile・最小test unitを承認済み設計後に別計画化する。既存DRM/KMS実装をframebuffer対応済みとは扱わない。

## 実施状況と次の手順

1. 完了: Windows/Ubuntu/Debian VM/ARM/framebuffer profile境界を含む要求・BD/DD/release plan・system scenario・requirement-test各文書を更新し、文書単位で承認を得た。
2. 未実施: Debian 12 VMで既存Linux artifactのdevice/surface取得から可視presentまでを確認し、guest image・hypervisor/version・virtual GPU/driver・候補SHA・結果を記録する。VM起動やheadless/pixel-readbackだけでは合格にしない。
3. 未実施: Ubuntu/Debianでの実行profileとbackendの成立条件を記録し、Debian VMは証拠が揃ったhypervisor/profileに限り対象化する。
4. 未実施: framebufferの方式、対象profile、pixel contract、failure/ownership、実装unit、fake-buffer testと実機表示証拠の境界を独立の実装計画にまとめてレビューに出す。
5. その計画の承認後、既存Windows/Linux CIとrelease profile matrixへ実装/test gateを反映し、unit単位に実装・検証する。
6. 候補SHAでWindows/Ubuntu/Debian VM profile、package consumer、UI表示・入力・回帰を検証する。ARM board等の未確認runtimeをpass扱いしない。

## 成果物

- 0.0.1正式対象・限定対象・将来対象を分けたprofile matrix
- Ubuntu/Debian VMでの描画経路成立条件と、VMで検証できないhardware特性の一覧
- 影響RQ/BD/DD/release-test文書の改訂案と承認境界
- framebuffer限定提供の保証・非保証と対象profile
- ARM/board/未検証Linux機能の後続version移管一覧
- 承認後の実装unit案、試験・CI gate・実機記録方法

## 承認境界

- 本計画と参照する要求・設計・試験文書の改訂版全体は利用者承認済み。過去版の承認記録は保持する。
- 文書承認はVM実証、framebuffer backend選定・実装、release合格、tag、公開を意味しない。
- 本計画の承認だけでは、framebuffer方式の選定、Linux backend実装、release合格、tag、公開を許可しない。
- 新規・改訂計画は利用者レビュー後にcommit/pushする。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-09 | 計画0.1 | 利用者承認 | Windows/Ubuntu正式対象、VMで描画経路を確認できるDebian 12 x64の対象候補、ARM等の本番機器固有gate分離、その他Linux framebuffer限定案の改訂計画を承認。RQ/設計の承認や実装開始は別 |
