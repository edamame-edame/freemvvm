# Linux DRM/KMS runtime resize 実装計画案

- 計画版: 0.1
- 状態: 保留。Wayland desktop経路を先行し、本計画の実施要否・順序はその後に再評価する。未承認、実装未着手
- 対象version: `0.0.1`
- 対象branch: `change/platform-support-scope`
- 親計画: [0.0.1 Linux LTS profile追加・順次検証計画](release-0.0.1-linux-lts-profile-verification.md)
- 対象要求: [RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)
- 承認済み設計: [Platform DD](../13_detail_design/platform/internal-layers.md) 1.5 client-area通知・resize、1.7 Linux KMSの1:1 client-area/device-scale
- 対応release test: [TC-TARGET-LNX-VM-01](../30_release_test/requirement-test-matrix.md#case-tc-target-lnx-vm-01)
- unit-test設計: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[UI test design](../32_unit_test/ui.md)

## 背景

2026-11-14に利用者がLinux desktop経路としてWaylandを先行する方針を選択したため、本計画の直接KMS runtime resizeは先行実装対象から外し保留する。Wayland window resizeとKMS console mode resizeは別profile・別責務であり、一方の実装/合格で他方を代替しない。

Debian 13 VirtualBoxでLinux DRM/KMS + GBM/EGL/GLES artifactのvisible present、背景redraw、close/reopen後の再presentを確認した。画面modeがopen時に固定され、runtimeのconnector/mode変更をLinux peerがCoreへ通知する経路は現状ない。承認済みrelease test `TC-TARGET-LNX-VM-01` はresize/redraw確認を含む。

Coreには`PlatformWindowClientAreaSink`と、generationを照合してLayout/frameをdirtyにする処理がある。Linux peerはsinkを登録・通知しておらず、DRM surfaceもopen時のmodeサイズで作成している。

## 設計根拠と境界

- RQ-11-009: platform backendをRust Coreから分離する。
- RQ-11-020/022: 表示資源喪失後もCoreの確定済みUI stateを保持し、失敗frameをPresentedとして報告しない。
- RQ-09-067: logical client-areaとDeviceScaleを区別する。
- Platform DD 1.5（2026-10-29全文承認）: peerはgeneration付きclient-area changeをUI dispatcherへ送り、Coreはroot Layout/frameをdirtyにする。zero-areaは有効で、native surfaceが処理できない場合はsubmitを止める。
- Platform DD 1.7（2026-10-31全文承認）: Linux KMS profileのDeviceScaleは1:1で、open generation内では不変。物理DPI/HiDPIとscale変更は対象外。
- Platform DD 2.8（2026-11-10全文承認）: backendはWindow寿命中固定。resizeは同じpeer/backend内で処理し、backend切替は行わない。
- C ABI、wrapper公開API、Linux window manager integration、Wayland/X11、multi-output、DPI migrationはこのunitで変更しない。

## 実装方針案

DRM connectorの変更通知を受けて現行選択outputを再probeし、同じ接続outputのclient pixel extentが変わった場合にsurfaceを安全に更新する。通知元はLinux DRM ueventを監視する`udev` monitorを第一候補とし、Debian/Ubuntu build・runtime依存（`libudev`）とVirtualBox VMSVGAで実際にmode変更通知が届くことを実装前のfeasibility gateで確認する。kernelはconnector変更をuserspace ueventで通知し、userspaceにmode再設定を委ねる。通知が届かない場合に定周期でforce-probeする方式は採らず、理由と代案をレビューへ戻す。

処理と所有は既存Linux window workerへ集約する。

1. `LinuxPeer::set_client_area_sink`でCore sinkをworkerへ渡す。workerの通常command、pending page-flip完了、DRM ueventを一つのowner thread上で処理する。
2. ueventは再probeの合図として扱い、イベント中のconnector情報を正本にしない。現在の選択card/connectorを再probeし、最新の接続状態・mode・pixel extentを確定する。連続通知は最新状態へcoalesceする。
3. mode変更中は保留中page-flipを完了させてからGBM/EGL surfaceを更新し、新しいmodeに対応するCRTC/framebufferを設定する。初回open時に保存したCRTC状態はclose時まで保持し、resize中に復元値を上書きしない。
4. positive extent適用後、同じWindow generationの`WindowClientArea`をDIP 1:1としてCore sinkへenqueueする。CoreがLayout/Sceneを更新した後に届くframeだけを新surfaceへpresentする。
5. outputが一時的に未接続またはextent zeroの場合はzero client-areaを通知し、native zero-size surfaceを作らずframe submitを止める。positive extentへの復帰ではsurfaceを再作成し、client-area通知後のframeを再開する。
6. resource再作成失敗では新しいareaを成功通知しない。Core UI stateを維持し、該当frameをfailureにする。古いgenerationの遅延通知はCore既存処理により破棄する。

## 作業範囲

1. Debian 13 VirtualBoxで、VirtualBox displayの解像度変更時にkernel DRM ueventが出ること、selected connectorのmode/extentが再probeで変わることを確認する。`udevadm monitor --kernel --property --subsystem-match=drm`等で読み取り確認し、出力と実操作をevidenceへ保存する。
2. event source、Rust `udev` crate/libudev link要件、Debian/Ubuntu CI・Docker・runtime package条件を確認する。既存Linux profileの静的link/shared library方針と矛盾がないことを確認する。
3. `UNIT-PLATFORM-004`を追加し、Linux workerへmode-change source、sink登録、mode再probeとsurface updateを実装する。公開C ABIとwrapperは変更しない。
4. `UT-PLATFORM-RESIZE-001`を追加し、fake event source/surfaceで以下を検証する。
   - 同一extent eventは重複通知しない。
   - positive resizeは同じgenerationの最新client-area通知を一度enqueueし、pending flipを壊さない。
   - event burstは最新modeへcoalesceし、中間の未開始Layout/frameをsubmitしない。
   - zero-areaではsurface作成/submitを止め、復帰eventで再作成と再描画を行う。
   - resize resource failureは成功area/Presentedを通知せず、close/reopenとCRTC restoreを安全に行う。
   - close後のstale eventとsink enqueue failureで旧Window/Core stateを変更しない。
5. Linux focused test、`cargo fmt`、Clippy、workspace build/test、Rust 1.85 minimum、文書link checkを同一revisionで確認し、Windows/Linux CIの同一SHAを確認する。
6. Debian 13 TTYでsmokeを拡張し、Window表示中にVirtualBoxの仮想画面解像度を変える。背景/sceneが新extentに合わせて再描画され、zero-area pause/recovery、close/reopenが成立するか目視し、kernel/deviceログを保存する。Debian 12では親profile planに従い別profile証跡を取る。

## 合格条件

- Coreへ通知されたclient area、Layout/Scene extent、surface pixel extentが新しいmodeで一致し、redraw後に切り詰め・古い寸法の残像がない。
- `Presented`は新surfaceへの対応frame completion後だけに進む。resize途中・failure frameを成功扱いしない。
- close/reopen時に元CRTC状態が一度だけ復元され、surface/GBM framebuffer/page-flip資源のownerと破棄順を維持する。
- Debian 13 VMで実画面のresolution changeを確認する。fake testだけでTC-TARGET-LNX-VM-01のresize evidenceをpassにしない。
- resize成功はDebian 13 VirtualBox profileに限定して記録し、Debian 12・Ubuntu・物理GPU・別hypervisorの結果へ外挿しない。

## 実施しないこと

- Wayland/X11のnative window作成、window decorations、multi-window/multi-output routing。
- HiDPI、device-scale変更、monitor間migration、異なるGPU/deviceへの切替。
- `Window` object寿命中またはclose/reopenをまたいだPresentationBackend切替。
- DRM ueventが届かないprofileでの周期的force-probe、暗黙software fallback。
- resize以外の要求や公開API変更。

## 承認境界

- 本書はPlatform DD 1.5/1.7/2.8の承認済み契約をLinux DRM/KMSへ実装する計画案であり、実装開始前のレビュー待ち。
- 実装中にDDの意味・外部保証・失敗回復範囲を変える必要が出た場合は、実装を止めてDD改訂案を別途提示する。
- 利用者が本計画を承認するまで、`UNIT-PLATFORM-004`の実装・dependency追加・test登録・commit/pushは開始しない。
- 実装unitの対応testが通った後に日本語commit/pushし、親Linux profile planの関連TODOが完了した段階でまとめてmerge PRを出す。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-13 | 計画0.1 | 利用者レビュー待ち | 利用者の「0.0.1に含める。DD案を先にレビューし、承認後に実装計画を作成」に対応。DD 1.5 resize契約は既に全文承認済みのため、本書を実装計画案として作成。実装未着手 |
| 2026-11-14 | 実施順変更 | 保留 | Wayland desktop backendを先行する利用者判断により、直接KMS runtime resizeはWayland経路確定後に再評価する。計画本文の実装・依存追加・test登録は未承認 |
