# Windows WM_CLOSE と Core close-request の接続計画

- 計画版: 0.1
- 状態: 利用者承認済み（2026-11-01）。unit実装・試験状態は別管理
- 対象version: `0.0.1`
- 対象: Win32 `WM_CLOSE` から既存Core Window close-request protocolへの通知とpeer lifecycle接続
- 実装branch案: `impl-rust/platform-adapter-windows-close-request`
- 親branch案: `impl-rust/core-window-text-service`
- 関連計画: [Windows Platform Adapter](windows-platform-adapter.md)、[Windows TextService・Scene接続](windows-text-scene-producer-integration.md)

## 目的

現在、Windows peerは`WM_CLOSE`を抑止している。0.0.1の初期UI要求に従い、OSからのWindow close intentを既存Core close-request protocolへ届け、Accept/Decline/Deferの結果を対象peerの表示状態へ反映する。

新しい公開APIやRuntime shutdown動作は追加しない。Core Window/Tree/Propertyを保持したままpeerを閉じ、明示的な再openを可能にする既存契約を保つ。

## 現状と根拠

- `crates/platform-windows/src/win32.rs`は`WM_CLOSE`を受信すると、Core連携が未実装であるためpeerを閉じずに抑止している。
- Core Runtime/C ABIにはWindow close-requestのsubscribe、callback中のAccept/Decline/Defer、deferred requestのresolve/releaseが実装済み。
- [RQ-09-071](../11_requirements/RQ-09_ui-scope.md#rq-09-071)は、Window終了後もCore状態を保持して再表示できること、close判断の既定Accept、未解決request破棄時のDecline、Runtime shutdownを個別veto対象にしないことを定める。
- [UIクラス設計](../13_detail_design/ui/controls.md)はCloseRequestとWindowの状態遷移を定める。
- [Platform抽象化設計](../13_detail_design/platform/abstraction.md)はOS close intentをCore requestとして伝える。一方、close accepted後のpeer/Core状態関係には確認事項が残る。
- [Platform内部層設計](../13_detail_design/platform/internal-layers.md)はOS close intentを後続unitとしている。Windows workerのHWND owner threadとCore UI threadをまたぐ通知・resolve call flowも、このunitで明示する必要がある。

## 実装前の設計確認

Source変更前に、承認済みRQとDD間の未確定点を照合し、必要なら関係する設計文書全体の改訂案をレビューへ出す。

1. Core close requestのcallbackが実行されるthreadと、Win32 workerのmessage処理threadを確認する。
2. `WM_CLOSE`処理中にCore callbackの終了を待つ必要があるか、またはUI threadへの非同期dispatch後にpeerをどの状態で保持するかを確定する。workerとUI threadの相互待ちを作らない。
3. Accept時は対象generationのpeerを閉じ、Core Window/Tree/Propertyを保持する。DeclineまたはDefer中はpeerを開いたままにする。Defer後は対応するrequestのresolve結果だけを反映し、stale generationは無視する。
4. 購読者がない場合、callbackが判断せず戻る場合、deferred requestの破棄、closeとRuntime shutdownの競合を既存Core契約へ対応付ける。
5. 上記のいずれかが承認済みDDで一意に決まらない場合、実装都合だけで補わず、UI/Platform DDの差分を先に提示する。

## 実装単位と試験案

| Unit | 責務 | 完了条件 |
| --- | --- | --- |
| `UNIT-PLATFORM-WINDOWS-006`（候補） | Windows peerの`WM_CLOSE`をCore close-requestへ接続し、decisionとpeer generationに沿ってpeer lifecycleを更新する。Core状態とRuntime全体は暗黙終了しない | `UT-PLATFORM-WINDOWS-CLOSE-001`（候補）でAccept/Decline/Defer、deferred resolve/release、subscriberなし、stale generation、close/shutdown race、worker停止/dispatch失敗をfake境界で確認。Windows target build、既存workspace gates、Rust minimum、文書検査が同一revisionで成功 |

実装着手前にunit/test識別子・path衝突を確認し、`.docs/32_unit_test/ui.md`、`.docs/32_unit_test/implementation-test-inventory.md`、`tools/units.yaml`へ対応RQ/DD/testを登録する。fake試験は実OS GUIでの可視close操作の代替ではない。

### 必須観測

- Acceptは要求元のWindow peerだけを閉じ、Core Window/Tree/Propertyを保ち、同じWindowの再openを可能にする。
- Declineはpeerとsurfaceを維持し、Runtimeや他Windowへcloseを波及させない。
- Deferはpeerを維持し、後続resolveのAccept/Declineに従う。request破棄時は既存規則に従いWindowを閉じない。
- 古いrequest/generationや閉じたpeerの遅延dispatchは、新generationのpeerを閉じない。
- `WM_CLOSE`の処理、Core callback dispatch、deferred resolve、Runtime shutdownでdeadlock・二重close・資源二重解放を起こさない。
- Core callback/user codeをWin32 worker上で直接実行しない。既存のUI-thread規約に従う。

## Linux profileとの境界

現行Linux profileはDRM/KMSの直接表示であり、Windows `WM_CLOSE`相当のwindow-manager close messageを持たない。したがってこのunitにはLinux Platform実装を含めない。Linux DRM/KMSの明示`Window.close()`、Runtime shutdown、実機表示確認は既存契約・別gateとして管理する。Wayland/X11等のwindow-manager adapterを追加するときは、そのnative close intentを同じCore protocolへ接続する個別unitを設ける。

## 検証・Git記録

- 対応focused test、workspace check/test、Windows target build、Clippy/format、Rust 1.85 minimum、文書link検査を実行し、同一SHAのWindows/Linux CIを記録する。
- 対応unitの試験合格後、日本語commit・pushする。
- Windows GUIでのtitlebar close、Decline/Deferを含むprofile smokeはCIから切り離し、0.0.1 release qualificationへ記録する。Windows GUI表示確認とLinux DRM実機確認を未実施のまま合格扱いしない。
- 関連する0.0.1 TODOが完了した段階で、まとめてmerge PRを作成する。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-01 | 計画0.1 | 利用者承認 | 利用者「OK」により計画を承認。計画のcommit/pushと、既存要求・設計との確認後に実装へ進む。試験合格・GUI確認は別管理 |
