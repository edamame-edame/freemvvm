# Rust wrapper / Async Command

- 状態: Rust Command wrapper plan、DD v1.6、test design v0.9は利用者承認済み。実装とlocal gates完了、Windows/Linux CI待ち。
- 前提: Core/C ABI Async Command unitは完了。Rust safe facadeにはまだCommand/worker operation APIがない。
- 設計注意: Rust wrapperにexecutor/thread pool/Future completionを追加しない。UI start callbackは短時間で戻り、workerはthread-safe cancellation query、`runtime_post`を介したUI-thread terminal completionを使う。
- 所有境界: UI-affine handlesは`!Send/!Sync`を維持する。workerに渡すoperation leaseだけをSend可能にし、retained command/runtime/cancel handles、Runtime anchor、cleanup ticketとpost destroyを同じ寿命契約で扱う。
- 計画: [Rust Async Command wrapper](../20_todo/rust-async-command-wrapper.md)。承認前にsource/manifestは変更しない。添付`file_lib.rs`はread-only snapshot。承認済み計画に基づくworkspace crate root exportは変更済み。
