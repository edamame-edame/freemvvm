# Scope: Window background color requirement and API change

- Current accepted RQ-09 has no user-visible Window background-color setting or unset/default behavior; RQ-09-073 covers later general Style/Theme/Resource scope and should not be silently shifted.
- Current Platform scene contract consumes normalized RGB in `SceneSnapshot` and forbids renderer-inferred colors. The user now requests per-Window configurable background and a Core-defined default when unset.
- This is a cross-layer requirement: RQ/UI Control behavior, ABI, each language wrapper, paint invalidation, SceneSnapshot propagation, and unit/requirement tests. Keep the request in a change branch before Core APIs per `.docs/02_git/rule.md`.
- Candidate initial default is opaque white `#FFFFFF` with normalized RGB; it is a proposal only until reviewed. Default, unset/reset semantics, alpha scope, API shape, and setter invalid-value atomicity must be explicitly decided before implementation.

### Scope: current review-draft status and API consistency

- Plan 0.1 was approved; this approval covers preparing requirement/design/test drafts, not approving their content or authorizing implementation.
- Draft requirement is RQ-09-074. Keep it out of the approved requirement count until the RQ-09 document revision is approved; preserve RQ-09-073 as later Style/Theme/Resource scope.
- Draft API shape uses per-Window `RgbColor` normalized RGB get/set/reset through C ABI; default is effective opaque white. Core owns validation and dirty propagation; wrappers map errors and do not cache state.
- C++ wrapper methods use `Result<T>`/`Result<void>` to retain the existing non-throw API convention. Python exposes a 3-float tuple; C# exposes `RgbColor`/`BackgroundRgb`/reset; Rust exposes `RgbColor` and `Result<_, UiError>` with existing `UiContext` boundary.
- Document revisions remain review drafts until the user approves each affected document set as a whole. No implementation, product test, commit, or push is implied by plan approval.
