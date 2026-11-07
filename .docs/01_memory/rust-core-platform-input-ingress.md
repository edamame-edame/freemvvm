# [Rust Core / Platform input ingress] 作業時の観察事項

- `PlatformWindowPeer`のOS入力通知は、backend callbackからRuntime/Coreを直接操作させず、peerへ接続したsink経由で渡す。
- Runtime側sinkは`Weak<RuntimeCore>`とCore Window identityを保持し、owned eventを`Dispatcher::post_completion`へ渡す。post成功はUI処理完了ではなくenqueue受理を表す。
- UI dispatcher callback内でWindowのopen状態とgenerationを再検証する。閉鎖後・再open前のeventは破棄し、新generationへ付け替えない。
- `PlatformWindowPeer` sink設定methodにはdefault no-opを使って既存backend/test peerのsource compatibilityを保ち、入力対応backendだけが実装する。
- `tools/units/<category>.yaml`はJSON互換YAMLで、root manifest経由の`load_catalog`がID重複・依存先・case shapeを検証する。unit登録後にこのloaderを実行する。
