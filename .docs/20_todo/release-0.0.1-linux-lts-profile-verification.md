# 0.0.1 Linux LTS profile追加・順次検証計画

- 計画版: 0.1
- 状態: 実行中。Debian 13の環境・visible-present証跡を記録。現行Linux backendにruntime resize経路がなく、resize確認は未解決。Debian 12/Ubuntu profileは未着手
- 対象version: `0.0.1`
- 親計画: [0.0.1 Platform対象範囲の変更計画](release-0.0.1-platform-support-scope-change.md)
- 対象要求: [RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[RQ-12 Build/Distribution](../11_requirements/RQ-12_build-distribution.md)
- 関連試験: [共通UI縦断・profile・配布物](../30_release_test/system-scenarios.md)、[requirements-test matrix](../30_release_test/requirement-test-matrix.md)

## 目的

0.0.1のLinux x64 runtime profile候補をUbuntu 24.04 LTSに加えてUbuntu 26.04 LTSへ拡張し、Debian 13 x64とDebian 12 x64のVM profileを候補へ加える。Debian 13を先に、Debian 12を次に検証する。Ubuntu 24.04/26.04とDebian 12/13は版別VMを使い、既存Ubuntu VMはこの計画では使用しない。各版の結果を独立したprofileとして記録する。

本計画の承認は、Debian 13/Ubuntu 26.04の要求・設計上の正式対象化やrelease合格を直接確定しない。要求・設計・試験文書の変更案をレビューし、承認履歴を更新してから確定する。検証結果がrelease criteriaを満たすまでは、profileを正式support passと表示しない。

## 実施順

1. 完了: VirtualBoxのglobal `GUI/Input/AutoCapture` を`true`に設定し、値を再読込確認した。Debian VMの物理キーボード操作はVMウィンドウへフォーカスを移して行い、右Ctrlでホストへ戻す。
2. 完了: Debian 13.7、kernel `6.12.116+deb13-amd64`、x86_64、Host VirtualBox `7.1.10r169112`、VM 4 vCPU/4096 MiB、VMSVGA/128 MiB/3D on、VMware SVGA II `[15ad:0405]`、`vmwgfx`、DRM device、root/TTY実行を記録した。
3. 完了: source HEAD `480d8f2f80ab0b2098eaa90b42f9d9ebf2eda045`のLinux debug artifactを作成・特定。artifact SHA-256とRust/Cargoをprofile recordへ記録した。
4. 一部完了: artifact load、peer open、red/blue redraw、close/reopen後のgreen visible presentを確認。画面上の色変化を利用者が確認した。resizeは未確認。現行Linux backendはopen時にmode/surface sizeを固定し、runtime mode/client-area changeを扱わないため、profile testのresize項目は未解決の実装gapとして扱う。kernelに`vmwgfx` host message channel errorが出たため診断として記録した。
5. 完了: [Debian 13 VirtualBox visible-present profile record](../01_memory/platform-debian13-vbox-visible-smoke.md)へ環境・artifact・結果・診断・再現コマンドを保存した。runtime resize未対応の実装gapも明記。
6. 未実施: Debian 13のresize gapを0.0.1で実装するか、固定modeのDRM/KMS profileとして試験対象外にするかを設計・試験範囲としてレビューする。その後Debian 12 x64用VM `freemvvm-debian12-validation` を1台作成する。Debian 13 VMは維持し、同じ版の重複VMは作成しない。
7. 未実施: Debian 12 VMで同じ確認手順を実行し、Debian 13と独立したprofile reportを保存する。
8. 未実施: Ubuntu 24.04 LTS x64用の新規VM `freemvvm-ubuntu24-validation` を作成し、候補SHAで配布artifact、UI縦断、runtime、visible presentを確認する。既存の`ubuntu` VMは使用しない。
9. 未実施: Ubuntu 26.04 LTS x64用VM `freemvvm-ubuntu26-validation` を作成し、同じrelease scenarioを実行する。使用した26.04 point-release imageとdigestを記録し、Ubuntu 24.04と独立したprofile reportを保存する。
10. 未実施: 全profileの結果からRQ・BD/DD・release scenario・release matrixの改訂差分を作り、文書単位でレビュー・承認を受ける。未合格項目を後続課題として残す。

## 合格証拠

Ubuntu 24.04、Ubuntu 26.04、Debian 12、Debian 13の各profileについて以下を個別に記録する。

- OS name/version/architecture、kernel、desktop session
- VirtualBox host version、VM CPU/RAM、virtual graphics controller/VRAM/3D設定
- guest virtual GPU、kernel driver、DRM node、実行ユーザーとdevice access
- source SHA、Rust/toolchain情報、artifact filename・SHA-256、wrapper/package経路
- device/surface/backend選択と初期化結果
- 画面上で観測した描画、resize/redraw、close/reopenの結果
- 実行コマンド、標準出力/標準エラー、failure diagnostic、report path

VM起動、build、headless GLES pixel test、またはDRM nodeの存在だけをvisible-present passとして扱わない。Debian 13の成功をDebian 12へ、Ubuntu 24.04/26.04間へ、またはVM結果を物理GPU・別hypervisor・ARM boardへ外挿しない。Docker仮passとVM visible-present結果を同じ証拠として集計しない。

## 文書変更候補

- `RQ-11-003`、`RQ-11-014`、`RQ-11-021`〜`RQ-11-024`: Ubuntu 24.04/26.04とDebian 12/13のprofile範囲、版別状態、独立証拠要件を提案する。
- `RQ-12`のLinux artifact/profile要求: Ubuntu 24.04/26.04とDebian 12/13それぞれの配布artifact条件とruntime statusの対応を提案する。
- Platform BD/DD: distro版・hypervisor/virtual GPU・backend依存のprofile matrixと保証境界を提案する。
- `system-scenarios.md`、`requirement-test-matrix.md`、`requirements-test-manifest.md`、release plan: Ubuntu 24.04/26.04とDebian 13/12を別scenario/profileとして追跡する差分を提案する。
- READMEやsupport表: 承認後にのみ版別statusを反映する。VM testの成否だけで他profileのstatusを変更しない。

既存のDebian 12承認範囲は履歴として保持し、Debian 13追加を既存文書へ黙って書き込まない。新たなbackend・API・描画方式の変更が必要と判明した場合は、別の設計レビューと実装計画へ分ける。

## 承認境界

- 本計画はDebian 13を先に、Debian 12を次に、Ubuntu 24.04/26.04を版別に検証する手順のレビュー案である。
- 対象VMはUbuntu 24.04、Ubuntu 26.04、Debian 13、Debian 12の各1台。既存の`ubuntu` VMは使わず変更しない。同一profileの重複VMは作らない。
- Debian 13とUbuntu 26.04を0.0.1正式対象へ追加するには、影響するRQ/BD/DD/test文書全体の変更案を別途提示し、文書単位で承認を受ける。
- 本計画・文書承認・VM実行のいずれも、個別profileの合格や0.0.1 release合格を自動的に意味しない。
- 計画のcommit/pushは利用者レビュー後に行う。実装や証拠記録は承認された範囲に従い、unitの検証完了後に日本語commit/pushする。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-11 | 計画0.1 | 利用者承認 | 「ok」によりDebian 13→12、Ubuntu 24.04/26.04のprofileを版別VMで検証する計画を承認。要求・設計文書変更、VM作成、試験実行、release合格は別 |
