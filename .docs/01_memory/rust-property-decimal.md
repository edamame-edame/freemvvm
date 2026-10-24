[Rust wrapper / DECIMAL Property]
# Rust DECIMAL Property implementation notes

- Keep DECIMAL as a distinct `Decimal64` scalar, not `u64` in the public generic Property API. The canonical `uint64_t` bits stay private and pass directly to/from the C ABI.
- Layout: sign at bit 63, scale in bits 58..62, coefficient in bits 0..57. `try_from_parts` rejects scale >31 and coefficient >2^58-1 with INVALID_ARGUMENT; every encoded 64-bit pattern is otherwise canonical.
- Avoid `PartialEq`/`Eq` on the wrapper if it would imply representation equality: values such as 1.0 and 1.00, or signed zero representations, compare numerically equal in Core. Test equal setter behavior through a real Property consumer and inspect the preserved parts.
- Add raw DECIMAL create/get/set declarations to `freemvvm-sys`, then implement the sealed `PropertyValue` mapping in the safe wrapper. Keep `handle.rs` and attached snapshots untouched.
