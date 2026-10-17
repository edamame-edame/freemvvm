# Rust Core / HandleEntry record

- 更新日: 2026-10-17
- スコープ: `crates/core-runtime/src/handle_registry/entry.rs`、公開参照recordの値動作
- 計画・結果: [HandleEntry実装slice](../20_todo/handle-entry-record.md)
- 設計根拠: [objects-values.md 1.1](../13_detail_design/runtime-objects/objects-values.md)
- 試験設計: [core-runtime.md 0.3](../32_unit_test/core-runtime.md)

`HandleEntry<K, R, O>` はkind、optional Runtime identity、一つのoptional owner referenceを保持し、identityの具体型とownerのthread-safe表現を選ばない。`matches_kind`/`matches_runtime`は記録を変更せず、`consume_reference`はownerを一度だけ呼出側へ移す。実registryはconsumeで得たownerの破棄をtable lock外に置く必要がある。MutexやRuntime UI状態をgeneric `O`のSend/Sync条件に押し込まない。

2件のunit testはkind/Runtime一致・不一致、Runtimeなし、照合後のlive維持、および同一対象を共有する二owner参照の独立consumeを検査する。Rust 1.90.0と1.85.0でCore全8件と開発16ゲートが成功。実registry、ValidationPin、実object owner型は別sliceである。
