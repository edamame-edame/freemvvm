# C# wrapperの学び: UiWindowと背景RGB ownership

- 対象: `wrappers/csharp` にC ABIのowned handleを公開するWindow wrapperを追加する作業。
- 設計根拠: [C# wrapper DD](../13_detail_design/wrappers/csharp.md) 1.6、[wrapper単体試験設計](../32_unit_test/wrappers.md) 0.4。
- Runtime tokenはWindow生成呼出し中だけ`CallLease`で借り、Window tokenは独立`TokenOwner`へadoptする。失敗statusと非zero out tokenの組合せでは、所有権を回収してからerrorを返す。
- Getterを含む各C ABI操作はWindowの`CallLease`で保護し、同時Disposeでは最後のcall lease解放後にtokenをreleaseする。release失敗はowner retryまたはcleanup ticketへ委譲する。
- 背景RGBの既定値・validation・値の保持はCoreに委ね、wrapperでcache/validationしない。Try APIはraw statusをUiResultへ写し、property形式は同じTry経路でUiExceptionにする。
- fake ABIではcreate失敗、unknown status、interop例外、invalid色転送、close retry、CallLeaseとDisposeの競合を検査し、consumer testでは実C ABI経由のdefault/set/reset/closeを確認する。
- `tools/dev.py check`はWindows上のMSVC compiler 19.44とVS 18の19.50+要求標準ライブラリヘッダー混在でC++段階が失敗し得る。これはC# focused/consumer testと分離して原因を記録する。
