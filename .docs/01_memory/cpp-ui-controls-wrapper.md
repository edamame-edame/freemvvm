# C++ wrapper: UI Control owners / tree operations / Window close request

- 適用範囲: C++17 `Widget`、`Container`、Window/StackPanel/Text/TextBox/Button、Window close-request wrapper。
- `ui_controls.hpp` は新しい公開header。既存の`Property<T>`、`Command`、`Subscription`、`RuntimeCleanupState`のowner/cleanup契約を再利用し、owned handleごとにcall前のcleanup nodeを予約する。
- Control create/property/tree getterは新しいownerを返す。Container childやWindow content getterの型は`Widget`のままとし、具体Controlへdowncastしない。Window content tree ownerはC ABI/Core側に残し、C++ child ownerはborrowed input。
- `Widget::close()`はhandle release、`Window::close_peer()`はnative peer操作。用途を混同しない。ButtonのCommand引数もborrowedで、getterはindependent retained owner。
- `WindowCloseRequest`はcallback中だけのweak scope borrowed view。callback後は`UI_CLOSED`を返す。`defer()`は新しいowned tokenにadoptし、deferred resolveとcloseを独立させる。callback rootはABI登録成功時にだけC ABI destroyへ移す。
- fake ABI focused coverageは`wrappers/cpp/tests/ui_control_tests.cpp`。実C ABI consumerとinstalled-header確認は`wrappers/cpp/tests/package_consumer/main.cpp`。入口は`tools/dev.py cpp-controls-test`と`cpp-package-test`。
- CMake再configureでWindows SDK libraryが見つからない場合は、Visual Studioの`VsDevCmd.bat -arch=x64`でtoolchain環境を初期化してから`tools/dev.py`を実行する。通常のcheck/minimumの意味は既存bootstrap gateを参照。
- `UNIT-WRAP-CPP-009`の実装commit `14b5d45`はWindows local focused/package/check/minimum pass後にpushし、Windows/Ubuntu CI run `40183464855`で成功を確認した。
