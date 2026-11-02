# freemvvm: Input/Focus実装計画とbranch依存

- カテゴリ: 実装計画 / UI Input・Focus / 0.0.1
- 対象: `.docs/20_todo/input-focus-event-path-implementation.md`

## 学んだこと

- 共通Input/Focus設計承認後も、wrapper固有のcallback署名・borrowed event view・subscription解放・例外隔離、およびWindows native key/IME mappingが未確定なら、wrapper実装unitをreadyにしない。
- 現在のCore APIs branchは`origin/version/0.0.1`より先行し、release branchだけにあるcommitがあった。層のbranch順を維持するには、実装child作成前にrelease差分を既存Core branchへ明示同期する計画が必要。
- Rust/C ABI Core、4言語wrapper、Windows Adapterは別unitとして同一revisionの試験を行い、各言語wrapperはCore APIs branchから兄弟branchとして作る。
- `.docs/32_unit_test/implementation-test-inventory.md`の`planned`はCIで実行されない。実在するpath・command・承認済み設計根拠を確認後に`ready`へ進める。
- `IsEnabled`の変更は直接Property setterだけでなくBinding伝播からも届く。Focusの無効化は個別setterに埋めず、両経路が通るProperty変更後の共通処理でLogical subtreeを再評価する。
- Focus eligibilityは対象Control自身とWindowまでの全Logical ancestorの`IsEnabled`を確認する。祖先disable時はFocusと復帰candidateを消し、reenable後も自動復元しない。
- Pointer targetは公開LayoutSnapshotと同revisionのControlTreeSnapshotから決める。Visual treeの後置siblingsをtopmostとし、Containerの`clips_children`とWindow client DIP境界をscene生成と同じく適用する。dirty/stale snapshotは入力targetに使わない。
- Coreの最小確認は`cargo test -p freemvvm-core focus_requires_active_window_and_restores_only_valid_attached_targets --locked`。この回は追加した祖先disable/reenable assertionsを含めてpassした。Core Input全体unitの完了とは区別する。
- Input dispatchは開始時にtargetからWindowまでのLogical routeをsnapshotし、各handlerの前後でWindow generation・target・現在nodeの有効性を再検査する。callback中にdetach/disable/closeされた後のancestor配送は止める。
- Input subscriptionは既存Subscription lifecycleへ接続する。自己cancel中はin-flight callbackの終了までhandlerを保持し、callback panicでもdelivery countを必ず戻してからpanicをdispatcherへ伝播させる。
- Input handler試験では、target→ancestor→Window順、handled停止、callback中detach、自己cancel、panic後のuserdata一回解放を確認する。該当回のCore suiteは191件pass、workspace Clippy・format・diff checkもpassしたが、C ABIを含む`UNIT-CORE-091`は未完了。
- Window向けtarget解決はKeyDown/KeyUp/TextCommitをactive WindowのFocusへ、PrimaryPointerDown/Upを同generationの公開LayoutSnapshot/ControlTreeSnapshotによるDIP hit-testへ振り分ける。target選択の回帰試験ではTextBoxとButtonを別配置し、キーとpointerで別targetに届くことを検査する。

参照: [Input/Focus実装開始計画](../20_todo/input-focus-event-path-implementation.md)、[Input/Focus次工程計画](../20_todo/input-focus-event-path.md)、[Git規則](../02_git/rule.md)、[実装unit台帳](../32_unit_test/implementation-test-inventory.md)。

## 設計案作成時の追加確認

- USB HID Usage Page 0x07は物理key identityなので、TextCommitの文字値とは別に扱う。Windows scan codeからはOS標準Scan 1/HID表で変換し、VK値を公開ABIへ流さない。
- Windows IMEのWM_CHAR/WM_IME_CHAR経路はOS既定処理が関与するため、設計案ではWM_CHARを単一commit源とする仮説を明示し、実機IME smokeで重複/欠落を検査する。結果が異なる場合はnative mapping設計を改訂する。
- 共通pointer eventの初期buttonはprimaryだけ。Windows native mapping案はWM_LBUTTON系のmouse-compatible pathに限定し、native touch/pen対応は別unitへ分ける。
