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

## 関連

- [C ABI Control実装計画](../20_todo/core-ui-controls-abi-implementation.md)
- [C ABI仕様](../13_detail_design/runtime-objects/abi.md)
- [UI close-request単体テスト設計](../32_unit_test/ui.md#ut-window-abi-001)
