# Scope: Core Runtime Binding validation error snapshots

- Keep validation error payloads as immutable `Arc<ValidationError>` values in Core. C ABI Error handles are issued from a snapshot when public APIs request one; do not store public handle tokens as Binding state.
- A Binding's current validation snapshot consists of the immutable error payload, rejected Property identity, and propagation direction. Clearing or replacing the current snapshot must not mutate a previously returned snapshot.
- `RuntimeBindingOwner` is the lifetime boundary for this state. Snapshot state must disappear when that Binding connection is canceled or Runtime shutdown clears active owners.
- Post-creation propagation validates candidate values outside Runtime object/graph locks. Rejection preserves the target group values, stops downstream edges by leaving their source at its committed value, and allows independent edges to continue. A later successful edge propagation clears the current snapshot while old Arc snapshots remain immutable.
- Core runtime tests cover OneWay chains, independent edges, target TwoWay-group atomicity, and recovery. C ABI snapshot/subscription and error-notification ordering remain separate work; check `.docs/20_todo/validator-binding-errors.md` before treating the full Binding error surface as complete.
