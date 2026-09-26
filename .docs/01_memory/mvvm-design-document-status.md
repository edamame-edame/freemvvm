# MVVM設計文書の状態と参照上の注意

確認日: 2026-09-26  
対象: `.docs/04_approval/` と `.docs/12_design/` を参照する設計・実装作業

- `adr-0001-rust-core.md` は `Accepted`。設計判断との優先関係は各文書の本文で確認する。
- `runtime-contract.md` の冒頭は `Review Draft（設計提案。承認前）`。R1～R4を承認済みと断定せず、状態を確認する。
- `mvvm-semantics.md`、`abi-spec.md`、`language-bindings.md` には、それぞれ承認済み判断の範囲が冒頭に記されている。文書全体の全記述が承認済みとは推定しない。
- `language-bindings.md` は今回の編集時点で存在する。参照時にはファイルの有無と状態を再確認する。
- `abi-spec.md` のC宣言は設計レビュー用であり、配布用ヘッダや実装済みAPIではない。
