# C++ wrapper: DECIMAL value and Property mapping

- 適用範囲: C++ `Decimal64`と`Property<Decimal64>`のcanonical DECIMAL mapping。
- canonical bitsは`sign=bit63`、`scale=bits58..62`、`coefficient=bits0..57`。scaleは0～31、coefficientは0～2^58−1。全64-bit patternが有効なABI表現なので、getterはparts検証を再適用せずbitsから値型を構築する。
- `Decimal64::try_from_parts`はC ABIを呼ぶ前にparts範囲を検査し、無効partsを`UI_INVALID_ARGUMENT`相当のraw statusで返す。native error snapshotは存在しない。bitsをmask・切捨て・丸めない。
- `Property<Decimal64>`はbitsを`property_create_decimal/get_decimal/set_decimal`へ直接渡す。数値上等価なbitsの比較・既存表現保持・通知抑止はCoreの責務であり、wrapperは再実装しない。
- `AbiApi`を追加するときは既存fake API tableのaggregate initializerを関数宣言順に拡張し、Runtime/BOOL/I64/UTF-8/F64テストを再実行してslotずれを検出する。
- relocated CMake package testはinstalled headersを使って`Decimal64`と`Property<Decimal64>`を確認する。新headerは`install(DIRECTORY include/)`でpackageへ含まれる。
