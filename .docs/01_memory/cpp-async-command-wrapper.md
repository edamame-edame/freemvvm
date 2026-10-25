# C++ wrapper: Async Command / worker lease / UI completion post

- 適用範囲: `wrappers/cpp`でC ABI Async Commandを公開するC++ wrapperとfocused fake ABI試験。
- `Command::close()`はCore側registration close、`release()`/destructorはowned command handle release。state/can-execute Propertyとfailure snapshotは独立owned handle。
- start callbackは受け取った`CommandWorker`をmoveして非同期作業へ保持できる。workerはCommand、cancel token、Runtime handleをそれぞれretainし、終端報告は`runtime_post`でUI loopへ返す。
- `runtime_post`が失敗した場合、post closureと各handle ownerをworkerへrestoreし、同じworkerからretry可能にする。成功はUI完了適用ではなくpost受付を意味する。
- C ABI `command_create_async`は成功時だけstart userdata ownershipを移管する。C++ callback rootは失敗時にローカルownerのまま破棄する。
- focused fake testは`wrappers/cpp/tests/command_tests.cpp`、relocated package consumerは`wrappers/cpp/tests/package_consumer/main.cpp`。実行入口は`.venv/Scripts/python.exe tools/dev.py cpp-command-test`と`cpp-package-test`。
- 2026-10-25 Windows localでfocused fake ABI、relocated CMake package consumer、`tools/dev.py check`がpass。unit `UNIT-WRAP-CPP-008`はWindows/Linux CI確認前のready。
- WindowsでCMake cacheを残したまま`tools/dev.py`が別MSVC toolsetを初期化するとcompiler/header不一致が起きることがある。`out/cmake/dev`の生成物だけを再作成後、通常の`tools/dev.py`入口で検証した。
