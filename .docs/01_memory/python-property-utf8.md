# Python wrapper / UTF-8 Property implementation

- `Property.create`はbool → int → built-in strの順でexact typeを判定する。UTF-8入力は`str.encode("utf-8", errors="strict")`で先に変換し、isolated surrogateはC ABI呼び出し前に`UnicodeEncodeError`にする。
- C ABI pointerはcall中だけ有効なbyte storageから作る。空byte列ではnull pointerと長さ0を渡し、embedded NULを含めbyte lengthを正として扱う。
- getterはquery/readを同じRuntime/Property leaseの範囲内で同期実行する。`BUFFER_TOO_SMALL`後は必要長を使い回すだけでなく再queryしてから再確保し、成功byte lengthだけstrict decodeする。invalid UTF-8/容量超過は`UiError(INTERNAL_ERROR)`。
- fake APIにはquery/read間に値が増える応答を入れ、再query、再read、非部分返却を確認する。owner lifecycleのclose retryとin-flight call/close競合もUTF-8 unitで再確認する。
- focused fake試験と別processの実C ABI consumerを`tools/dev.py`専用commandに分け、両方をrepository `check` gateへ加える。既存型試験も新しい`str`受け入れ契約と衝突しないよう更新する。
- 実装・検証記録: [Python UTF-8 Property計画](../20_todo/python-property-utf8.md)、[wrapper test design](../32_unit_test/wrappers.md)、`UNIT-WRAP-PY-PROP-UTF8-001`。CI確認前のため完了記録は保留。
