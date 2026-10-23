# C++ wrapper: Binding owner / error snapshot / callback category

- 適用範囲: C++ `Property<T>`間のOneTime/OneWay/TwoWay Binding、Binding token、遅延error snapshot、error Subscription。
- Bindingは`Binding<T>`とし、error snapshotが所有する拒否Propertyを同じ`T`の`Property<T>`として返す。同型のProperty間だけを公開し、異種型変換は行わない。
- `Property<T>::bind_to`のOneTime成功はnative handle 0を返すので`Result<std::optional<Binding<T>>>`の空optionalで表現する。statusとhandleを合わせて成功/失敗を判別し、失敗statusだけを見て初期値を変更しない。
- Binding error snapshot ABIはerror handleとrejected Property handleの2つを新規所有で返す。呼出し前にcleanup nodeを2個準備し、失敗時にtokenを失わず`UiError::ErrorSnapshot`と`Property<T>`へ移す。空snapshotはstatus成功・3出力zeroである。
- Binding handleの`cancel()`と`close()`/destructorを分ける。cancel成功後もhandleはreleaseされない。Binding releaseだけではCore接続を止めない。close/release失敗時はownerを保ち、retry可能にする。
- Binding error callbackは既存のSubscription callback state/trampolineを再利用するが、`CallbackFailure::kind`を`kBindingErrorNotification`に設定してProperty通知と分類する。登録失敗ではcallback stateをwrapperが保持・破棄し、成功後はCore destroyへ渡す。
- `binding.hpp`は`property.hpp`の後方からincludeし、単体include時もinclude guardで型定義の循環を避ける。CMake `install(DIRECTORY include/)`により追加公開headerはrelocated packageへ含まれる。
- fake ABI focused testは全5値型のmode/ownership/error pathsを制御し、relocated CMake package consumerはCore実接続のmode、値伝播、cancelを確認する。C++ public validator APIがない場合、遅延error stateの生成はfake ABIで検証し、Core/C ABI suiteを別のoracleにする。
