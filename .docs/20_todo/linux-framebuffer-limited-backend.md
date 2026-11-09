# Platform別描画backend既定値・Linux framebuffer限定profile計画案

- 計画版: 0.2 review draft
- 状態: 計画0.2は利用者承認済み（2026-11-09）。profile別default/override policyのDD案とframebuffer方式・実装unitの承認前。実装未開始
- 対象version: `0.0.1`
- 上位計画: [0.0.1 Platform対象範囲変更](release-0.0.1-platform-support-scope-change.md)、[承認済みscope改訂](release-0.0.1-platform-scope-revision-proposal.md)
- 要求: [RQ-11 Platform](../11_requirements/RQ-11_platform.md) RQ-11-001、002、017、021〜024
- 設計: [Platform BD](../12_design/platform/abstraction.md) 1.2、[Platform DD](../13_detail_design/platform/internal-layers.md) 2.6
- test: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[UI unit-test design](../32_unit_test/ui.md)、[requirements-test matrix](../30_release_test/requirement-test-matrix.md)

## 1. 目的

既存Platform Adapterの開発済み・検証済み経路を置き換えず、各runtime profileに適したbackendを規定値にする。ARMv7/AArch64を含む追加Linux targetでは、通常desktop windowing/input/IME/Accessibilityを含まない限定経路を別profileとして検討する。fake pixel bufferの結果と実profile上の可視描画を別statusで記録し、未確認architectureや機器をruntime supportedと表示しない。

## 2. 現状と前提

- `crates/platform-linux` の現行NativeSurfaceはDRM/KMS+GBM/EGL/GLESを使い、Linux direct-display profileとして動く。KMS framebuffer objectの利用を、汎用Linux framebuffer backendの存在とは扱わない。
- 現行RendererはSceneSnapshotのClip、FillRect、GlyphRunをGLESへ渡し、未知/未対応draw operationはframe failureにする。Linux text serviceは既存GlyphRunからglyph maskを解決する。
- WindowsにはVirtualBox 7.1.10が標準パスにあり、既存Ubuntu VMとは別にDebian 12.15 x64の検証VMを作成した。VMは4 vCPU/8 GiB、VMSVGA 128 MiB、3D acceleration on。guestはkernel `6.1.0-54-amd64`、virtual GPU `VMware SVGA II [15ad:0405]`、`vmwgfx` driverで、`/dev/dri/card0`と`renderD128`が存在する。既存Ubuntu VMは変更していない。
- 候補source SHA `eeb2dc322ef9667170028f010c812f13361fbf3b`をVM内でbuildした。`tools/linux_platform_docker.sh`相当のLinux検証はCore 203件、Linux Platform 21件、headless GLES pixel test 2件、Linux check/ClippyおよびRust 1.85 checkがpassした。headless試験はsurfaceless software renderingであり、可視出力の証拠とは分離する。
- VM上の既存DRM/KMS+GBM/EGL/GLES経路で、Python wrapperからC ABI debug libraryへ接続し、`runtime_dispatch_turn`をpumpしながら背景色frameを表示した。赤系frameのvisible presentと、`close_peer`後に青系frameへ再open/presentする表示を画面で確認し、closeも成功した。artifact SHA-256は`abb2d5fecfc737e4f2f5f11edf08a5373879815cc61e5b2ec0febecbb52c953f`。smokeはrootで実行しGDMを一時停止したため、配布package consumer、一般ユーザー権限、input、window resize、別VM/hypervisorの合格証拠ではない。`vmwgfx`の`Failed to open channel` kernel diagnosticも記録されたが、2色の可視表示は成立した。
- 詳細な実行条件・ログと設計境界は[Debian 12 VirtualBox visible-present記録](../01_memory/platform-debian-vbox-visible-smoke.md)を参照する。通常sandboxからの`VBoxManage`はCOM `E_ACCESSDENIED`となったため、必要なVM操作は権限付きで実行した。ISOのSHA512はDebian公式SHA512SUMSと一致した。
- 承認済みRQ/DDはframebufferの実装方式、pixel format、stride、orientation、flush/present、resource ownerを未確定としている。これらを実装上の都合だけで暗黙確定しない。

## 3. 提案する構成

### Platform別の規定backend

全profile共通のsoftware renderer/backendへ置き換えない。検証対象profileごとに利用するSurfaceとRendererを定め、利用者が未指定ならそのprofileの規定値を選ぶ。利用者が対応済み候補を明示指定した場合は、その候補がprofile/surface/scene capabilityを満たすときだけ使う。未登録・非互換・初期化失敗の場合は明示failureとし、別backendへ黙ってfallbackしない。この選択は要求[RQ-11-022]のbackend明示・暗黙fallback禁止に従う。

| runtime profile | 既定backend方針 | 根拠・適用境界 |
|---|---|---|
| Windows 11 x64 | 既存Win32 peer + Direct2D/DirectWriteを維持 | 承認済みWindows Platform設計と既存adapterを基準にする。他Windows版へ一般化しない |
| Debian 12.15 x64 / VirtualBox 7.1.10 VMSVGA | 検証した既存DRM/KMS + GBM/EGL/GLESを維持 | 本VMで色付きframeのvisible presentとclose/reopenを確認。このhypervisor・virtual GPU・guest driverの組だけに対応する |
| Ubuntu 24.04 x64 | 既存Linux DRM/KMS adapterを現profileの既定候補として維持 | Ubuntuのsession/profile別device/surface/present evidenceを確認し、desktop session向け別adapterを用意する場合は独立候補として比較する |
| ARMv7/AArch64・その他Linuxのframebuffer限定profile | 承認後に選ぶ当該profile専用backend | 既存の検証済みprofileを置き換えない。framebuffer方式を選んだprofile内だけで既定値にし、別profileから自動選択しない |

「最適」は曖昧なOS/GPU runtime heuristicで判定せず、profileごとの承認済み優先backendとして固定する。異なる候補を選ぶ利用者設定のAPI面（Runtime/Windowのどちらで指定するか）、互換性判定、診断結果はDD案で具体化して文書全体レビューを受けてから実装する。

### framebuffer限定profileの追加候補

framebuffer限定profileには、SceneSnapshotを既存のGLES rendererから分離したsoftware Rendererが検証可能なPixelBufferを生成し、別のLinux output sinkが対象deviceへ書き込む構成を候補とする。これは既存profileを置き換える汎用rendererではない。テスト用memory sinkはhardware outputの代替passにしない。

実出力sinkは次の候補を設計レビューで選ぶ。ここではまだ採用を確定しない。

| 候補 | 長所 | 制約・確認事項 |
|---|---|---|
| Linux fbdev (`/dev/fbN`) | 用語としてframebuffer出力に直接対応し、software pixel bufferの行をデバイスへ転送する境界を取りやすい | deviceの有無、bits-per-pixel、channel layout、line length、可視/virtual height、mapping、更新反映方法をprofileごとに確認する。存在しない環境はruntime対象にしない |
| DRM/KMS dumb buffer | 現行Linux direct-display経路と同じKMS device familyを利用できる | modeset/DRM master/output選択を引き続き所有する。現行GBM/EGL/GLES surfaceとの代替・併存・復元を設計する必要があり、fbdevと同じ簡易sinkとは扱えない |
| memory-backed PixelBufferのみ | CIでsoftware rasterとformat/strideを決定的に検証できる | 出力装置へ表示しないため、framebuffer runtime supportにはならない |

fbdevを選ぶ場合の推奨初期境界は、現在のdisplay modeを変更せず、要求pixel geometry/formatを満たさないdeviceを明示failureにすること。mode settingや暗黙fallbackは追加しない。最終契約は選択方式の承認後にDDへ記録する。

## 4. 実装unit案

| unit案 | 責務 | 主な試験 | 要求根拠 |
|---|---|---|---|
| `UNIT-PLATFORM-005` | RendererがSceneSnapshotを所有pixel storageへラスタライズする。background、Clip、FillRect、既存text serviceのGlyphRun maskを定義した順で合成し、未対応operationを成功扱いしない | `UT-LINUX-FB-RASTER-001`: pixel format、row stride/padding、bounds/clip、scale、clear/background、重なり順、alpha、glyph mask、buffer不足とunsupported operationのfailureをmemory sinkで確認 | RQ-09-067、RQ-09-072、RQ-09-075、RQ-11-017、RQ-11-022 |
| `UNIT-PLATFORM-006` | 承認されたLinux output sinkを開き、geometry/formatを検査し、PixelBufferの各行をdevice layoutへ転送し、close/failureをworker lifecycleへ接続する。Core/公開C ABIは変更しない方針 | `UT-LINUX-FB-SINK-001`: fake deviceでopen/validation/row copy/short buffer/write failure/cleanupを確認。実device可視描画は独立profile smoke | RQ-11-001、002、017、020〜024 |

二unitともLinux Platform crateにまとめ、必要なnative dependency・target cfg・manifest mapping・unit-test inventoryを同じ実装revisionへ登録する。unit IDは実装管理用であり、設計項目IDや項目別承認には使わない。

## 5. 実施手順

1. **default/override policy:** Platformごとのsurface/renderer規定値と明示選択範囲をDD案にする。既存Windows/Linux adapterを維持し、未登録候補の自動ロードや暗黙fallbackを加えない。
2. **framebuffer方式選択:** fbdev、DRM/KMS dumb bufferの候補をframebuffer限定profileで比較し、PixelBufferのformat・色表現・alpha・stride・orientation・flush・mode policy・lifecycle・failure contractを別profile用DD案にする。Debian VMの既存GLES smokeを新sinkの根拠にしない。
3. **unit-test設計:** `UNIT-PLATFORM-005/006`の入力、観測pixel/byte列、失敗・後始末、対象RQ、対応test IDと実行commandをUI unit-test designとmanifestに登録する。
4. **software raster/sink:** 承認されたframebuffer限定profileだけにPixelBuffer rendererとdevice sinkを実装する。fake device test、Linux native dependency/build、ABI open-failure smoke、target cross-buildを行い、未対応deviceでは明示failureにする。
5. **profile確認:** 対象profileの候補SHA、kernel/device、backend identity、可視出力、close/cleanupを記録する。CIのmemory sink testやcross-buildを可視表示passにしない。ARM boardが未用意ならbuild/backend evidenceまでで止めhardware runtimeをdeferredとする。
6. **support反映:** `tools/units/platform-linux.yaml`、unit inventory、CI条件、release test/support tableへprofileごとのdefaultと証拠境界を反映する。0.0.1 release gateで未確認profileを正式runtime supportとして表示しない。

## 6. 完了条件と未完了境界

- memory-backed renderer testsで承認済みPixelBuffer contractと全対応DrawOp/failureを検証し、同じrevisionのformat/build/lint gateがpassする。
- 選択されたnative sinkがfake deviceでlayout transferとfailure cleanupを検証し、Linux build/ABI smokeがpassする。
- cross-buildはtargetごとに`build-only`として記録する。ARM board実機を確認していない結果をruntime/hardware passとしない。
- 可視出力はprofile・候補SHA・実行環境を付けて別記する。Debian VMでは現行DRM/KMS経路のbasic visible presentとclose/reopenを確認済みだが、Debian profileの配布consumer・一般ユーザー権限・input等のrelease evidenceは別途必要。今回のsmokeは新規fbdev/DRM dumb-buffer sinkを検証していない。
- unit test、CI build、fake sink testの合格だけでは、他Linux distribution/deviceまたはarchitectureの正式supportを宣言しない。

## 7. 未決事項と停止条件

1. Linux framebuffer限定profileのsinkをfbdev (`/dev/fbN`) とDRM/KMS dumb bufferのどちらにするか。既存のGLES profileの既定を変更する選択ではない。
2. framebuffer限定backendの対象となる追加Linux profile/機器と必要なdevice access/権限。
3. Runtime/Windowのどちらに明示backend指定を置くか、利用可能候補のcompatibility reportと失敗診断をどう公開するか。既定選択はprofileで固定する。
4. 初期PixelBufferのformat、alpha合成、orientation、stride、presentation/flush、既存window modeとの共存・復元契約。
5. 実機表示可能な環境がない場合、0.0.1でframebuffer限定profileを実験的/build-onlyとして残すか、実装を後続versionへ送るか。

この計画案のレビューが済むまで実装・依存追加・target support表示の変更は行わない。要求の新しい外部保証が必要になった場合はRQ改訂を先に行う。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-09 | 計画0.1 | 利用者レビュー待ち | Platform範囲変更の承認後に作成。実装方式・unit分割・実装開始は未承認 |
| 2026-11-09 | 計画0.2 | 利用者承認 | 完了済みadapterを維持し、profileごとの既定backendと明示選択を先に設計する方針を承認。DD案・framebuffer方式・unit分割・実装開始は別レビュー |
