# Core input ingress / Windows Platform input bridge

- 計画版: 0.2（承認済み）
- 状態: 利用者承認済み（2026-11-08）。phase 1～4完了、Core ingressの同revision CI確認待ち
- 対象version: `0.0.1`
- 上位計画: [Input / Focus event path実装計画](input-focus-event-path-implementation.md)
- 対象unit: Core input ingress `UNIT-CORE-093`（ready）、Windows adapter `UNIT-PLATFORM-WINDOWS-007`（planned）
- branch案: `version/0.0.1` → 案件 `change/windows-platform-input-bridge`（commit `20a43a1`）→ Core ingress `impl-rust/core-platform-input-ingress` → Windows adapter `impl-rust/platform-adapter-windows-input`
- 実装開始条件: この計画0.2と、参照するunit-test設計文書の改訂全文がそれぞれ承認済み

## この計画を追加する理由

4 wrapperのInput実装はそれぞれ独立branch上で実装され、Rust CI `41642705673`、C++ CI `41648453537`、C# CI `42030613702`、Python CI `42054917806` が成功している。

Windows native input adapterの実装開始前に、設計と実装の間にあるCore ingressの不足が見つかった。

- 承認済み[Platform internal layers](../13_detail_design/platform/internal-layers.md) 2.3は、`PlatformWindowInputSink`がWindow identity/generationに結び付き、normalized eventをCore UI dispatcherへpostする責務を定めている。2.4はWin32 messageからneutral Input valuesへのmappingを定めている。
- 現行Core `PlatformWindowPeer`にはinput sinkの接続口がなく、Windows `NativeWindowBackend::open`にもinput eventをCoreへ渡す接続口がない。
- Coreの`RuntimeCore::dispatch_window_input_event`はcrate-privateで、Platform crateから呼び出せない。したがってWindows crateだけでは承認済みbridgeを実現できない。
- [UI単体試験](../32_unit_test/ui.md)は1.8案を全文レビュー待ちとしている。Windows mapping caseはあるが、Core ingress queue/wake/lifecycleの試験設計はまだない。上位計画のフェーズ2完了記録とこの状態に差があるため、承認状態を先に照合する。

## 目的

Windows message workerからnative input値をCore UI dispatcherへ非同期に届ける内部SPIと、Win32 event mappingを依存順に実装・検証する。CoreがFocus・routing・handled・Button/TextBox意味論のsource of truthであること、WndProc/workerからuser callbackを呼ばないことを維持する。

新しい利用者向け保証、公開C ABI、OS固有値のCore流入は追加しない。承認済み要求・DDで定まらないqueue/wake/lifecycleの条件は、実装都合だけで決めず設計文書でレビューする。

## 根拠

- 要求: [RQ-06 thread/dispatch](../11_requirements/RQ-06_threading-dispatch.md)、[RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)。特にworkerからUIへ渡す経路、serial UI dispatch、入力/Focus/IMEの既存保証。
- Core/Windows設計: [Platform internal layers](../13_detail_design/platform/internal-layers.md) 2.3・2.4全文承認済み、[Platform abstraction](../13_detail_design/platform/abstraction.md)のPlatform input delivery boundary、[UI Input/Focus](../13_detail_design/ui/input-focus.md)。
- 現行実装境界: `crates/core-runtime/src/ui_window.rs`、`crates/core-runtime/src/runtime.rs`、`crates/platform-windows/src/peer.rs`、`crates/platform-windows/src/win32.rs`。
- 試験: [UI単体試験](../32_unit_test/ui.md) 1.8（Core ingress / Windows mapping全文承認済み）、[implementation unit inventory](../32_unit_test/implementation-test-inventory.md)、[unit-test strategy](../32_unit_test/unit-test-strategy.md)。
- Git: [branch rule](../02_git/rule.md)。実装先がCoreからWindows Platformへ切り替わる時点でbranchを切り替え、wrapper branchは親にしない。

## 実施順

1. [x] 現行`version/0.0.1`のHEAD、上位計画、unit inventory、Core/C ABI/4 wrapperの完了記録を照合する。Core `UNIT-CORE-091`、C ABI `UNIT-ABI-022`、4 wrapperの各Input unitはcomplete、統合SHA `82923807a819d6151ee10aeb601dc3da55e3d03a`のWindows run `42118740862`・Ubuntu run `42118808951`もsuccess。`UNIT-CORE-092`はreadyのまま、Windows `UNIT-PLATFORM-WINDOWS-007`はplannedのまま維持。
2. [x] Internal input sink契約を実装粒度まで具体化し、`internal-layers.md` 2.5全文の承認を得る。sink接続、owned event、generation検査、Dispatcher FIFO/UI-thread delivery、shutdown/stale drop、post失敗を反映し、既存要求から新しい利用者保証を追加しない。
3. [x] `.docs/32_unit_test/ui.md` 1.8全文の承認を得る。Core ingressとWindows mappingのcaseを分離し、前提・操作・oracle・cleanup・要求caseを記載。Core unit `UNIT-CORE-093` / `UT-CORE-PLATFORM-INPUT-001` を登録。
4. [x] 案件branch `change/windows-platform-input-bridge`（commit `20a43a1`）を親にCore bridge branch `impl-rust/core-platform-input-ingress`を作成。Core側に内部Platform input sinkとUI dispatcherへの入口を実装し、FIFO/owned text/thread/generation/shutdown/post-failureをfake試験する。`UNIT-CORE-093`をreadyとして登録。
5. [ ] Core unitのfocused/Core全試験、Clippy、Rust 1.85 check、workspace testsはローカル同revisionで成功。日本語commit/push後に必要CIの同revision成功を確認する。Core ingress変更はCore branchで完結させ、既存wrapperやWindows crateの製品変更を混ぜない。CI成功後、検証済みCore branchを案件branchへ統合する。
6. [ ] Windows adapter branch `impl-rust/platform-adapter-windows-input`をCore bridge branchから作成する。fakeable message mapperとworker sink接続を実装し、次を確認する。
   - key/system-keyのscan code + extended bitからHID keyboard usage、modifier、unknown-key扱い。system keyはneutral eventをpostしつつnative default procedureへ委譲。
   - `TranslateMessage`後のUnicode `WM_CHAR`、surrogate pairのscalar/UTF-8変換、dead/system char除外、同一TextCommitの重複防止。OS IMEの実機挙動はfake試験合格に含めない。
   - mouseとprimary touch/pen down/upのclient座標→DIP一回変換、順序、pointer promotion重複防止。非primary contact、PT_MOUSE重複経路は無視する。
   - activation/focus、event FIFO、stale generation、closed/shutdown peer、post失敗、WndProcからuser callbackを呼ばない境界。
7. [ ] Windows Rust target build、adapter focused/all-target tests、Rust minimum、workspace CI、document linksを同一revisionで検証し、日本語commit/pushする。実機keyboard/IME/touch/pen確認はCIとは分け、0.0.1 release qualificationへ記録する。adapter unit完了後、検証済みadapter branchを案件branchへ統合する。
8. [ ] Core ingress/Windows adapterと全Input wrapperのunit台帳・CI証跡を再照合し、Input/Focus関連TODO batch完了後に案件branchから`version/0.0.1`向けの集約merge PRを作成する。マージ後は案件/Core/Platformの作業branchとremote/local追跡refを削除する。

## branch依存

```text
version/0.0.1 (InputFocus Core/C ABI/4 wrappersを統合済み)
└─ change/windows-platform-input-bridge
   └─ impl-rust/core-platform-input-ingress
      └─ impl-rust/platform-adapter-windows-input
```

各PRをマージした後は、その実装branchとremote/local追跡refを削除する。`version/...` branchは維持する。

## 完了条件

- Native input sourceからCore InputRouterまで、copy・FIFO・generation・UI thread dispatch・shutdown failureを通して接続される。
- Core ingressとWindows mappingが個別unitとして同一revisionで検証され、Rust minimumおよびWindows/Linux CIが成功する。
- Win32固有message値はPlatform crate境界に留まり、public C ABIを変更しない。
- 実機IME/keyboard/pointer確認とCI/fake試験の結果を混同せず、残る実機確認をrelease qualificationへ明記する。

## 未決としてレビューする点

- `PlatformWindowInputSink`の具体Rust型・設定methodと、Core dispatcherとの接続方法。
- Queue容量/確保失敗時の戻り値と診断方法、UI wake要求が失敗した場合の再試行/破棄境界。
- Per-window generationのstale判定をCore ingressで行うか、peerとCoreの両方で行うか。
- IME確定文字は承認済み2.4の`WM_CHAR`一経路で受ける。Windows profile smokeでduplicate/欠落が観測された場合、実装都合でsourceを増やさず設計変更として扱う。

計画0.2、`internal-layers.md` 2.5、`ui.md` 1.8は全文承認済み。`UNIT-CORE-093`はローカル実装・試験済みで、同revision CI確認中とする。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-07 | 計画0.1 | superseded | PR #5後に削除済みの`impl-rust/core-input-focus`をCore ingressの親にしていた |
| 2026-11-07 | 計画0.2 | 利用者承認 | PR #6 merge後の最新`version/0.0.1`を起点に案件branch/Core/Platformの階層へ変更。unit完了状態は同revision CI証跡と再照合し、PR後に作業branch/追跡refを削除する運用を追加。利用者が「ok」と承認 |
| 2026-11-07 | phase 1 unit/CI照合 | 完了 | Input Core/C ABI/4 wrapperの既存complete状態とWindows/Ubuntu統合CI runを確認。`UNIT-CORE-092`はready、`UNIT-PLATFORM-WINDOWS-007`はplannedとして維持 |
| 2026-11-08 | phase 2/3 DD・test design | 完了 | 利用者が`internal-layers.md` 2.5と`ui.md` 1.8全文を承認。`UNIT-CORE-093` / `UT-CORE-PLATFORM-INPUT-001`を登録し実装へ進行 |
| 2026-11-08 | phase 4 Core input ingress | local pass / CI待ち | `impl-rust/core-platform-input-ingress`でCore sink、owned event、dispatcher FIFO/generation処理、focused/Core/workspace tests、Clippy、Rust 1.85 checkを実施 |
