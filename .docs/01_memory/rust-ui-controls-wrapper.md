# Rust wrapper: UI Control preflight and branch ordering

- 適用範囲: Rust safe wrapperで初期UI Control/Tree/Layout/Focus/Window close requestを実装する計画と着手前調査。
- `wrappers/rust/freemvvm/src`にはRuntime、Property、Subscription、Binding、Async Commandがあるが、Window/Widget/Container/StackPanel/Text/TextBox/Buttonの公開UI facadeは未実装。
- Rust wrapper DD v1.6は既承認だが、UI Control API、borrowed child/content、close-request callback scopeをまだ具体化していない。DD全文レビュー前にRust public UI APIを確定・実装しない。
- Core UI Control/C ABI branch `impl-rust/core-ui-controls-c-abi`のrevision `dfb86cf`はCI run `40164472118` success。Rust wrapper branchはこのCore APIs branchを親にし、C++/C# wrapper siblingを親にしない。
- C++ UI Controls (`UNIT-WRAP-CPP-009`) はcommit `14b5d45` / CI run `40183464855` success。C# UI Controls/Window は`UNIT-WRAP-CS-CONTROLS-001`/`UNIT-WRAP-CS-WINDOW-001`が別branchで完了、CI run `40169552932` success。
