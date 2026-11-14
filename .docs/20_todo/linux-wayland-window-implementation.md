# 0.0.1 Linux Wayland Window実装計画

- 計画版: 0.2
- 状態: 計画版0.2承認済み・push済み。unit実装中。UNIT-CORE-095完了、ABI以降は未着手
- 対象version: `0.0.1`
- 対象branch: `change/platform-support-scope`
- 親計画: [Linux desktop backend scope](linux-desktop-backend-scope-proposal.md) 0.6
- 関連保留計画: [Linux DRM/KMS runtime resize](linux-drm-runtime-resize.md)
- 要求: [RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)、[RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[RQ-12 build/distribution](../11_requirements/RQ-12_build-distribution.md)
- 設計: [UI controls DD 1.8](../13_detail_design/ui/controls.md)、[Platform internal layers DD 3.0](../13_detail_design/platform/internal-layers.md)、[ABI DD 1.8](../13_detail_design/runtime-objects/abi.md)、[build/toolchain DD 1.3](../13_detail_design/build/toolchain-packages.md)、[Rust wrapper DD 1.9](../13_detail_design/wrappers/rust.md)、[C++ wrapper DD 1.6](../13_detail_design/wrappers/cpp.md)、[C# wrapper DD 1.7](../13_detail_design/wrappers/csharp.md)、[Python wrapper DD 2.3](../13_detail_design/wrappers/python.md)、[API/build承認記録](linux-wayland-window-api-proposal.md)
- 試験: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[UI test design](../32_unit_test/ui.md)、[release test matrix](../30_release_test/requirement-test-matrix.md)
- 実装上の引継ぎ知識: [Linux Wayland feasibility](../01_memory/platform-linux-wayland-feasibility.md)

## 目的

承認済みRQ-09-076のWindow初期client size設定、省略時800×600 DIP、Platformが確定する実寸への追従をCore・C ABI・Rust/C++/C#/Python wrapper・Linux Wayland peerへ実装する。同時に、既存DRM/KMS console経路と分けたWayland desktop経路を実装し、0.0.1対象Linux profileを個別に確認する。

初期サイズはWindowごとの希望値として保持し、open/reopen時にpeerへ渡す。OS/compositorが返す実client areaをCoreへ返し、その実寸でLayout/Sceneを更新する。指定値を満たす保証は追加しない。有限かつ正でない値はWindow作成前に拒否し、部分状態を公開しない。既存create APIは互換維持し、未指定時もCoreの800×600 DIP defaultを使う。

## 承認済み設計と実装境界

1. RQ-09-076、RQ-11/12、UI DD 1.8、Platform BD 1.5、Platform DD 3.0、ABI DD 1.8、build DD 1.3、およびRust/C++/C#/Python wrapper DD 1.9/1.6/1.7/2.3は、2026-11-14に文書全体承認済み。本計画はこれらの契約を実装する。
2. Window初期サイズAPIはABI feature 10 `core.ui.window.initial_client_size.v1`と`ui_abi_1_window_create_with_initial_client_size`を使う。既存create symbolとwrapper生成APIを維持し、サイズ指定時だけfeature 10を使う。C ABIが有限かつ正の寸法を検証し、失敗時にpartial Windowを公開しない。Rust wrapperは対応native artifactと直接linkする。C++/C#/Pythonは追加symbolをoptional resolutionし、旧symbolだけのartifactでも既存constructorを維持する。
3. Wayland backend identityは`linux-wayland-egl`とする。DRM/KMSとWaylandはCargo feature・artifact profileを分離し、同一artifact内での併用、実行中hot-swap、暗黙fallback/plugin探索は行わない。Wayland desktop peerは現在のWayland sessionに接続し、TTY切替、root権限、display manager停止、DRM CRTC直接modesetを動作条件にしない。
4. build profileは`debian-13-vm-wayland-x64`、`debian-12-vm-wayland-x64`、`ubuntu-24.04-wayland-x64`、`ubuntu-26.04-wayland-x64`とする。Rust minimum 1.85を維持し、lock-resolved dependency graphとOS提供runtime library条件を検証する。
5. `GlesRenderer`再利用、Wayland worker ownership、xdg-shell configure/ack、EGL Wayland surface、frame completion、close/reopenは承認済みPlatform DD 3.0に従う。新しい利用者保証またはDDとの不整合が必要になった場合は実装を止め、差分を設計レビューへ戻す。

## ブランチと実施順

既存のGit規約に従い、作業は `change/platform-support-scope` からCore APIsブランチを切って始める。Core APIsブランチにはCore Rust、C ABI、Linux peer/backendとそのbuild設定を含める。Core APIsが完成・pushされてから、Rust、C++、C#、Pythonの各wrapperを更新済みCore APIsから兄弟ブランチとして順番に切り、各言語のテスト後に統合する。wrapper間で親子依存を作らない。

wrapper中にCore/C ABI変更が必要と判明したらwrapper作業を止め、change branchへ戻ってCore APIs更新と他の全wrapperへの影響確認を行う。そのCore APIsから対象wrapper兄弟ブランチを切り直す。全計画のunit完了後にchange branch向けのmerge PRをまとめて出す。対応unit test合格ごとに日本語commit/pushする。

## 実装unitと同revision test

各IDはunit inventoryとの重複確認後に登録済み。source変更と対応test design/catalogは同じrevisionに含め、unit test pass後に該当unitを完了へ更新する。

| Unit | 責務と変更範囲 | 対応test案 |
| --- | --- | --- |
| `UNIT-CORE-095` | `Window`にimmutable初期client sizeを保持し、未指定時800×600 DIP、有限/正値検証、close/reopen保持、peer open情報への受渡し、実client areaでLayout/Sceneを開始する | `UT-WINDOW-INITIAL-SIZE-001`: default/explicit/NaN/±infinity/zero/negative、作成失敗時partial stateなし、reopenで同じ希望値、peer実寸優先をfake peerで検証 |
| `UNIT-ABI-024` | 既存window create symbolsを保つ追加ABI API、ヘッダ、status/error/out handle、feature/query/API表、C/C++ compile・runtime probe | `UT-ABI-WINDOW-OPTIONS-001`: default/明示backend+size、backend省略/明示のpresence、無効寸法、null/lengthとout handle初期化、既存symbol互換を検証 |
| `UNIT-PLATFORM-005` | Linux Wayland build/profile factory、接続・xdg-shell lifecycle、configure/ack、EGL/GLES surface、resize/redraw、frame completion、close/reopen、failure cleanup。既存DRM/KMS console artifactを分離 | `UT-PLATFORM-WAYLAND-001`: protocol/worker/surface seamをfake化しconfigureの正値/0軸、serial ack、resize burst、stale generation、present/failure、close/shutdownを決定的に検証 |
| `UNIT-WRAP-RS-WINDOW-SIZE-001` | Rust wrapperでdefaultと明示backend/sizeの生成・型安全な寸法指定・エラー変換 | `UT-WRAP-RS-WINDOW-SIZE-001` とRust consumer smoke |
| `UNIT-WRAP-CPP-WINDOW-SIZE-001` | C++ wrapperでdefaultと明示backend/sizeの生成・idiomatic値型・例外/Result規約 | `UT-WRAP-CPP-WINDOW-SIZE-001` とC++ consumer/package smoke |
| `UNIT-WRAP-CS-WINDOW-SIZE-001` | C# wrapperでdefaultと明示backend/sizeの生成・値型・例外/Result規約 | `UT-WRAP-CS-WINDOW-SIZE-001` とC# consumer smoke |
| `UNIT-WRAP-PY-WINDOW-SIZE-001` | Python wrapperでdefaultとkeyword引数等のidiomatic API、native error mapping、built wheelの利用 | `UT-WRAP-PY-WINDOW-SIZE-001` とwheel/import/consumer smoke |

### 進捗（2026-11-14）

- `UNIT-CORE-095` complete: `WindowInitialClientSize`の800×600 DIP defaultと有限・正値検証、Window生成時保持、peer size-aware open受渡し、実client areaでのLayout、close/reopen値保持を実装。Core focused testとCore全206 testsがpass。
- `UNIT-ABI-024`を次の実装unitとする。ABI API、feature 10、C/C++ header/runtime probesは未実装。
- Platform peerの既存`open(generation)`は維持し、default `open_with_initial_client_size`へ追加委譲する。旧peerの互換動作を保ち、Wayland unitでサイズ適用を具体化する。

実装台帳 (`tools/units.yaml` と責務別 `tools/units/*.yaml`) にsource/config/test-design path、依存unit、RQ・承認設計、test command、release caseを追加する。ID重複または現在の最大番号更新があれば仮IDを繰り下げる。test IDは既存命名規則と文書全体整合を見て登録する。

## テスト文書・CI・配布設定

- `.docs/32_unit_test/ui.md`にWindow初期size Core/ABI/Platform test oracleを追加し、wrapper test design文書にも各言語のdefault/explicit/invalid-input parityを記す。
- RQ-09-076に対応するlogical assertion/caseをrequirements-test matrix/manifestへ追加する。Wayland runtime caseはbuild成功やfake testと別にし、RQ-11-015とRQ-12-010をOS/session/artifact単位で追跡する。
- 既存のLinux release caseをDebian 13/12、Ubuntu 24.04/26.04へ整合する形で改訂し、各profileのexact OS/kernel、compositor/session、hypervisor/virtual GPU/driver、backend、artifact SHA/digest、結果を記録する。TC-TARGET-LNX-VM-01をDebian 13合格でDebian 12の代替証拠にしない。
- `libwayland-dev`等のbuild package、runtime library依存、static/shared link要件をDebian/Ubuntu向けに記録する。Cargo lock-resolved依存全体をRust 1.85で検証し、wayland-protocols等にMSRV超過があれば互換版を選ぶ。CI/開発環境手順はbuild profileと同じ依存選択を使う。
- build feature/profileで未使用backend依存を除外できること、WaylandとDRM/KMSの選択がartifact時に決まりruntimeで交換されないことをCargo feature graphとpackage outputで確認する。Python wheelは既存方針に合わせ、native libraryの発見・同梱境界をconsumer testで確認する。

## 段階ゲートと順序

1. **設計gate（完了）**: API/build方式を含む7つの正本DD全体が2026-11-14に承認済み。詳細は[API/build承認記録](linux-wayland-window-api-proposal.md)。
2. **unit/test準備gate（完了）**: Core・ABIの試験設計、RQ traceability、unit path・command・依存を登録し、利用者レビュー承認済み。Platform・wrapperの詳細試験設計は各unit着手前に同様に確認する。
3. **Core/ABI実装gate**: `UNIT-CORE-095`と`UNIT-ABI-024`を順に実装し、focused test、header probes、既存API regression、Rust 1.85 checkを同revisionで通す。これを完了してからLinuxと各wrapperの基点とする。
4. **Wayland backend gate**: Debian 13 graphical Wayland sessionで`libwayland-dev`等の環境を整え、承認済みprofile `debian-13-vm-wayland-x64`、feature `presentation-backend-linux-wayland-egl-gles`を使う。native Wayland client + wayland-egl +既存GLES rendererを実装し、EGL/GLES依存・thread affinity・frame completionとevent loopを検証する。DRM console profileを壊さない。
5. **wrapper sibling gates**: Rust → C++ → C# → Pythonの順に更新済みCore APIs branchから兄弟branchを切る。各言語のdefault/explicit creation、invalid input、既存API互換、公開package consumer testを通す。各wrapper間の意味とdefault値を一致させる。
6. **profile release gates**: `debian-13-vm-wayland-x64` → `debian-12-vm-wayland-x64` → `ubuntu-24.04-wayland-x64` → `ubuntu-26.04-wayland-x64`の順に、Window visible present、初期希望寸法とcompositor実寸、compositor resize後のLayout/再描画、close/reopen、stderr/logを個別に記録する。各profileのpassは独立した証拠に限定する。
7. **統合完了gate**: 全unitと関連CI、profile evidence、文書・unit catalog・release matrixがそろった時だけ本計画を完了扱いにし、まとめてmerge PRを提示する。マージ後は本流以外の統合済み一時branchをGit規約に従い整理する。

## 完了条件

- RQ-09-076の省略default、明示size、invalid寸法拒否、OS/compositor実寸追従がCore/C ABI/Rust/C++/C#/Pythonで同じ意味となり、既存ABI create/APIを壊さない。
- Wayland desktop artifactは通常のWayland session内でWindow表示・frame present・compositor resize後の再Layout/再描画・close/reopenができる。TTY切替/root/device直開きを動作条件にしない。
- xdg-shell configure serial、surface buffer size、Core logical client area/DeviceScale、frame completionの対応が承認済みPlatform DDに一致する。stale/failure frameをPresented扱いしない。
- Wayland desktopとDRM/KMS consoleのartifact/build profileが区別され、backend切替/fallbackがない。最低Rust 1.85と対象Linux packagingが通る。
- Windows/Linux CI、各unit test、release test文書とrequirements traceabilityが同一revisionで検証される。runtime supportは個別OS/session/VM profileでvisible evidenceを得たものだけを記録する。

## 対象外

- Linux DRM/KMS runtime resize。Wayland desktop resizeで代替扱いせず、[保留計画](linux-drm-runtime-resize.md)に残す。
- X11、Wayland compositorごとの未検証機能、multi-output、HiDPI/monitor間scale migration、runtime backend hot-swap。
- ARM hardware runtime。framebuffer限定profileとcross-buildは既存範囲のまま別statusとする。
- 未承認のWindow sizing guarantee、ABI破壊、backend自動fallback、Wayland対応だけを根拠としたIME/Accessibility support。

## 承認境界

- 0.1計画は`ddaeb94`で承認・push済み。今回は、承認済みAPI/ABI・wrapper/build設計を反映した計画0.2をレビュー案として提示する。
- 0.2計画は利用者レビュー承認後、`9f20dfb`で日本語commit/push済み。source実装は承認済みunit/test準備を済ませた範囲から進め、各実装unitを対応testと同revisionで検証する。
- 対応unitのtest pass後は日本語commit/pushし、全unit完了後にchange branchへのmerge PRをまとめて提示する。
- 実装はunitと同revisionのtest合格後に日本語commit/pushし、全unit完了後にchange branch向けのmerge PRをまとめて出す。

## レビュー履歴

| 日付 | 計画 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-14 | 0.1 | 利用者レビュー待ち | 承認済みRQ-09-076/UI BD/UI DD/Platform BD/Platform DDをCore、C ABI、Linux Wayland、4 wrapper、profile別release testへ展開。API/ABI詳細はimplementation前の別レビューgate。実装・依存追加は未着手 |
| 2026-11-14 | 0.1 | 承認 | commit `ddaeb94`で承認計画をpush。API/ABI詳細設計承認後に実装を開始するgateを含む |
| 2026-11-14 | 0.2 | 利用者レビュー待ち | ABI/Platform/build/four wrapperの承認済み改訂を反映し、API設計gate完了、unit/test準備gate、採用済みfeature/profile名を更新。source実装は未着手 |
| 2026-11-14 | 0.2 | 承認・push済み | 利用者の「ok」により計画全体を承認。commit `9f20dfb`。Core branch `impl-rust/core-linux-wayland-window`から実装開始 |
