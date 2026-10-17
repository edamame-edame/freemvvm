# Rust Core Runtime: priority shutdown dispatch

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-008`.

- A shutdown request is an allocation-free coalescing flag, not an ordinary queued callback. It schedules a wake even when the ordinary queue is empty.
- If requested during a callback, let that callback finish. Posts accepted before the next shutdown turn remain owned by the dispatcher and are cancelled/destroyed in FIFO order on the UI thread; posts after lifecycle enters Closing return ownership to the caller.
- The shutdown turn cancels queued work before invoking its supplied shutdown handler, then marks the dispatcher Closed. UNIT-CORE-012 invokes this priority service at a successfully closed outer HostRegistration event boundary; Runtime cleanup integration must supply the handler.
- UT-RUNTIME-004 models both an ordinary callback and pending notification work as `PostedWork`. A real notification engine and its own delivery lifecycle need a later implementation unit.
