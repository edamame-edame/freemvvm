# Windows WM_CLOSE と Core close-request の接続計画

- 計画版: 0.2
- 状態: 計画0.1/0.2利用者承認済み（2026-11-01）。unit実装・試験状態は別管理
- 対象version: `0.0.1`
- 対象: Win32 `WM_CLOSE` から既存Core Window close-request protocolへの通知とpeer lifecycle接続
- Core APIs branch: `impl-rust/core-window-text-service`
- Windows Platform子branch案: `impl-rust/platform-adapter-windows-close-request`
- 関連計画: [Windows Platform Adapter](windows-platform-adapter.md)、[Windows TextService・Scene接続](windows-text-scene-producer-integration.md)

## 目的

現在、Windows peerは`WM_CLOSE`を抑止している。0.0.1の初期UI要求に従い、OSからのWindow close intentを既存Core close-request protocolへ届け、Accept/Decline/Deferの結果を対象peerの表示状態へ反映する。

新しい公開APIやRuntime shutdown動作は追加しない。Core Window/Tree/Propertyを保持したままpeerを閉じ、明示的な再openを可能にする既存契約を保つ。

## 現状と根拠

- `crates/platform-windows/src/win32.rs`は`WM_CLOSE`を受信すると、Core連携が未実装であるためpeerを閉じずに抑止している。
- Core Runtime/C ABIにはWindow close-requestのsubscribe、callback中のAccept/Decline/Defer、deferred requestのresolve/releaseが実装済み。
- 実装時のCore identity/generation bridgeの注意点は[Window close-request memory](../01_memory/abi-window-close-request.md)へ記録する。
- [RQ-09-071](../11_requirements/RQ-09_ui-scope.md#rq-09-071)は、Window終了後もCore状態を保持して再表示できること、close判断の既定Accept、未解決request破棄時のDecline、Runtime shutdownを個別veto対象にしないことを定める。
- [UIクラス設計](../13_detail_design/ui/controls.md)はCloseRequestとWindowの状態遷移を定める。
- [Platform抽象化設計](../13_detail_design/platform/abstraction.md)はOS close intentをCore requestとして伝える。一方、close accepted後のpeer/Core状態関係には確認事項が残る。
- [Platform内部層設計](../13_detail_design/platform/internal-layers.md)はOS close intentを後続unitとしている。Windows workerのHWND owner threadとCore UI threadをまたぐ通知・resolve call flowも、このunitで明示する必要がある。

## 実装前の設計確認

Source変更前に、承認済みRQとDD間の未確定点を照合する。特にPlatform abstractionの「close accepted後のpeer/Core状態は確認が必要」という記述をRQ-09-071とUI class DDの合意内容へ揃え、対象設計文書全体を改訂・レビューする。設計レビューの完了前にsourceへ進まない。

1. Core close requestのcallbackが実行されるthreadと、Win32 workerのmessage処理threadを確認する。
2. `WM_CLOSE`処理中にCore callbackの終了を待つ必要があるか、またはUI threadへの非同期dispatch後にpeerをどの状態で保持するかを確定する。workerとUI threadの相互待ちを作らない。
3. Accept時は対象generationのpeerを閉じ、Core Window/Tree/Propertyを保持する。DeclineまたはDefer中はpeerを開いたままにする。Defer後は対応するrequestのresolve結果だけを反映し、stale generationは無視する。
4. 購読者がない場合、callbackが判断せず戻る場合、deferred requestの破棄、closeとRuntime shutdownの競合を既存Core契約へ対応付ける。
5. 上記のいずれかが承認済みDDで一意に決まらない場合、実装都合だけで補わず、UI/Platform DDの改訂案を先に提示する。

## 実装単位と試験案

| Unit | 責務 | 完了条件 |
| --- | --- | --- |
| `UNIT-CORE-090`（候補） | private Platform SPIへclose-intent sinkを加え、RuntimeがWindow identity/generationをdispatcherへpostして既存close-request lifecycleをUI thread上で実行する。公開C ABIは変更しない | `UT-CORE-PLATFORM-CLOSE-001`（候補）でpost/dispatch、既定Accept、購読callbackのDecline/Defer、stale generation、dispatcher unavailable/shutdownをfake peerで確認。Core/workspace gates、Rust minimum、文書検査が同一revisionで成功 |
| `UNIT-PLATFORM-WINDOWS-006`（候補） | Windows peerの`WM_CLOSE`をCore close-intent sinkへ渡し、decision結果としてCoreから呼ばれるpeer close commandをWin32 workerで安全に実行する | `UT-PLATFORM-WINDOWS-CLOSE-001`（候補）でclose intent forwarding、Accept/Decline/Defer、stale peer、worker停止、close/shutdown raceをfake境界で確認。Windows target build、workspace gates、Rust minimum、文書検査が同一revisionで成功 |

設計reconciliationのレビュー後、unit/test識別子・path衝突を確認し、`.docs/32_unit_test/ui.md`、`.docs/32_unit_test/implementation-test-inventory.md`、`tools/units.yaml`へ両unitの対応RQ/DD/testを登録する。Core unitを親branchで実装・検証・commit/pushしてから、その更新済みCore branchを親にWindows Platform childを作成する。fake試験は実OS GUIでの可視close操作の代替ではない。

### 必須観測

- Acceptは要求元のWindow peerだけを閉じ、Core Window/Tree/Propertyを保ち、同じWindowの再openを可能にする。
- Declineはpeerとsurfaceを維持し、Runtimeや他Windowへcloseを波及させない。
- Deferはpeerを維持し、後続resolveのAccept/Declineに従う。request破棄時は既存規則に従いWindowを閉じない。
- 古いrequest/generationや閉じたpeerの遅延dispatchは、新generationのpeerを閉じない。
- `WM_CLOSE`の処理、Core callback dispatch、deferred resolve、Runtime shutdownでdeadlock・二重close・資源二重解放を起こさない。
- Core callback/user codeをWin32 worker上で直接実行しない。既存のUI-thread規約に従う。
- Native `WM_CLOSE` handlerはCore/UI threadへのdispatchを待って同期blockしない。handlerがmessage loopへ戻った後にCoreがacceptしてもpeer close commandを処理できる。

## Linux profileとの境界

現行Linux profileはDRM/KMSの直接表示であり、Windows `WM_CLOSE`相当のwindow-manager close messageを持たない。したがってこのunitにはLinux Platform実装を含めない。Linux DRM/KMSの明示`Window.close()`、Runtime shutdown、実機表示確認は既存契約・別gateとして管理する。Wayland/X11等のwindow-manager adapterを追加するときは、そのnative close intentを同じCore protocolへ接続する個別unitを設ける。

## 検証・Git記録

- 対応focused test、workspace check/test、Windows target build、Clippy/format、Rust 1.85 minimum、文書link検査を実行し、同一SHAのWindows/Linux CIを記録する。
- Core unitとWindows Platform unitの各試験合格後、それぞれの層のbranchで日本語commit・pushする。
- Windows GUIでのtitlebar close、Decline/Deferを含むprofile smokeはCIから切り離し、0.0.1 release qualificationへ記録する。Windows GUI表示確認とLinux DRM実機確認を未実施のまま合格扱いしない。
- 関連する0.0.1 TODOが完了した段階で、まとめてmerge PRを作成する。

## 実装・検証状況

- `UNIT-CORE-090`をCore private Platform SPIへ実装し、`UT-CORE-PLATFORM-CLOSE-001`を追加した。off-UI-thread通知はRuntime dispatcherへpostされ、existing close subscriptionがUI threadで処理される。
- local pass: focused close-intent test 1件、Core 185 tests、workspace all-target tests、workspace Clippy、Rust 1.85 workspace check、format check、document links 7133件。
- 同revision CI pass: `3de3ec5340c0b8a055a8ba8047ca14a5da3dccb8`、[Actions run 41307305993](https://github.com/edamame-edame/freemvvm/actions/runs/41307305993)（Ubuntu 24.04 / Windows 2022 success）。Windows Platform `UNIT-PLATFORM-WINDOWS-006`へ進む。
- `UNIT-PLATFORM-WINDOWS-006`を実装し、Win32 `WM_CLOSE`をgeneration付きCore sinkへ通知する。通知中にHWNDを破棄せず、post失敗時もdefault closeを抑止し、Coreからの後続peer close commandだけがworker上でnative resourceを閉じる。
- local pass: Windows crate 20 tests（2 manual profile smoke ignored）、workspace all-target tests、Clippy、Rust 1.85 workspace check、format check、document links 7133件。対象revisionのCIをpush後に確認する。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-01 | 計画0.1 | 利用者承認 | 利用者「OK」により計画を承認。計画のcommit/pushと、既存要求・設計との確認後に実装へ進む。試験合格・GUI確認は別管理 |
| 2026-11-01 | 計画0.2 | 利用者承認 | 利用者「ok」によりCore APIs unit先行、設計reconciliation、続くWindows Platform unitの分割を承認。実装・試験・GUI確認は別管理 |
