# C++ wrapper: input-event borrowed view / subscription

- 適用範囲: C++17 `Widget::subscribe_input`、`InputEventView`、Input callback root。
- `InputEventView`はmove/copy不可で、event scopeをweak参照する。callback中にだけC ABI borrowed tokenを使い、callback終了後はscopeが失効する。Text payloadはUTF-8をowned `std::string`へcopyする。
- C ABIのtext getterはcapacity不足時に`UI_BUFFER_TOO_SMALL`と必要byte長を返すため、最初のsize queryでこのstatusを許容してからbufferを確保する。empty textはsize queryが`UI_OK`になる。
- callback recordは成功登録時だけCoreへownershipを移す。cancelとsubscription handle releaseは別操作で、self-cancel中のdestroyはcallbackが戻った後になる。
- Input callback exceptionは`CallbackKind::kInputEvent`でfailure handlerへ渡し、default handlerはstderrへ記録する。例外発生時はcallback viewが有効なうちにeventをhandledにする。
- focused fake ABI testは`wrappers/cpp/tests/input_event_tests.cpp`、installed public consumerは`wrappers/cpp/tests/package_consumer/main.cpp`。CMakeのfocused suiteは`wrapper.cpp.input`。
- Visual Studio generatorでInput suiteと全C++ 15 CTestがpassし、installed packageを移動したconsumerでも5具象Widgetのregister/cancelがpassした。GCC `-fno-exceptions`ではInput実装とfocused test translation unitのcompile-only checkを通した。`clang-format`はこの環境に未導入のため自動C++ format checkは未実施。
