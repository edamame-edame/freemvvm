<!-- scope: Git branch hierarchy and language-wrapper branch switching -->
# Git branch hierarchy memory

- Core API changes belong on a branch based on `main` or an optional version/change branch.
- Wrapper-language branches descend from the relevant Core APIs branch. When implementation language changes, create a new language branch from that Core branch; do not branch one wrapper language from another.
- A large approved implementation plan may be managed by one feature branch. Add feature/test sub-branches only when independent review or validation warrants them.
- Preserve old wrapper branches when correcting ancestry; create and publish a correctly based replacement branch instead of rewriting a shared branch.
