# C++ wrapper: Property notification subscriptions and callback failures

- 適用範囲: C++ `Property<T>`通知購読、`Subscription`所有、Runtime callback failure handler。
- C++ Property通知は`subscribe(std::function<void()>)`で登録し、初回通知は行わない。`cancel()`は登録を止め、`close()`/destructorはsubscription handleだけをreleaseする。明示cancelまではProperty teardownまたはRuntime shutdownがcallback stateを解放する。
- Coreへのcallback state所有移転はsubscribe成功後だけに行う。cancel中にcallbackがin-flightなら、Core destroy thunkはcallback終了後に一度だけ呼ばれる。self-cancelも同じ遅延destroy規則を通る。
- Runtime callback failure handlerはmutex下で共有recordをsnapshotし、lock外でinvokeする。handler交換後も実行中callbackがrecordを保持し、最後のinvoke後にuserdata destroyを一度実行する。`HandlerRegistration`のclose/destructorは自分のrecordがまだactiveな場合だけ解除する。
- C++例外はnoexcept C ABI callback trampolineで捕捉する。handlerがあれば`CallbackFailure`を渡し、handlerが未設定なら`std::exception::what()`またはnon-standard exception用固定文をstderrへ記録する。handler自体が例外を投げても固定文をstderrへ記録する。
- fake ABIの`AbiApi`は位置順aggregateなので、callback function pointerを追加したら全slotを宣言順に確認する。新しい公開headerはCMake `install(DIRECTORY include/)`でrelocated packageに含まれる。
- focused test `UT-WRAP-CPP-PROP-SUB-001`は5種のProperty型、初回通知なし、cancel/self-cancel、close単独、cancel/release失敗retry、in-flight callback lifetime、handler交換、例外stderrを確認する。package consumer `UT-PKG-CPP-006`はinstalled/moved headerとnative runtimeで購読変更cancelを確認する。
- 実装・検証の進捗と承認済み境界は[Property subscription計画](../20_todo/cpp-property-subscription.md)、設計根拠は[承認済みC++ wrapper DD](../13_detail_design/wrappers/cpp.md)と[通知DD](../13_detail_design/events/notifications-binding.md)、unit設計は[wrapper tests](../32_unit_test/wrappers.md)を参照する。
