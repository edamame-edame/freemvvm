# freemvvm: UI Input / Focus試験設計・unit登録

- カテゴリ: 試験設計 / UI Input・Focus / 0.0.1
- 対象: `.docs/32_unit_test/wrappers.md`、`.docs/32_unit_test/implementation-test-inventory.md`、`tools/units.yaml`

## 学んだこと

- Input route、handled、Focus、Button default actionはCoreの`ui.md`試験で検証し、ABI borrowed event契約と各wrapper固有のview寿命・callback root・例外/GIL境界はwrapper試験に分ける。
- C ABI/wrapper consumer試験はfake hostから入力を注入し、OS実入力の再現性に依存しない。Windows IME・keyboard/mouse/touch/pen実機動作はrelease qualificationに分離する。
- Input/Focus実装unitはCore、ABI、4 wrapper、Windows Adapterを独立した7 unitとして`planned`登録する。候補path/commandは実装開始時に実在確認後`ready`に更新し、plannedをCI pass扱いしない。
- close/releaseとcancelは別操作。callback-captured event viewはcallback後無効となり、各wrapperは言語固有の借用・所有表現でこの寿命を保持する。
- branch同期では`origin/version/0.0.1`固有のcommitがrelease TODO 1件だけであることをmerge-baseと3点diffで確認し、Core APIs branchへ取り込んだ。Windows/Ubuntu CI success後、Core child `impl-rust/core-input-focus`を作成・pushした。

参照: [Input/Focus実装開始計画](../20_todo/input-focus-event-path-implementation.md)、[UI試験設計](../32_unit_test/ui.md)、[wrapper試験設計](../32_unit_test/wrappers.md)、[実装unit台帳](../32_unit_test/implementation-test-inventory.md)。
