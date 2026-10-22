# Runtime handle registration and borrowed resolution

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-026`.

- Runtime handles identify a RuntimeCore; they are distinct from Runtime-owned object controls, which carry Runtime identity and use RuntimeCleanupQueue for final UI-state cleanup.
- Use the same monotonic process issuer for Runtime identities and handle tokens so numeric collisions and reuse cannot occur across handle categories.
- A borrowed lookup should clone an internal Arc for the duration of a call. This lifetime pin alone does not authorize UI-affine operations; the resolved Runtime must still check its captured UI thread.
- Defer consuming the last Runtime token until shutdown completion behavior is implemented, so an arbitrary release thread cannot drop UI-affine pending state.
