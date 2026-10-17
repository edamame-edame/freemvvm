# Rust Core / Handle token発行器

- 更新日: 2026-10-17
- スコープ: `crates/core-runtime/src/handle_registry/token.rs`、HandleRegistry実装の初期slice
- 計画・結果: [token issuer実装](../20_todo/handle-token-issuer.md)
- 試験設計: [core runtime unit tests](../32_unit_test/core-runtime.md)

承認済みDD `runtime-objects/objects-values.md` v1.1のtoken規則を、Core内部の `Mutex<Option<NonZeroU64>>` 発行器として具体化した。`None` は最大値発行後の枯渇状態で、token 0へ周回しない。同期mutexがpoisonした場合は新tokenを出さず失敗する。1つのstatic issuerをCore内の全Runtimeで共有し、ABIへ値を公開する入口はまだない。

TokenIssuerの局所テストは4件。逐次発行、最大値の一回限り発行と枯渇、並行発行の一意性、mutex poison時のfail-closedを検査する。Rust 1.90.0とminimum 1.85.0の両方で成功し、bootstrap checkも全gate成功。HandleRegistryの登録表・kind/Runtime照合・retain/release、製品ABI統合は後続sliceとする。
