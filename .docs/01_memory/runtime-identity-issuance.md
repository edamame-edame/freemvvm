# Runtime identity issuance

Scope: `C:\Users\yuki\Desktop\program\freemvvm`, `UNIT-CORE-024`.

- RuntimeCore identity must share `PROCESS_TOKEN_ISSUER` with public handle tokens. Separate counters could produce ambiguous identity values when later passed through Runtime-scoped registry validation.
- Allocate the identity before creating Runtime-owned components. If the monotonic issuer is exhausted or poisoned, fail construction rather than create an owner that cannot be safely identified.
- Keep the identity opaque/internal for now. `UNIT-CORE-024` does not implement public Runtime handles or C ABI entry points.
- Validate uniqueness under concurrent RuntimeCore creation and run the repository's documented checks before committing.
