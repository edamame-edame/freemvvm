# Rust wrapper: UI Control preflight and branch ordering

- 適用範囲: Rust safe wrapperで初期UI Control/Tree/Layout/Focus/Window close requestを実装する計画と着手前調査。
- `wrappers/rust/freemvvm/src`にはRuntime、Property、Subscription、Binding、Async Commandがあるが、Window/Widget/Container/StackPanel/Text/TextBox/Buttonの公開UI facadeは未実装。
- Rust wrapper DD v1.6は既承認だが、UI Control API、borrowed child/content、close-request callback scopeをまだ具体化していない。DD全文レビュー前にRust public UI APIを確定・実装しない。
- Core UI Control/C ABI branch `impl-rust/core-ui-controls-c-abi`のrevision `dfb86cf`はCI run `40164472118` success。Rust wrapper branchはこのCore APIs branchを親にし、C++/C# wrapper siblingを親にしない。
- C++ UI Controls (`UNIT-WRAP-CPP-009`) はcommit `14b5d45` / CI run `40183464855` success。C# UI Controls/Window は`UNIT-WRAP-CS-CONTROLS-001`/`UNIT-WRAP-CS-WINDOW-001`が別branchで完了、CI run `40169552932` success。
- 承認済み Rust UI Control safe APIは`WidgetRef`をsealed traitにし、borrowed Widget tokenを公開せず、typed ownerを`Window`/`StackPanel`/`TextBox`/`Text`/`Button`/`Widget`に分ける。各ownerは既存`Owned<T>`/cleanup ticket/RuntimeAnchorを使う。
- Rust UI実装時はC ABI返却token用のcleanup ticketとerror snapshot用ticketの両方をABI呼出し前に確保する。ABI呼出し後に追加allocationして失敗tokenを取りこぼさない。
- Buttonの`command()`は独立owned handleを返すが、`AsyncCommand::close()`はhandle releaseだけでなく共有Core Commandをcloseする。getter ownerはdrop/releaseし、Core Command closeは一度だけ行う。
- Host event turnなしのC ABI consumerではBinding伝播が即時とは限らない。UI peer/host loopを前提にしないconsumerでは同期伝播をassertせず、別のfocused testでhost turnを使って検証する。
- `UNIT-WRAP-RS-UI-CTRL-001` commit `159cf70`はWindows/Linux CI run `40185381670` success。local `rust-wrapper-ui-controls-test`、`tools/dev.py check`、`tools/dev.py minimum`もpass。
