# Rust Core / Opaque handle token value

- 更新日: 2026-10-17
- スコープ: `crates/core-runtime/src/handle_registry/handle.rs`、ABI整数と内部token型の境界
- 計画・結果: [opaque handle token value](../20_todo/opaque-handle-token-value.md)
- 上位根拠: [RQ-04-004](../11_requirements/RQ-04_abi-boundary.md#rq-04-004)、[共通C ABI DD 1.1](../13_detail_design/runtime-objects/abi.md)、[handle・値・error DD 1.1](../13_detail_design/runtime-objects/objects-values.md)

Core内部の `HandleToken` は `#[repr(transparent)]` な `NonZeroU64` newtypeとし、ABIの `uint64_t` 0をchecked conversionで拒否する。非zero値はそのまま往復し、内部値をpointerへ変換しない。Rust 1.90と1.85でreject-zero、1/MAX round trip、size/alignmentのテストに合格した。これは型境界だけであり、C header・export関数・live registry・owner参照はまだ実装していない。
