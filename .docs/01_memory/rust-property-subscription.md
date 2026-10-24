[Rust wrapper / Property Subscription]
# Rust Property notification wrapper notes

- The approved notification ABI takes a no-argument callback plus userdata/destroy; subscribe does not issue an initial notification.
- Subscription cancellation and handle release are distinct. Releasing the public token alone leaves an active registration owned by the Runtime registry; Property teardown and Runtime shutdown invalidate registrations and destroy callback state after any in-flight invocation returns.
- Rust callbacks are UI-only and need not be `Send`. Use `FnMut() + 'static`; serialize invocation per Core contract and catch unwinds at the C callback boundary.
- Keep the callback root in an owned state object until Core calls destroy. On registration failure, retain/reclaim state on the Rust side. Reserve token/error cleanup tickets before calls that can return owned handles.
- The approved Rust DD 1.3 has only generic Property create/get/set/close, no Subscription API. Propose method signatures, `Subscription::cancel/close`, and ownership behavior in a whole-document DD revision before source changes.
