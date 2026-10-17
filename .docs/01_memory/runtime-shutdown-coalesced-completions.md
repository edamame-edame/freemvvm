# Rust Core Runtime: coalesced shutdown completion ownership

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-018`.

- Dispatcher shutdown is coalesced, but each distinct accepted completion record remains strongly owned by HostRegistration until cleanup commit or host-stop notification.
- Repeated admission of the same completion deduplicates by `Arc::ptr_eq`; different completion records share the same cleanup pass and each transition to Completed afterward.
- Cleanup runs once before records are taken and notified. If cleanup fails, the host retains records for a later retry or stopped notification.
- Abrupt host stop drains the strong record list and marks each pending record HostUnavailable. Releasing the last external observer before termination does not cancel the accepted shutdown.
