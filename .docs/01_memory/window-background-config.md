# Scope: Window background color requirement and API change

- Current accepted RQ-09 has no user-visible Window background-color setting or unset/default behavior; RQ-09-073 covers later general Style/Theme/Resource scope and should not be silently shifted.
- Current Platform scene contract consumes normalized RGB in `SceneSnapshot` and forbids renderer-inferred colors. The user now requests per-Window configurable background and a Core-defined default when unset.
- This is a cross-layer requirement: RQ/UI Control behavior, ABI, each language wrapper, paint invalidation, SceneSnapshot propagation, and unit/requirement tests. Keep the request in a change branch before Core APIs per `.docs/02_git/rule.md`.
- Candidate initial default is opaque white `#FFFFFF` with normalized RGB; it is a proposal only until reviewed. Default, unset/reset semantics, alpha scope, API shape, and setter invalid-value atomicity must be explicitly decided before implementation.
