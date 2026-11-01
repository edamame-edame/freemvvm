# カテゴリ: C ABI / Window close-request ownership

## 用途

WindowのOS close intent、C ABI Subscription、borrowed/deferred request handleを実装・レビューするときに参照する。

## 実装上の学び

- close subscriberはWindowごとに同時一つ。callerのSubscription handle解放だけでは登録を解除せず、明示cancelまたはWindow/Runtime cleanupで解除する。
- callback実行中は現在のborrowed request tokenをthread-localで公開し、`defer`が成功した場合だけ独立owned request tokenを発行する。
- callback中のAccept/Declineはcallback完了後にCoreへcommitする。Defer後のresolveはWindow identityとgenerationを照合し、古いpeerへcloseを適用しない。
- owned requestのretainは独立tokenを増やす。最後の未解決owner releaseはDeclineとなり、解決済み/stale requestのhandle releaseは通常のreleaseとして成功させる。
- 共通Subscription owner registryはPropertyとWindow callbackの両方を保持できる。Window subscriberのcallback/destroyはcancelまたはowner teardownで一度だけ実行する。
- `has_feature(8)`はControl APIとclose lifecycle一式が同revisionで揃った時だけtrueにする。
- Platform native close intentはpeerからprivate sinkでRuntime UI dispatcherへ値としてpostし、Core/UI stateやC callbackをworker上で直接操作しない。dispatcher turn前にはcloseせず、UI turn内で既存subscriptionを使う。
- Platform-origin close intentにはWindow `ObjectIdentity`とpeer generationを保持し、dispatcher実行時に現在generationを`begin_close_intent`前に照合する。古いHWND/peerから遅延したcloseが再open済みWindowを閉じないようにする。
- C ABIの`window_close_intent_subscribed`はpublic Window handle tokenを入口にする。native sink側はtokenを保持・再利用せず、Core内部のidentity-based bridgeから同じsubscription/callback lifecycleを呼び出す。
- Windows peerはCore close sinkをnative backendの`open`へ渡し、各HWND stateへopen時のgenerationとともに保持する。WndProcはsinkを同期waitせず、通知の成功/失敗にかかわらず`WM_CLOSE`のdefault destructionを抑止する。HWNDの破棄はCoreが後続peer close commandを出したworker turnで行う。

## 関連

- [C ABI Control実装計画](../20_todo/core-ui-controls-abi-implementation.md)
- [C ABI仕様](../13_detail_design/runtime-objects/abi.md)
- [UI close-request単体テスト設計](../32_unit_test/ui.md#ut-window-abi-001)
