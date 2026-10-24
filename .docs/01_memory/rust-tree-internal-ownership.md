# Rust Core / UI Tree internal ownership

- カテゴリ: Rust Core・HandleRegistry・UI Tree ownership
- 用途: CoreObjectControlとLogical Treeの寿命接続を変更する際に参照

## 規則

- `CoreObjectControl`はpublic reference、Tree internal owner、call-duration `ValidationPin`を別々に数える。Pinを長期Tree所有として流用しない。
- 最初のTree internal owner取得時にRuntime cleanup reservationを先取りする。予約失敗時はowner count/Treeを変更しない。追加internal ownerはそのreservationを共有する。
- 最終public referenceがreleaseされてもinternal ownerがあればUI stateを維持する。最終internal owner release時は、public ref/internal owner/pinが全て0の場合だけ予約ticketをactivateする。
- internal ownerからpublic handleを発行する場合は、live ownerを根拠に独立した新tokenを発行し、token/table確保失敗では既存owner/countを維持する。
- Runtime shutdownでUI stateが既に破棄されたらpending reservationを破棄し、後続owner/pin dropからcleanup ticketを再発行しない。
- cleanup reservation activate/dropはregistry map lock外で行う。UI stateにはUI thread上の`ObjectIdentity`経由でアクセスする。
- `LogicalNode`はUI thread context、弱いparent identity、子ごとの`TreeOwner`を持つ。attachはpeer準備後のcommitで親子リンクを同時に設定し、`detach_all`は親の破棄前に子をDetached化して各subtree内リンクを維持する。
- Tree mutationとchildren由来public handleの発行はRuntime UI threadでのみ行う。`parent_ancestry`はUI storeから取得したparentを含む祖先snapshotで、child identityを含む場合はcycleとして拒否する。

## 実装と検証

- 所有制御と予約handoffは `crates/core-runtime/src/handle_registry/registry.rs` の `CoreObjectControl` / `ValidationPin` / `TreeOwner`。
- focused確認: `cargo test -p freemvvm-core tree_owner_`。
- `UNIT-CORE-067` はHandleRegistryのowner lifetime primitive。`UNIT-CORE-068` はTreeCoordinatorのLogical attach/commit/detachとchild-owner由来handle発行を扱う。Runtime UI object storeやC ABIへのWindow/Control接続は別unitで実装・確認する。
