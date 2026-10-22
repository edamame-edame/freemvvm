# C++ wrapper: Property owner / C ABI adapter / cleanup

- 適用範囲: C++ wrapperでC ABI所有handleをRAII型にする実装と単体試験。
- `Property<T>`はbool、`std::int64_t`、`std::string`（UTF-8 byte列）を明示分岐でC ABIへ変換する。型を増やす場合はABI表現・エラー所有・境界値を個別に確認する。
- `std::string` UTF-8は指定byte lengthを維持し、embedded NULを許容する。C ABI getterは容量照会→copyとし、`UI_BUFFER_TOO_SMALL`なら値全体を再照会してやり直す。getter自体はerror tokenを持たない。
- C ABI `property_create_*` と `property_set_*` はerror handleを返すが、`property_get_*` はstatusだけを返す。C++ `UiError`のsnapshot有無は実際のABI関数契約に合わせる。
- native create前にProperty cleanup nodeとerror cleanup nodeを確保する。成功したownerは既存`RuntimeCleanupState`を共有し、明示close失敗でtokenを保持し、destructorのrelease失敗は共通cleanup supervisorへ移す。
- fake ABI単体試験とrelocated CMake package consumer試験を分ける。fakeはownership/error/status境界を、consumerは公開header・export target・同梱native runtimeの接続を検査する。
- `Property<std::string>` のcreate/get/set、buffer growth retry、embedded NUL、invalid UTF-8のsnapshot、close/destructor cleanupは実装unit `UNIT-WRAP-CPP-003` としてCIのWindows/Linux両方で確認した（run `39872968706`）。
