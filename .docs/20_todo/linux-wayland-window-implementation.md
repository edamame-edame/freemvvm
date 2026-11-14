# 0.0.1 Linux Wayland Window実装計画案

- 計画版: 0.1
- 状態: レビュー待ち・未承認・未コミット。実装未着手
- 対象version: `0.0.1`
- 対象branch: `change/platform-support-scope`
- 親計画: [Linux desktop backend scope](linux-desktop-backend-scope-proposal.md) 0.6
- 関連保留計画: [Linux DRM/KMS runtime resize](linux-drm-runtime-resize.md)
- 要求: [RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)、[RQ-11 Platform](../11_requirements/RQ-11_platform.md)、[RQ-12 build/distribution](../11_requirements/RQ-12_build-distribution.md)
- 設計: [UI controls DD](../13_detail_design/ui/controls.md)、[Platform internal layers DD](../13_detail_design/platform/internal-layers.md)、[ABI DD](../13_detail_design/runtime-objects/abi.md)、[Rust wrapper DD](../13_detail_design/wrappers/rust.md)、[C++ wrapper DD](../13_detail_design/wrappers/cpp.md)、[C# wrapper DD](../13_detail_design/wrappers/csharp.md)、[Python wrapper DD](../13_detail_design/wrappers/python.md)
- 試験: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[UI test design](../32_unit_test/ui.md)、[release test matrix](../30_release_test/requirement-test-matrix.md)
- 実装上の引継ぎ知識: [Linux Wayland feasibility](../01_memory/platform-linux-wayland-feasibility.md)

## 目的

承認済みRQ-09-076のWindow初期client size設定、省略時800×600 DIP、Platformが確定する実寸への追従をCore・C ABI・Rust/C++/C#/Python wrapper・Linux Wayland peerへ実装する。同時に、既存DRM/KMS console経路と分けたWayland desktop経路を実装し、0.0.1対象Linux profileを個別に確認する。

初期サイズはWindowごとの希望値として保持し、open/reopen時にpeerへ渡す。OS/compositorが返す実client areaをCoreへ返し、その実寸でLayout/Sceneを更新する。指定値を満たす保証は追加しない。有限かつ正でない値はWindow作成前に拒否し、部分状態を公開しない。既存create APIは互換維持し、未指定時もCoreの800×600 DIP defaultを使う。

## 設計・実装境界

1. UI DD 1.8、Platform BD 1.5、Platform DD 2.9、RQ-09-076、RQ-11/12は承認済み。本計画はこれらの既決事項を実装する。別途承認されていない公開ABI signature、wrapper各言語の引数形式、Cargo feature名、crate/version、native link方式はまだ確定事項として扱わない。
2. 最初の実装前gateでABI DDへ後方互換な追加symbol案を記載する。提案する公開面は、既存default create/backend createを維持しつつ、明示backendと初期サイズを同時指定できる追加操作を設ける形とする。引数は固定幅・明示的なpresence規則とし、無効値/不正な組合せ/out handle規則を定義する。実際のsignatureと各wrapperのidiomatic APIは該当設計文書全体をレビュー・承認してから実装へ進む。
3. Linuxのbackendはbuild/profile選択で固定し、Window実行中のbackend交換や暗黙fallbackを入れない。Wayland desktop artifactとDRM/KMS console artifactの依存・既定値・配布条件を区別する。通常desktopのWayland peerは現在のWayland sessionに接続し、TTY切替、root権限、display manager停止、DRM CRTC直接modesetを要求しない。
4. `GlesRenderer`再利用、Wayland worker所有、xdg-shell configure/ack、EGL Wayland surface、frame completion、close/reopenの詳細は承認済みPlatform DD 2.9に従う。新しい利用者保証またはDDとの不整合が必要になった場合は、実装を止めて差分を設計レビューへ戻す。

## ブランチと実施順

既存のGit規約に従い、作業は `change/platform-support-scope` からCore APIsブランチを切って始める。Core APIsブランチにはCore Rust、C ABI、Linux peer/backendとそのbuild設定を含める。Core APIsが完成・pushされてから、Rust、C++、C#、Pythonの各wrapperを更新済みCore APIsから兄弟ブランチとして順番に切り、各言語のテスト後に統合する。wrapper間で親子依存を作らない。

wrapper中にCore/C ABI変更が必要と判明したらwrapper作業を止め、change branchへ戻ってCore APIs更新と他の全wrapperへの影響確認を行う。そのCore APIsから対象wrapper兄弟ブランチを切り直す。全計画のunit完了後にchange branch向けのmerge PRをまとめて出す。対応unit test合格ごとに日本語commit/pushする。

## 実装unitと同revision test

以下のIDはレビュー用の仮割当で、台帳登録時に既存台帳と再照合する。source変更と対応test design/catalogは同じrevisionに含める。

| Unit | 責務と変更範囲 | 対応test案 |
| --- | --- | --- |
| `UNIT-CORE-095` | `Window`にimmutable初期client sizeを保持し、未指定時800×600 DIP、有限/正値検証、close/reopen保持、peer open情報への受渡し、実client areaでLayout/Sceneを開始する | `UT-WINDOW-INITIAL-SIZE-001`: default/explicit/NaN/±infinity/zero/negative、作成失敗時partial stateなし、reopenで同じ希望値、peer実寸優先をfake peerで検証 |
| `UNIT-ABI-024` | 既存window create symbolsを保つ追加ABI API、ヘッダ、status/error/out handle、feature/query/API表、C/C++ compile・runtime probe | `UT-ABI-WINDOW-OPTIONS-001`: default/明示backend+size、backend省略/明示のpresence、無効寸法、null/lengthとout handle初期化、既存symbol互換を検証 |
| `UNIT-PLATFORM-005` | Linux Wayland build/profile factory、接続・xdg-shell lifecycle、configure/ack、EGL/GLES surface、resize/redraw、frame completion、close/reopen、failure cleanup。既存DRM/KMS console artifactを分離 | `UT-PLATFORM-WAYLAND-001`: protocol/worker/surface seamをfake化しconfigureの正値/0軸、serial ack、resize burst、stale generation、present/failure、close/shutdownを決定的に検証 |
| `UNIT-WRAP-RS-WINDOW-SIZE-001` | Rust wrapperでdefaultと明示backend/sizeの生成・型安全な寸法指定・エラー変換 | `UT-WRAP-RS-WINDOW-SIZE-001` とRust consumer smoke |
| `UNIT-WRAP-CPP-WINDOW-SIZE-001` | C++ wrapperでdefaultと明示backend/sizeの生成・idiomatic値型・例外/Result規約 | `UT-WRAP-CPP-WINDOW-SIZE-001` とC++ consumer/package smoke |
| `UNIT-WRAP-CS-WINDOW-SIZE-001` | C# wrapperでdefaultと明示backend/sizeの生成・値型・例外/Result規約 | `UT-WRAP-CS-WINDOW-SIZE-001` とC# consumer smoke |
| `UNIT-WRAP-PY-WINDOW-SIZE-001` | Python wrapperでdefaultとkeyword引数等のidiomatic API、native error mapping、built wheelの利用 | `UT-WRAP-PY-WINDOW-SIZE-001` とwheel/import/consumer smoke |

実装台帳 (`tools/units.yaml` と責務別 `tools/units/*.yaml`) にsource/config/test-design path、依存unit、RQ・承認設計、test command、release caseを追加する。ID重複または現在の最大番号更新があれば仮IDを繰り下げる。test IDは既存命名規則と文書全体整合を見て登録する。

## テスト文書・CI・配布設定

- `.docs/32_unit_test/ui.md`にWindow初期size Core/ABI/Platform test oracleを追加し、wrapper test design文書にも各言語のdefault/explicit/invalid-input parityを記す。
- RQ-09-076に対応するlogical assertion/caseをrequirements-test matrix/manifestへ追加する。Wayland runtime caseはbuild成功やfake testと別にし、RQ-11-015とRQ-12-010をOS/session/artifact単位で追跡する。
- 既存のLinux release caseをDebian 13/12、Ubuntu 24.04/26.04へ整合する形で改訂し、各profileのexact OS/kernel、compositor/session、hypervisor/virtual GPU/driver、backend、artifact SHA/digest、結果を記録する。TC-TARGET-LNX-VM-01をDebian 13合格でDebian 12の代替証拠にしない。
- `libwayland-dev`等のbuild package、runtime library依存、static/shared link要件をDebian/Ubuntu向けに記録する。Cargo lock-resolved依存全体をRust 1.85で検証し、wayland-protocols等にMSRV超過があれば互換版を選ぶ。CI/開発環境手順はbuild profileと同じ依存選択を使う。
- build feature/profileで未使用backend依存を除外できること、WaylandとDRM/KMSの選択がartifact時に決まりruntimeで交換されないことをCargo feature graphとpackage outputで確認する。Python wheelは既存方針に合わせ、native libraryの発見・同梱境界をconsumer testで確認する。

## 段階ゲートと順序

1. **API/ABI設計gate**: 既存API・ABI・4 wrapper DD・build/profile DDの差分を調査し、追加symbol、引数表現、error/marshalling、feature selection、library link/配布を設計案にする。文書全体レビュー・承認前にsource、test、dependencyを変更しない。
2. **Core/ABI実装gate**: `UNIT-CORE-095`と`UNIT-ABI-024`を順に実装し、focused test、header probes、既存API regression、Rust 1.85 checkを同revisionで通す。これを完了してからLinuxと各wrapperの基点とする。
3. **Wayland backend gate**: Debian 13 graphical Wayland sessionで`libwayland-dev`等の環境を整え、native Wayland client + wayland-egl +既存GLES rendererを実装する。EGL/GLES依存・API thread affinity・frame completionとevent loopを検証し、DRM console profileを壊さない。
4. **wrapper sibling gates**: Rust → C++ → C# → Pythonの順にCore APIs更新済みbranchから兄弟branchを切る。各言語のdefault/explicit creation、invalid input、既存API互換、公開package consumer testを通す。各wrapper間の意味とdefault値を一致させる。
5. **profile release gates**: Debian 13 Wayland visible test、Debian 12、Ubuntu 24.04、Ubuntu 26.04の順に各実行環境で同じ検証手順を行う。windowのvisible present、初期サイズ希望と実寸、compositor resize後のLayout/再描画、close/reopen、stderr/logを記録する。各profileのpassは個別に限定し、未実施は未確認のままにする。
6. **統合完了gate**: 全unitと関連CI、profile evidence、文書・unit catalog・release matrixがそろった時だけ本計画を完了扱いにし、まとめてmerge PRを提示する。マージ後は本流以外の統合済み一時branchをGit規約に従い整理する。

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

- 本書はWayland desktopとWindow初期サイズ設定を実装するための計画案であり、利用者レビュー待ち。
- 本計画の承認後も、段階ゲート1のABI/wrapper/build設計案をレビュー・承認してから実装へ進む。
- 計画承認までは依存追加、source/test変更、unit catalog登録、commit/pushを行わない。本TODOはレビュー用に未コミットで提示する。
- 実装はunitと同revisionのtest合格後に日本語commit/pushし、全unit完了後にchange branch向けのmerge PRをまとめて出す。

## レビュー履歴

| 日付 | 計画 | 状態 | 備考 |
|---|---|---|---|
| 2026-11-14 | 0.1 | 利用者レビュー待ち | 承認済みRQ-09-076/UI BD/UI DD/Platform BD/Platform DDをCore、C ABI、Linux Wayland、4 wrapper、profile別release testへ展開。API/ABI詳細はimplementation前の別レビューgate。実装・依存追加は未着手 |
