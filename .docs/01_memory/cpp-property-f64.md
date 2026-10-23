# C++ wrapper: F64 Property mapping and verification

- 適用範囲: `freemvvm::Property<double>` のC++/C ABI mappingと検証。
- `Property<double>` は ABI の `double` create/get/set関数へ値を直接渡す。wrapperでNaN/infinityを制限せず、signed zeroやNaN payloadを変換・正規化しない。
- bit-preserving fake ABI testはwrapper境界での往復だけを検証する。F64の同値判定、同値設定時の表現保持、通知抑止はCoreの試験対象であり、wrapper testのpassから推定しない。
- ABI API tableを拡張したときは、positional fake API初期化子も関数宣言順に合わせる。create/get/set各型のslot数を数え、隣接テストが誤った関数pointerを受けないよう全wrapper suiteを再実行する。
- `tools/dev.py check` はC++ wrapper CTest 8件とbootstrap gatesを実行し、`tools/dev.py minimum` はRust 1.85互換性を確認する。relocated CMake package consumerも別工程でF64 finite/infinityを検証する。
