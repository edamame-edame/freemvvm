# Validator・Bindingエラー実装計画

- 分類: 実装計画 / Core Runtime・C ABI
- 状態: UNIT-CORE-065完了（UNIT-CORE-064・UNIT-ABI-015も完了、次はUNIT-ABI-016）
- 根拠: 承認済みDD `runtime-objects/abi.md` §11・§13、`events/notifications-binding.md` の初期同期・伝播エラー契約、RQ-02-020/021・RQ-07-009/010

## 目的

Property setterとBindingが同一の同期Validatorを通し、拒否時に値・通知・Binding構造を部分更新しない。後続伝播の拒否はsetter成功と分離し、Binding固有のsnapshotと通知として観測できるようにする。

## 実装単位

1. `UNIT-CORE-064`（完了）: Core内のValidator保管、lock外callback実行、同値を含むsetter候補検証、再入setter拒否を実装・検証する。
2. `UNIT-ABI-015`（完了）: BOOL/I64/UTF8 Validator登録・解除、immutable validation Errorの生成と読取、Property setterおよびOneTime/OneWay/TwoWay初期同期への検証接続を実装する。拒否時は値・通知・graphを不変にし、callback ownershipとUI-thread cancelを確認する。
3. `UNIT-CORE-065`（完了）: 後続Binding伝播をedge単位で検証し、拒否Propertyと方向を含むimmutable error snapshotをBindingごとに保持する。source setterと他edgeの処理は成功のまま、target値・通知を維持し、後の成功伝播でcurrent errorをclearする。
4. `UNIT-ABI-016`（CORE-065後）: `binding_error_snapshot`でError・拒否Property・方向の新しいowned handleを返し、`binding_subscribe_error`を追加する。Binding伝播後・Property subscriber前のerror通知順、同一区間のcoalescing、cancel/shutdownのdestroy lifecycleを公開ABIで確認する。取得済みError handleはcurrent error変化後も不変とする。

各単位は対応するoracle、`tools/units.yaml`登録、実装、focused/workspace tests、`tools/dev.py check`を同じrevisionで完了させる。CORE-064とABI-015はこの条件で完了。snapshot ownershipを確立するCORE-065を先に終え、その後にABI-016を公開する。

## 現在の進捗

- CoreのValidator拒否payloadをimmutable `Arc<ValidationError>`として扱い、C ABI用handleへ変換する境界を整理した。
- `BindingConnectionId`を伝播edge snapshotへ保持し、`RuntimeBindingOwner`ごとのerror snapshot（error payload・拒否Property・direction）とcurrent errorのclearを試験した。
- Runtimeの実配信経路にValidatorをlock外実行するedge単位の処理を追加した。拒否edgeのtarget groupは値・通知をcommitせず、後続edgeは古いsource値を読み、独立edgeは継続する。次の成功伝播では同edgeのcurrent errorをclearする。
- Core focused testsではOneWay chain・独立edge・TwoWay target group atomic rejection・snapshot recoveryを確認した。workspace testsと`tools/dev.py check`も成功し、CORE-065を完了とした。

## 完了条件

- `UNIT-CORE-064`: kind検査後にsetter候補を検証し、同値もValidatorへ渡す。拒否時は値と通知queueを保ち、setter/Binding作成再入を状態変更前に拒否する。
- `UNIT-ABI-015`: Propertyごと最大1つのValidator登録、immutable validation Error、C ABI setter検証を実装する。Binding初期同期はtarget Validator検証後に値・graph・通知を一括commitし、拒否時はBinding未作成でError handleを返す。
- `UNIT-CORE-065`: 後続伝播拒否をsetter成功と分け、Binding単位のerror snapshot、拒否Property/direction、次の成功時のcurrent error clear、保持済みsnapshotの不変性を検証する。
- `UNIT-ABI-016`: snapshot handleの所有、error subscriptionの通知順/coalescing、cancel/shutdown時のclosure destroyを検証する。
- ABI focused testでBOOL/I64/UTF8 callback、重複登録、same-value setter、Error読み書き・retain/release、worker cancel拒否、OneTime/OneWay/TwoWay初期同期拒否とgraph/value不変を確認する。
- `.docs/32_unit_test/core-runtime.md`、`tools/units.yaml`、`.docs/01_memory/`を同期する。

## 実行

1. 要求・DD・現行実装とtest documentの状態を再確認する。
2. test oracleを追記してからsourceを変更する。
3. focused Core/C ABI tests、workspace tests、format、`tools/dev.py check`を実行する。
4. 変更差分、文書リンク、unit registry、`git diff --check`を確認する。
5. 承認済みのunit単位commit/push規約に沿って完了状態を記録する。

