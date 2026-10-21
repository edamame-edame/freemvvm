# Validator・Bindingエラー実装計画

- 分類: 実装計画 / Core Runtime・C ABI
- 状態: 作業中
- 根拠: 承認済みDD `runtime-objects/abi.md` §11・§13、`events/notifications-binding.md` の初期同期・伝播エラー契約、RQ-02-020/021・RQ-07-009/010

## 目的

Property setterとBindingが同一の同期Validatorを通し、拒否時に値・通知・Binding構造を部分更新しない。後続伝播の拒否はsetter成功と分離し、Binding固有のsnapshotと通知として観測できるようにする。

## 実装単位

1. `UNIT-CORE-064`: Core内のValidator保管、lock外callback実行、同値を含むsetter候補検証、再入setter拒否を実装・検証する。
2. `UNIT-ABI-015`: BOOL/I64/UTF8 Validator登録・解除、immutable validation Errorの生成と読取、Property setterおよびOneTime/OneWay/TwoWay初期同期への検証接続を実装する。拒否時は値・通知・graphを不変にし、callback ownershipとUI-thread teardownを確認する。
3. `UNIT-CORE-065` / `UNIT-ABI-016`: Binding後続伝播の拒否をsetter成功と分離し、Binding error snapshot・Property/direction情報・error subscription・通知順序・次回成功でcurrent errorをclearする動作を実装する。保持済みError handleは不変であることを確認する。

各単位は対応するoracle、`tools/units.yaml`登録、実装、focused/workspace tests、`tools/dev.py check`を同じrevisionで完了させる。現在は第1単位だけを進め、第2単位の設計・oracleを先取りして実装しない。

## 完了条件

- `UNIT-CORE-064`: kind検査後にsetter候補を検証し、同値もValidatorへ渡す。拒否時は値と通知queueを保ち、setter/Binding作成再入を状態変更前に拒否する。
- `UNIT-ABI-015`: Propertyごと最大1つのValidator登録、immutable validation Error、C ABI setter検証を実装する。Binding初期同期はtarget Validator検証後に値・graph・通知を一括commitし、拒否時はBinding未作成でError handleを返す。
- `UNIT-CORE-065` / `UNIT-ABI-016`: 後続伝播拒否をsetter成功と分け、Binding error snapshot、Property/direction、購読通知、次の成功時のcurrent error clear、保持済みErrorの不変性を検証する。
- wrong type/runtime/thread、invalid status/error handle、cancel・Property/Runtime teardownでのcallback ownershipとdestroy回数をABI focused testで確認する。
- `.docs/32_unit_test/core-runtime.md`、`tools/units.yaml`、`.docs/01_memory/`を同期する。

## 実行

1. 要求・DD・現行実装とtest documentの状態を再確認する。
2. test oracleを追記してからsourceを変更する。
3. focused Core/C ABI tests、workspace tests、format、`tools/dev.py check`を実行する。
4. 変更差分、文書リンク、unit registry、`git diff --check`を確認する。
5. 承認済みのunit単位commit/push規約に沿って完了状態を記録する。

