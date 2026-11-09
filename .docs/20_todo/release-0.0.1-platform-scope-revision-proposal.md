# 0.0.1 Platform対象範囲の要求・設計・試験改訂案

- 提案版: 0.1
- 状態: scope改訂案と関連するRQ/BD/DD/release-test文書全体を利用者承認済み（2026-11-09）。Debian VM feasibility確認、framebuffer実装計画、実装・release判定は未完了
- 対象version: `0.0.1`
- 対象となる承認済み要求: [RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[RQ-12 Build/Distribution](../11_requirements/RQ-12_build-distribution.md)
- 根拠設計: [Platform BD](../12_design/platform/abstraction.md)、[Platform DD](../13_detail_design/platform/internal-layers.md)
- 承認境界: 本提案と参照する文書版全体の内容は承認済み。実装・VM確認・release合格・tag・公開を意味しない

## 目的

実機の準備が困難なARM等のarchitecture固有profileを0.0.1のruntime/release gateから切り離す。VM上で同じ成果物の描画経路まで確認できるx64 Linux profileは、確認できた具体的なguest/hypervisor profileに限り0.0.1の対象に含める。その他Linuxへのframebuffer限定経路は、対象機器上で未確認の限り正式runtime support済みと表示しない。

## 提案する0.0.1 profile区分

| profile | 0.0.1での扱い案 | 合格に必要な証拠 | 保証しない範囲 |
|---|---|---|---|
| Windows 11 x64 | 正式runtime profile | 候補SHAの配布package consumer、UI表示・入力・close/reopen、Windows GUI上のframe presentation | 未実施のWindows ARM64/他Windows版 |
| Ubuntu 24.04 x64 | 正式runtime profile候補 | 対象sessionを固定し、配布consumerと実UI表示・入力を確認。既存Linux backendを使う場合はbackend/surface/presentまで観測 | 未検証Ubuntu版、他desktop session、物理GPU固有性能 |
| Debian 12 x64 VM | VM profileとして正式runtime対象に加える候補 | hypervisor/product/version、guest image、virtual GPU/driver、device/surface open、visible present、配布consumerを記録。同一候補SHAで再現 | 物理GPU、実機display mode、ARM board、他hypervisorへの一般化 |
| Linux ARMv7/AArch64 | framebuffer描画限定の実験的経路・cross-build/fake testを0.0.1候補に含め、本番機器runtime gateは後続へ分離 | cross-build/ABI linkとfake pixel-buffer testを別evidenceとして記録 | board runtime, input/IME/accessibility, hardware display/device recoveryのpass表示 |
| その他Linux framebuffer profile | 0.0.1の限定backend候補 | profileとframebuffer contractを固定し、software/fake pixel bufferでrendererとfailure境界を検証 | 実機未確認targetをruntime-supportedとする表示。WM/windowing、input、IME、Accessibilityは別途承認・検証がない限り保証しない |

Ubuntu 24.04は既存CIの`ubuntu-24.04`と揃える候補であり、要求への確定反映前にprofileとしてレビューする。Debian VMはOS起動だけではprofile passとしない。現在のLinux Adapterでdevice/surface取得から可視presentまで到達しないVM構成は、build/test環境またはcontainer provisionalに留める。VMで確認したprofileの結果を物理GPU・他hypervisor・ARM boardへ外挿しない。

## 要求変更案

### RQ-11 Platform

次版候補は0.6。以下のID本文を一括改訂し、承認履歴へ利用者の文書単位承認を記録する。

| 要求 | 改訂案 |
|---|---|
| RQ-11-001 / 002 | ARMv7/AArch64向けにframebuffer描画限定の実験的経路とcross-buildを0.0.1候補に含める。本番board runtime gateは後続versionへ送り、実機未確認中は正式hardware supportと表示しない |
| RQ-11-003 | Linux x64を開発/CI targetに保つ。正式runtime profileはUbuntu 24.04 x64と、条件を満たすDebian 12 x64 VM profileのようにdistribution/session/hypervisorまで識別する |
| RQ-11-014〜018 | backendの検証順を0.0.1 profileに合わせて再定義。Windows native backend、Ubuntu/Debian x64の選択backend、その他Linuxの限定framebuffer出力を区別し、未採用のWayland/X11/DRM/GLES候補は将来候補として保持する |
| RQ-11-021 | build、VM/OS runtime、physical hardware profileを別statusにする。VM profile passから物理GPU/board supportを主張しない |
| RQ-11-022 | 正式profileごとのbackendと限定framebuffer profileのfailure/recovery範囲を明示する。未承認のfallbackやsilent backend switchingを追加しない |
| RQ-11-023 | Ubuntu/Debian VMとframebuffer限定profileごとにfont/IME/Accessibilityの利用条件を識別する。framebuffer出力だけではIME/Accessibility対応済みとしない |
| RQ-11-024 | 正式runtime profile候補をWindows 11 x64、Ubuntu 24.04 x64、検証に合格したDebian 12 x64 VMとする。ARM cross-build/framebuffer fake testをbuild/backend evidenceとして記録し、ARM hardware runtimeは後続profile gateとして分離する |

### RQ-12 Build/Distribution

次版候補は0.6。target別artifact生成の一般要求を保ちつつ、artifactの存在をruntime supportの証拠と混同しないよう以下を改訂候補とする。

- RQ-12-002: artifact targetと正式support profileの一覧・statusを別に表せるよう明確化する。
- RQ-12-004/006/008: Linux x64 package targetのdistro/profile条件を、Windows x64と同様に識別可能にする。Debian/Ubuntu別のpackage構成はimplementation reviewで確定し、現行のtarget tripleだけから互換性を推定しない。
- RQ-12-009/010: Windows x64/Ubuntu 24.04 x64 build gate、Debian VM runtime gate、ARM cross-build/framebuffer fake test、ARM physical runtime gateを別々に記録する。ARM physical gateは0.0.1完了条件から外す。
- RQ-12-018: 公表support tableで`build-only`、`VM-runtime-verified`、`hardware-runtime-verified`を区別する。

## 基本設計・詳細設計の改訂案

- Platform BD [abstraction.md](../12_design/platform/abstraction.md) 1.1から次の版案へ改訂する。要求対応表、対象profile、backend責務と開発順をRQ-11の新しい区分へ合わせる。OS/session/hypervisor依存の可視Windowとdirect-display profileの区別を記す。
- Platform DD [internal-layers.md](../13_detail_design/platform/internal-layers.md)の次版案で、現行DRM/KMS+GBM/EGL/GLES経路がUbuntu/Debian VMのどの仮想GPUで利用可能かを実証後に記録する。VMで不成立なら、そのLinux backendを0.0.1正式profileへ流用せず、対象profileに合う代替方式を別案としてレビューする。
- 「framebuffer」は設計承認前に用語を固定する。少なくともsoftware raster output buffer、Linux fbdev (`/dev/fb*`)、DRM/KMS dumb bufferを区別し、pixel format/stride/orientation/flush/present/failure/ownershipを定義する。名前にframebufferを含む現在のKMS framebuffer objectだけをframebuffer limited backendとみなさない。
- Framebuffer限定profileで非対応とするWindow management、keyboard/pointer input、IME、Accessibility、device loss recoveryは、要求本文へ除外条件として置き、設計・READMEでも一致させる。

## Release test / CI / 実機証拠の変更案

- `system-scenarios.md` REL-PROFILE-001の0.0.1適用profileをWindows 11 x64、Ubuntu 24.04 x64、VM成立条件を満たすDebian 12 x64にする。TC-TARGET-BOARD-01は削除せず、ARM profile再開時の将来caseとして残す。
- Debian VM smokeのassertionにOS image、hypervisor/version、virtual GPU driver、native artifact digest、device/surface open、実表示、resize/redraw、close/reopenを含める。VM sessionの起動だけ、Docker headless GLES、pixel-readbackだけではvisible present passとしない。
- ARM CI cross-build/linkは継続可能だが、board runtime/hardware testは0.0.1 release gateから切り離し`blocked/deferred`として記録する。
- Framebuffer限定rendererのunit testはmemory-backed fake bufferでformat/stride/clip/clear/failureを検証する。これは実機表示の代替passではない。
- Requirement-test matrixのTC-TARGET-LNX-01/TC-TARGET-BOARD-01とRQ-11-024の対応を再点検し、要求を移したcaseやprofileを孤立させない。既存test IDは意味を変えず、範囲変更時はcase本文・承認履歴を更新する。
- README/support matrixはprofileごとに検証状態、限定事項、未検証architectureを明記する。

## 承認済みの範囲と残る実行gate

1. Windows 11 x64、Ubuntu 24.04 x64を0.0.1の正式runtime profileとする。
2. Debian 12 x64はVM/hypervisor/virtual GPUごとの可視presentと配布consumer確認に合格したprofileに限り対象とする。VM smokeが未実施のため、現時点でDebian profile合格とは表示しない。
3. その他Linuxはframebuffer描画出力に範囲を限定する。window management、input、IME、Accessibilityの保証を含めず、framebufferの方式・契約・実装unitは別のDD/実装計画で具体化する。
4. ARMv7/AArch64のcross-buildとfake-buffer確認はbuild/backend evidenceとして分離し、physical board runtime gateは後続versionへ送る。

この範囲で要求、BD、DD、release plan、system scenario、requirements-test文書の版全体は承認済み。実装開始前には、framebuffer方式とunit/test範囲を独立計画として提示・承認し、上記profile境界と整合させる。

## 実施順

1. Debian 12 VMで既存Linux artifactの可視present可否を確認し、成功・失敗・未実施をcandidate SHAとともに記録する。Docker結果とは混ぜない。
2. framebuffer方式、実装unit、fake-buffer testと実機表示の証拠境界を独立の実装計画にまとめて提示する。
3. 承認済みprofile matrixに従い実装・CI・VM smokeを進め、evidenceをprofileとcandidate SHAへ対応付ける。
4. 実装とrelease gate完了後にsupport tableを確定し、release・tag・公開判断を別途行う。

## 現在の進捗

- 利用者はscope改訂案を承認し、2026-11-09に関連するRQ-11/12、要求仕様索引、Platform BD/DD、0.0.1 plan、system scenario、requirements-test assertion/matrixの各文書版全体も承認した。
- 承認済み範囲は本書のprofile境界と一致する。各文書の過去版・承認記録は保持し、実装・試験結果は別管理とする。
- Debian 12 VMのvisible-present feasibility smokeは未実行。framebuffer方式と実装unitの具体計画、CI/evidence、formal supportの実績判定、release判定は未完了。
- Debian 12 VMのvisible-present feasibility smokeは未実行。profileを追加するには実行証拠が必要。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-09 | 提案0.1と関連する文書版全体 | 利用者承認 | 「ok」によりscope改訂案とRQ/BD/DD/release-test文書全体の内容を承認。実装開始・試験合格・release判定は含まない |
