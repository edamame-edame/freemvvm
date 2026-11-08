# Git運用の学び: wrapper作業中のCore変更と他言語影響確認

- 対象: freemvvmのRust Core/C ABI変更が言語wrapper実装中に必要となった場合のbranch依存・横断検証。
- 関連ルール: [Git管理ルール](../02_git/rule.md)。
- wrapper branchでCore/C ABIを直接変更しない。作業を止め、未完了wrapper変更を保存して、最新の`version/...`または承認済み`change/...`を起点にCore APIs branchを作る。
- Core APIs branchでRust Core/C ABIを変更し、対応Core/ABI試験、workspace build/test/Clippy、最低Rust toolchain、公開header probeを同一revisionで検証する。
- Core変更が後方互換でも、C++/C#/Rust/Python等の既存wrapperのbuildとconsumer試験を実行し、ABI mapping・型・error/status伝達・package linkの影響を確認する。影響が見つかったwrapperは対象言語ごとの兄弟branchで更新し、独立に検証する。
- Core APIs branchをpushした後、各wrapper branchをその最新Core APIs branchから切り直す。別言語wrapper branchを親にしない。古いwrapper branchのcommitをそのまま取り込まず、現行の型・所有モデルへ機能を統合する。
- この順序を守る理由: 古いWindow owner facadeと現行UI Window型が重複し、言語wrapper branchの直接統合で同名型・API table・test catalogの衝突が発生した。Core変更とwrapper変更の依存順を分け、他言語を含む統合revisionで再検証する。
