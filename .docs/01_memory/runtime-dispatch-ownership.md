# Rust Core Runtime: dispatcher work ownership

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-007`.

- Keep each queued item as one owned callback/destroy pair. A successful post transfers both actions to the dispatcher; invocation runs callback then destroy exactly once on the UI thread.
- A rejected post (allocation failure, Closed lifecycle, or poisoned queue synchronization) returns the intact work to the caller and does not run either action. The caller retains responsibility for destroy on registration failure.
- Shutdown closes posting and cancels queued work at a UI-thread safe point, running each accepted item's destroy action once without invoking its callback. Active-turn shutdown remains rejected for the coordinator to retry at a safe point.
- `UNIT-CORE-007` covers only ownership transfer and queued cancellation. Priority shutdown scheduling, host completion records, and public ABI integration remain separate units.
