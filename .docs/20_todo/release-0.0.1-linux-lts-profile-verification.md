# 0.0.1 Linux LTS profile追加・順次検証計画

- 計画版: 0.1
- 状態: 計画承認済み（利用者、2026-11-11）。RQ・設計・試験文書の変更、VM作成、実行証跡は未着手
- 対象version: `0.0.1`
- 親計画: [0.0.1 Platform対象範囲の変更計画](release-0.0.1-platform-support-scope-change.md)
- 対象要求: [RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[RQ-12 Build/Distribution](../11_requirements/RQ-12_build-distribution.md)
- 関連試験: [共通UI縦断・profile・配布物](../30_release_test/system-scenarios.md)、[requirements-test matrix](../30_release_test/requirement-test-matrix.md)

## 目的

0.0.1のLinux x64 runtime profile候補をUbuntu 24.04 LTSに加えてUbuntu 26.04 LTSへ拡張し、Debian 13 x64とDebian 12 x64のVM profileを候補へ加える。Debian 13を先に、Debian 12を次に検証する。Ubuntu 24.04/26.04とDebian 12/13は版別VMを使い、既存Ubuntu VMはこの計画では使用しない。各版の結果を独立したprofileとして記録する。

本計画の承認は、Debian 13/Ubuntu 26.04の要求・設計上の正式対象化やrelease合格を直接確定しない。要求・設計・試験文書の変更案をレビューし、承認履歴を更新してから確定する。検証結果がrelease criteriaを満たすまでは、profileを正式support passと表示しない。

## 実施順

1. 完了: VirtualBoxのglobal `GUI/Input/AutoCapture` を`true`に設定し、値を再読込確認した。Debian VMの物理キーボード操作はVMウィンドウへフォーカスを移して行い、右Ctrlでホストへ戻す。
2. 未実施: Debian 13 VMのOS版、kernel、VirtualBox版、VM CPU/RAM、VMSVGA設定、virtual GPU/driver、DRM device、desktop sessionを読み取り記録する。
3. 未実施: 現行branchの候補SHAでLinux artifactを作成・特定し、必要依存と起動条件を確認する。
4. 未実施: Debian 13でartifact load、backend/device/surface open、visible present、resize/redraw、close/reopenを確認する。画面での観測とAPI結果を分けて記録し、失敗・未対応も保持する。
5. 未実施: Debian 13の環境・artifact・結果・診断を別profile reportへ保存し、対象SHAと再現可能なコマンドを記録する。
6. 未実施: Debian 13 VM証拠を保存した後、Debian 12 x64用VM `freemvvm-debian12-validation` を1台作成する。Debian 13 VMは維持し、同じ版の重複VMは作成しない。
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
