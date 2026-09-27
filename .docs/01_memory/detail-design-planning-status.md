# 詳細設計: 計画時点の承認境界

確認日: 2026-09-26  
対象: `.docs/13_detail_design/` を新たに作成する際の参照範囲

- 詳細設計のフェーズ計画は `.docs/20_todo/detail-design-phases.md`。各フェーズは小さなレビュー案を作り、判断IDと承認結果を同じ詳細設計文書に記録する。
- `design-overview.md` は Accepted。`runtime-contract.md` のR1～R4は承認済み（2026-09-26）。Runtime詳細設計では承認済み範囲と後続の未定義事項を区別する。
- `abi-spec.md` はA1～A32、`mvvm-semantics.md` はP1～P4・B1～B4・C1～C4・D1～D4・V1～V8、`language-bindings.md` はL1～L4を承認済みと記す。文書内の後続設計事項まで承認済みと拡張しない。
- UI tree、Layout、Input/Focus/IME、Control、Platform/Rendering、Accessibilityの詳細設計は、対応する基本設計の判断を先に確認する。
- `mvvm-ui-library-requirements.md` もAccepted。第12節の未決事項は別途判断する。DD-00の出典・判断対応表は `.docs/13_detail_design/00-decision-map.md` にレビュー案としてあり、未承認である。要求仕様の状態訂正をDD-00の承認とみなさない。
- `BD`は「上位設計」ではなく、基本設計（Basic Design）に係る判断箇所を示す識別子。保存時にファイル名へ`BD-xx_`を手作業で付ける運用であり、参照名との対応を確認する際はこの命名意図を踏まえる。
