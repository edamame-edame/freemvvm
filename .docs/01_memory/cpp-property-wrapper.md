# C++ wrapper: Property owner / C ABI adapter / cleanup

- 適用範囲: C++ wrapperでC ABI所有handleをRAII型にする実装と単体試験。
- `Property<T>`の最初の実装はBOOLとI64に限定し、型ごとのC ABI呼出しを明示変換する。型を増やす場合はABI表現・エラー所有・境界値を個別に確認する。
- C ABI `property_create_*` と `property_set_*` はerror handleを返すが、`property_get_*` はstatusだけを返す。C++ `UiError`のsnapshot有無は実際のABI関数契約に合わせる。
- native create前にProperty cleanup nodeとerror cleanup nodeを確保する。成功したownerは既存`RuntimeCleanupState`を共有し、明示close失敗でtokenを保持し、destructorのrelease失敗は共通cleanup supervisorへ移す。
- fake ABI単体試験とrelocated CMake package consumer試験を分ける。fakeはownership/error/status境界を、consumerは公開header・export target・同梱native runtimeの接続を検査する。
