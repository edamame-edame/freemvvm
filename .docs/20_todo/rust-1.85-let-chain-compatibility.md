# Rust 1.85最小版のlet-chain互換性修正

- 更新日: 2026-10-22
- 状態: 完了。Rust 1.85対応修正とWindows/Linux CI確認済み（2026-10-22）
- 対象: CI `rust-minimum` gateがRust 1.85.0で失敗する既存let-chain構文

## 背景

CI run `39792223019` のWindows/Linux両jobで通常の`check`とC++ package integrationは成功したが、後段の`cargo +1.85.0 check --workspace --all-targets --locked`が`E0658`で失敗した。RQ-03-014でRust 1.85を最低版としている一方、次の3つのlet-chain条件（source上2ブロック）がこの版で未安定だった。

- `crates/core-runtime/src/lib.rs:436`: `call_destroy`とoptional destroy関数の複合条件
- `crates/core-runtime/src/runtime.rs:2912-2913`: property store lookupとdowncastの複合条件

## 該当コードと失敗理由

### `crates/core-runtime/src/lib.rs`

現在のコードは、`call_destroy`がtrueで、かつ`self.destroy`が`Some`の場合だけdestroy callbackを呼ぶ。

```rust
if call_destroy && let Some(destroy) = self.destroy {
    // SAFETY: destroy is called exactly once after in-flight wake callbacks drain.
    unsafe { destroy(self.userdata as *mut c_void) };
}
```

`&& let`でbool条件と`let` patternを一つの条件式に連結している。Rust 1.85.0ではこの位置のlet-chainが安定機能ではないため、CIは`E0658`（`let expressions in this position are unstable`）で停止する。次のように条件を入れ子にすれば、同じ順序とcallback呼出し条件を保てる。

```rust
if call_destroy {
    if let Some(destroy) = self.destroy {
        // SAFETY: destroy is called exactly once after in-flight wake callbacks drain.
        unsafe { destroy(self.userdata as *mut c_void) };
    }
}
```

### `crates/core-runtime/src/runtime.rs`

現在のコードは、subscriptionをowner bucketから外した後、identityの解決と`PropertyState`へのdowncastが両方成功した場合だけproperty側からもsubscriptionを外す。

```rust
if let Ok(state) = store.resolve_mut(identity)
    && let Some(property) = state.downcast_mut::<PropertyState>()
{
    property.remove_subscription(&removed);
}
```

こちらもRust 1.85.0で未安定のlet-chainを使う。失敗時は何もせず後続のowner cleanupへ進むため、次の入れ子条件なら現状のno-op条件と処理順を保てる。

```rust
if let Ok(state) = store.resolve_mut(identity) {
    if let Some(property) = state.downcast_mut::<PropertyState>() {
        property.remove_subscription(&removed);
    }
}
```

## 提案する変更

1. 上記2箇所を示した入れ子の`if` / `if let`へ書き換える。評価順、呼出し回数、失敗時のno-op動作を維持し、挙動変更を加えない。
2. `tools/units.yaml`の既存実装単位との対応を確認し、必要な場合だけ実装/test mappingを更新する。新しい製品挙動や要求は追加しない。
3. 同じ変更revisionでRust 1.85 minimum check、workspace unit tests、`tools/dev.py check`を実行し、Windows CIでfailureが解消したことを確認する。Linux CIも同じrevisionで確認する。
4. 結果を本計画書へ記録する。計画書のcommitは変更内容のレビュー後に行う。

## 検証

- `cargo +1.85.0 check --workspace --all-targets --locked`
- `cargo test --workspace --all-targets --locked`
- `python tools/dev.py check`
- `python tools/dev.py minimum`
- `python tools/check_docs.py`
- `git diff --check`
- CIでWindows/Linux両jobのminimum checkが通ること

## 承認境界

この計画は2026-10-22に利用者が承認した。変更はこの互換性修正に限定した。

## 実装・検証記録

- 実装revision: [`ae45097`](https://github.com/edamame-edame/freemvvm/commit/ae45097b6b348436a12c1a365a670a82360ff318)（Windows/Linux CIでclean worktreeを確認）
- Rust 1.85 minimum check: local成功（`.venv/Scripts/python.exe tools/dev.py minimum`）
- workspace unit tests: 成功（`cargo test --workspace --all-targets --locked`、156件）
- 開発check / 文書検査: 成功（`.venv/Scripts/python.exe tools/dev.py check`、C/C++ CTest 5件を含む）
- Windows/Linux CI: 成功（[run 39796877396](https://github.com/edamame-edame/freemvvm/actions/runs/39796877396)）。両OSの`check.json`と`minimum.json`が`passed`、`rust-minimum=0`、対象HEADは`ae45097b6b348436a12c1a365a670a82360ff318`。
