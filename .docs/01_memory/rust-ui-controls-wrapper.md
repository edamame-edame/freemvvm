# Rust wrapper: UI Control implementation and branch ordering

- 適用範囲: Rust safe wrapperで初期UI Control/Tree/Layout/Focus/Window close requestを実装する計画と着手前調査。
- UI Control facadeは`impl-rust/ui-controls-wrapper`で実装済み。Core Input branchを基準にした別checkoutでは未統合に見える場合があるため、checkout上の有無だけで完了branchの有無を判断しない。
- UI Control / Window close-request safe APIはRust wrapper DD v1.7～1.8全文承認済み。Input event wrapperは後続の1.8文書範囲で管理する。
- UI Control wrapper branchの親はCore APIs branch `impl-rust/core-ui-controls-c-abi`（revision `dfb86cf`）。別言語wrapper branchを親にしない。Input wrapper branchへ取り込む場合も共通Core APIs基準を維持する。
- C++ UI Controls (`UNIT-WRAP-CPP-009`) はcommit `14b5d45` / CI run `40183464855` success。C# UI Controls/Window は`UNIT-WRAP-CS-CONTROLS-001`/`UNIT-WRAP-CS-WINDOW-001`が別branchで完了、CI run `40169552932` success。
- 承認済み Rust UI Control safe APIは`WidgetRef`をsealed traitにし、borrowed Widget tokenを公開せず、typed ownerを`Window`/`StackPanel`/`TextBox`/`Text`/`Button`/`Widget`に分ける。各ownerは既存`Owned<T>`/cleanup ticket/RuntimeAnchorを使う。
- Rust UI実装時はC ABI返却token用のcleanup ticketとerror snapshot用ticketの両方をABI呼出し前に確保する。ABI呼出し後に追加allocationして失敗tokenを取りこぼさない。
- Buttonの`command()`は独立owned handleを返すが、`AsyncCommand::close()`はhandle releaseだけでなく共有Core Commandをcloseする。getter ownerはdrop/releaseし、Core Command closeは一度だけ行う。
- Host event turnなしのC ABI consumerではBinding伝播が即時とは限らない。UI peer/host loopを前提にしないconsumerでは同期伝播をassertせず、別のfocused testでhost turnを使って検証する。
- `UNIT-WRAP-RS-UI-CTRL-001` commit `159cf70`はWindows/Linux CI run `40185381670` success。local `rust-wrapper-ui-controls-test`、`tools/dev.py check`、`tools/dev.py minimum`もpass。
- Window close-requestでは`CloseRequestView::defer()`がborrowed viewのprivate RuntimeAnchorでRuntime identity/UI thread/open状態を確認する（DD v1.8全文承認済み）。`'static` callbackにborrowed `UiContext`を捕捉させずdeferし、返却されたowned requestは別のUI操作でresolveする。
- `UNIT-WRAP-RS-WINDOW-001` commit `37d8a29` は`rust-wrapper-window-close-test`、`tools/dev.py check`、`minimum`とWindows/Linux CI run `40187756235`がsuccess。
