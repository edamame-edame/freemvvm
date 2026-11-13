# 0.0.1 Linux Wayland / Debian profile 要求改訂案

- 計画版: 0.1
- 状態: 要求改訂案0.1は利用者承認済み（2026-11-14）。RQ-11/RQ-12へ適用済み。BD/DD/test正本は未変更、実装未着手
- 対象version: `0.0.1`
- 親計画: [Linux Wayland desktop backend先行計画](linux-desktop-backend-scope-proposal.md)、[Linux LTS profile追加・順次検証計画](release-0.0.1-linux-lts-profile-verification.md)
- 対象要求: [RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[RQ-12 Build/Distribution](../11_requirements/RQ-12_build-distribution.md)
- downstream: [Platform BD](../12_design/platform/abstraction.md)、[Platform DD](../13_detail_design/platform/internal-layers.md)、[release test matrix](../30_release_test/requirement-test-matrix.md)、[requirements test assertions](../30_release_test/requirements-test-assertions.md)、[system scenarios](../30_release_test/system-scenarios.md)

## 背景

現在のRQ-11/RQ-12はDebian 12 VMを候補とし、Waylandを後続Linux desktop候補としている。2026-11-14にDebian 13のgraphical sessionがWayland (`XDG_SESSION_TYPE=wayland`, `WAYLAND_DISPLAY=wayland-0`) で動作すること、EGL 1.5/GLES 3.2、`libwayland-egl1` runtimeが存在することを確認した。一方、`libwayland-dev`は未導入であり、Wayland client development metadataは未確認。

利用者の既承認方針はDebian 13を先に対象検証し、Debian 12をその後に進めること、およびWayland desktop経路を先行させることである。Ubuntu 24.04/26.04も既承認計画のprofile候補とする。各profileは独立証拠が揃うまでformal runtime support passとしない。

現runtime実装はDRM/KMS direct-displayであり、通常のdesktop window経路ではない。Wayland desktopではcompositor managed windowとして描画し、通常desktop sessionから利用可能にする。Waylandの選定・実装順だけでX11、IME、Accessibility等をsupport済みとは扱わない。

## 改訂対象と提案文

以下をRQ-11 v0.8 / RQ-12の次版へ反映する提案。profile順序と受入条件を要求本文に明記し、API/library選定は要求で固定しない。

| 要求 | 提案する要求詳細 | 対象段階 |
|---|---|---|
| RQ-11-003 | Linux x64を開発・CI targetとする。0.0.1のLinux runtime profile候補はUbuntu 24.04 x64、Ubuntu 26.04 x64、Debian 13 x64、Debian 12 x64とする。Debian 13を先に検証し、Debian 12はその後に検証する。各OS版・hypervisor/virtual GPU・desktop session/backendを独立profileとして記録し、実画面での描画確認に合格したprofileだけをruntime verifiedとする。OS起動、build成功、別profileのpassを代用しない。 | 0.0.1候補 |
| RQ-11-014 | Windows 11および選定Linux runtime profileごとに、desktop/console profile、描画backend、依存条件を識別すること。Wayland desktop profileはcompositor-managed windowとして通常desktop sessionから使用し、DRM/KMS direct-displayは別console profileとして識別する。 | 0.0.1候補 |
| RQ-11-015 | Waylandを0.0.1 Linux desktop向けbackend候補とする。Wayland desktop session上でWindow生成、可視描画、compositorからのresizeに対する再layout/再描画、close/reopenを検証する。正式runtime supportは対象OS版・VM/virtual GPU・session/backendごとのrelease evidenceが揃ったprofileに限る。X11対応やTTY/KMS consoleからのdesktop supportを推定しない。 | 0.0.1候補 |
| RQ-11-023 | Windows 11、Ubuntu 24.04/26.04、Debian 13/12 VM、framebuffer限定各profileについてfont/文字対応、日本語IME条件、Accessibility provider有無を識別すること。Wayland描画の確認だけでIME/Accessibility passとしない。framebuffer描画のみのprofileをIME/Accessibility対応済みと表示しない。 | 初期共通基盤 |
| RQ-11-024 | 正式runtime profile候補はWindows 11 x64、Ubuntu 24.04/26.04 x64、Debian 13/12 x64の個別profileとする。Linux profileはWayland desktop sessionでの可視描画を候補受入条件とし、各profileのrelease evidenceが合格したものだけをformal runtime supportとして表示する。Debian 12をDebian 13の証拠でpass扱いしない。Linux ARMv7/AArch64のframebuffer限定範囲とhardware gateは従来どおり分離する。 | 0.0.1候補 |
| RQ-12-010 | CIでWindows x64とLinux x64をbuild/testする。0.0.1 Linux runtime candidateはUbuntu 24.04/26.04およびDebian 13/12の版別profileとし、Wayland sessionでartifact loadから可視描画・resize/redraw・close/reopenまで確認する。Debian 13を先に、Debian 12を次に実施する。profile evidenceにはexact OS/kernel、compiler、Wayland session/compositor、hypervisor/virtual GPU、driver、backend、artifact digestを記録する。ARM build/fake testとboard runtime gateは従来どおり分離する。 | 0.0.1候補 |

## test文書の同時変更範囲

要求承認後にRQ-11/RQ-12へのtraceabilityを保って同時改訂する対象。

- `TC-TARGET-LNX-01`と`TC-TARGET-LNX-VM-01`をUbuntu 24.04、Ubuntu 26.04、Debian 13、Debian 12の独立profile reportへ割り当てる。各profileは同一caseを使っても別実行・別SHA/evidenceにする。
- `TC-ARM-BACKEND-02`は名称と対象が食い違うためWayland desktop caseとして識別可能なID/説明へ改称し、すべての参照先を更新する。Wayland session open/present/resize/close/reopenをdesktop runtime oracleにする。
- `requirements-test-matrix.md`、`requirements-test-assertions.md`、`system-scenarios.md`、release plan/matrixのDebian 12限定記載を4 Linux版別profileへ揃える。
- 証跡の無いVM、TTY-only描画、headless pixel testはWayland desktop runtime passにしない。OS版/profile別にpass/fail/blocked/not-runを記録する。

## downstream設計変更候補

RQ承認後、BD/DD全文案として別途提示する。以下は変更範囲の事前特定であり、設計決定ではない。

- Platform BD: compositor-managed desktop windowとDRM/KMS console profileの責務、session依存とCoreへのclient-area/scale通知を区別する。
- Platform DD: Wayland connection/window lifecycle、xdg-shell configure/ack、EGL surface/buffer ownership、frame callback/present、resize generation、close-request、error/cleanupを詳細化する。既存GLES rendererを再利用する案はDDで比較・確定する。
- dependency/package、X11/TTY fallback、Wayland session不在時のfailureを明記する。Wayland session不在時にKMS/X11へ暗黙fallbackしない。
- 要求・設計承認後にunit test / release test / Debian13 visible VM testの実装計画を作成する。要求または設計レビュー前にsource/dependency/testを変更しない。

## 承認境界

- この文書はRQ-11/RQ-12の本文改訂案であり、RQ正本・BD/DD・test文書は未変更。
- 利用者がこの改訂案全体を承認した後にのみRQ本文と承認履歴を更新する。
- RQ承認後もBD、DD、test文書は各文書単位のレビュー・承認後に確定する。実装、VMパッケージ変更、release support宣言は別の承認境界として保持する。
- Wayland実装方式、crate/library、C ABI/API変更はRQ本文では確定しない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-14 | 改訂案0.1 | 利用者承認・RQ適用済み | Debian 13先行・Debian 12後続、Ubuntu 24.04/26.04、Wayland desktop候補をRQ-11/RQ-12へ反映。BD/DD/test正本は別レビュー、実装未着手 |
