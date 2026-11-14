# 0.0.1 Linux Wayland desktop backend 先行計画案

- 計画版: 0.6
- 状態: Wayland先行方針0.2、RQ-11/12改訂、RQ-09-076、UI BD 1.2、Platform BD 1.4/1.5、UI DD 1.8、Platform DD 2.9、計画0.6は利用者承認済み（2026-11-14）。実装計画・実装は未着手
- 対象version: 0.0.1
- 対象branch: change/platform-support-scope
- 関連計画: [Linux DRM/KMS runtime resize](linux-drm-runtime-resize.md)、[Linux LTS profile検証](release-0.0.1-linux-lts-profile-verification.md)、[0.0.1 platform support scope revision](release-0.0.1-platform-support-scope-change.md)
- 対象要求: [RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)
- 設計根拠: [Platform BD](../12_design/platform/abstraction.md)、[Platform DD](../13_detail_design/platform/internal-layers.md)
- test規約: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[UI test design](../32_unit_test/ui.md)、[release test matrix](../30_release_test/requirement-test-matrix.md)

## 背景と調査結果

現Linux runtime backendはDRM/KMS + GBM/EGL/GLESで、GNOME等のcompositor-managed desktop windowではなく、選択connector/CRTCを直接使用する。Debian 13 VirtualBoxのTTY確認では表示はできるが、desktop sessionから通常windowとして描画できる証拠はない。

VirtualBox VMSVGA調査では、1920x1080がDRM mode候補に存在し、VBoxDRMClientとvboxadd-serviceは起動していた。一方、host menuで1920x1080を選んでもactive DRM modeは1280x800のままだった。DRM hotplug ueventは通知であり、active modeを設定する利用者空間処理が別に必要となる。これは直接KMS経路の課題であり、desktop windowのsurface resizeとは別責務である。

Waylandではcompositorがxdg-shell configureを通知し、clientはconfigure serialをacknowledgeして新しいsurface bufferをcommitする。アプリは物理displayのKMS modeを直接設定しない。[xdg-shell protocol](https://wayland.app/protocols/xdg-shell)、[Linux DRM/KMS](https://docs.kernel.org/gpu/drm-kms.html)

2026-11-14のDebian 13 graphical terminal確認では`XDG_SESSION_TYPE=wayland`、`WAYLAND_DISPLAY=wayland-0`、`DISPLAY=:0`だった。`pkg-config`でEGL 1.5/GLES 3.2を確認し、`libegl-dev`、`libgles-dev`、`libwayland-egl1`、`pkgconf`は導入済みだが、`libwayland-dev`は未導入で`wayland-client`/`wayland-egl` metadataが見つからなかった。`libwayland-dev`はWayland library development headersのDebian packageである。[Debian libwayland-dev](https://packages.debian.org/trixie/libwayland-dev)

現workspaceのminimum Rustは1.85で、現在の`wayland-client` 0.31.14/0.31.15はRust 1.86を要求するため、dependency version selectionはMSRV gateに含める。Smithay Client Toolkit 0.20.0はRust 1.65を宣言しているが、semver依存で解決されるwayland-client版もlockで確認する。[wayland-client package metadata](https://docs.rs/crate/wayland-client/latest/source/Cargo.toml)、[Smithay Client Toolkit 0.20.0 metadata](https://docs.rs/crate/smithay-client-toolkit/0.20.0/source/Cargo.toml)

初期feasibilityではnative Wayland client + EGL Wayland surfaceを有力候補とする。既存`GlesRenderer`はGLES loaderを注入でき、Linux peerはnative surfaceごとのworkerを持つため、Core/C ABIとrendererを保った構成が見込める。Wayland clientを作成しxdg-shell configure/ackを処理し、`wayland-egl` surfaceと既存EGL/GLES rendererを接続する一方、DRM/GBMの直接表示経路とは分ける案である。[wayland-egl surface API](https://docs.rs/wayland-egl/latest/wayland_egl/struct.WlEglSurface.html)

SDL3はwindow/event abstraction候補として比較対象に残す。ただし既存backendが直接EGL context/surfaceとGLES loaderを管理しているため、SDL GL contextとのresource ownership、Wayland native handle interop、native library packagingを比較してから採否を決める。[SDL3 Wayland notes](https://wiki.libsdl.org/SDL3/README-wayland)

利用者は2026-11-14に「Waylandを先に進める」方針を選択し、続いてRQ-11/RQ-12の改訂案を承認した。Waylandは0.0.1 Linux desktop backend候補として要求に反映済みだが、これはprofileのformal runtime support合格や具体的な実装方式の承認を意味しない。

## 方針と範囲

1. 0.0.1のLinux desktop経路はWaylandを最初に調査・設計・実装する。通常のdesktop sessionでWindowを開き、visible present、compositorからのresize、Core再Layout・再描画、close/reopenを検証する。
2. Linux DRM/KMS direct-displayはconsole profileとして別扱いに保つ。KMS runtime resize実装はWayland desktop backendの方針と必要性が確定するまで保留する。既存TTY証跡はconsole profileに限定し、Wayland desktop passへ流用しない。
3. 0.0.1 formal runtime profileとWayland support statusは、承認済みRQ-11/12のprofile別evidence gateに従って確定する。要求上の候補化だけで正式supportと表示しない。
4. 実装方式はPlatform DD 2.9改訂案でnative Wayland client + EGL surfaceを推奨候補としている。文書全体承認後に採用とし、exact crate versions/system linkingはRust 1.85・Debian/Ubuntu package gateを通した実装計画で確定する。
5. C ABIと各language wrapperの公開APIは変更しない案を初期値とする。既存backend選択契約へWayland backendを接続できない場合は、source変更前に設計差分を提示する。

## 実施順

### A. feasibilityとprofile確認

1. Debian 13 VMのgraphical session種類、Wayland socket、compositor、EGL/GLES、必要dev/runtime packageを読み取り確認する。通常desktop sessionから実行できる状態を作り、sudoやdisplay manager停止を利用条件にしない。
2. Ubuntu profileについて、親support-scope planの対象版・VM準備状況・Wayland sessionを照合する。未準備profile用VMを重複作成しない。
3. Rust native Wayland client構成とSDL等の既存抽象化libraryを比較する。Window create/present、configure/ack、surface buffer更新、close、input、DPI/scale、既存GLES rendererとの接続、build/link配布条件、license、CI testabilityを記録する。
4. compositorがconfigureを送るだけで物理mode設定を保証しない境界、Wayland server/desktop integrationの依存と明示failureを記録する。

### B. 要求・設計レビュー案

1. 完了: RQ-11 v0.8/RQ-12 v0.7の改訂案を利用者承認後に正本へ適用し、Wayland desktop candidateとprofile別証拠ゲートを定義した。
2. 完了: Platform BD 1.4を利用者が文書全体承認。Linux desktop compositor-managed surfaceとdirect KMS console profileの責務を分離した。
3. 完了: Platform DD 2.9を利用者が文書全体承認。Wayland lifecycle/presentとWindow初期client sizeの希望値・実寸境界を定義した。
4. 完了: RQ-09-076、UI BD 1.2、Platform BD 1.5、UI DD 1.8を利用者が文書全体承認。Window生成時の初期client size設定と省略時800×600 DIPを確定した。
5. 現在: 実装unit・unit-test・release test matrix・VM visible testをまとめた実装計画案を作成し、利用者レビューへ提示する。実装計画承認前に依存追加・source/test変更をしない。

### C. 実装・検証の候補分割

設計承認後のimplementation planでは、責務を少なくとも次の実装単位へ分け、各unitに同revisionのtestを対応させる。

1. Wayland connection/window lifecycleと明示failure、native EGL surface作成・破棄。
2. Window初期client size設定のCore保管・wrapper/C ABI設計接続と、xdg-shell configure状態機械、serial ack、0寸法軸での希望値使用、buffer resize/recreate、Core client-area sink通知。
3. Core scene render/present/frame completionをWayland surfaceへ接続し、close/reopenを実装。
4. 承認されたinput/close/scale要件を順に接続し、必要なunit testを追加する。
5. Debian 13およびRQで承認されたUbuntu profileのWayland session上で、同一artifact SHAのvisible window/present/resize/close/reopenを確認する。build成功やTTY表示はdesktop runtime passとしない。

## 合格境界

- compositorから届くconfigureを最新serial規則に従って処理し、ack前に未設定bufferをcommitしない。
- Window初期client sizeを未指定時800×600 DIPで開始要求し、configureの正値または0寸法時の希望値から決まった実寸をCoreへ通知する。無効値はWindow生成前に拒否する。
- active Window generationのclient-areaをCoreへ通知し、Core layout/scene extentとpresent buffer pixel extentを正しく対応させる。
- resize連続通知は未開始frameへ集約し、old generationや古いconfigureへの遅延responseが新状態を上書きしない。
- surface/renderer更新中およびfailure後に不正なframeをPresentedとして報告しない。Window close/reopenとRuntime shutdownでnative resourceを一度だけ破棄する。
- Wayland visible testは指定sessionの配布artifactで実施する。特定VMでの合格を別OS版、別compositor、物理GPUへ一般化しない。
- formal support表示は承認済みRQの対象profileと対応release caseがすべてpassしたprofileに限る。

## 実施しないこと

- 今回のWayland計画から直接KMS runtime resizeの完了を推定すること。
- X11 backend、multi-output、runtime backend切替、異なるGPU間移行、未承認のDPI/scale変更保証。
- Wayland対応だけを根拠にIME、Accessibility、window decorations等の未検証範囲をsupport済みにすること。
- 利用者が要求・設計・implementation planを承認する前の要求/設計正本変更、実装、依存追加、test登録、commit/push。

## 承認境界

- 本文はWaylandを0.0.1 Linux desktopの先行実装経路とし、feasibility、RQ/BD/DD review、実装計画の順に進む作業計画案である。
- 利用者がこの計画本文を承認するまでは、依存追加、source/test変更、実装unit登録、commit/pushを行わない。
- RQ、BD、DDは各文書全体の承認を得てから次段へ進む。既存承認範囲と不整合が見つかった場合は差異と改訂対象を先に示す。
- 実装unitの対応test合格後は日本語commit/pushし、Wayland計画の関連TODO完了後にまとめてmerge PRを出す。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-13 | 計画0.1 | Waylandを後続候補とする案 | RQ/BD/DDは未変更。KMS resizeは別profile計画 |
| 2026-11-14 | 計画0.2 | 利用者承認 | Wayland先行を選択。Wayland正式support、実装方式、要求対象範囲は未決。実装未着手 |
| 2026-11-14 | 計画0.3 | 適用記録 | Debian 13 Wayland sessionとEGL/GLES runtime/dev、Wayland EGL runtimeを確認。`libwayland-dev`未導入。native Wayland + EGL案をfeasibility有力候補として記録 |
| 2026-11-14 | 計画0.4 | 利用者承認 | RQ-11 v0.8/RQ-12 v0.7適用済み。Platform BD 1.4を承認 |
| 2026-11-14 | 計画0.5 | 更新案・レビュー待ち | Platform DD 2.9案と初期寸法sourceの未決gateを反映 |
| 2026-11-14 | 計画0.6 | 利用者承認 | Window初期client size設定と省略時800×600 DIP、Platform実寸との境界を含むRQ/UI/Platform BD/DDおよび実装計画前の設計gateを承認。実装計画/実装は未着手 |
