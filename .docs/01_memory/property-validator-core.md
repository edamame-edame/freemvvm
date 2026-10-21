# 分類: Core Runtime / Property Validator

同期Validator callbackは利用者コードなので、`ui_objects` lockを保持したまま実行しない。PropertyからcallbackのArcをsnapshotしてlockを外し、Runtime単位のactive guardでsetterやBinding/Subscription変更の再入を拒否してから、commit用lockを再取得する。

kindを先に確認し、一致候補は同値を含めvalidatorへ渡す。受理した同値候補は既存表現を保ち、拒否候補は値とnotification queueを変更しない。Core setterのvalidator経路があっても、C ABI registration/Error handle、Binding初期同期、遅延伝播error snapshotは別実装単位として扱う。

関連: [Validator・Bindingエラー実装計画](../20_todo/validator-binding-errors.md)、[単体試験oracle](../32_unit_test/core-runtime.md#ut-core-property-validator-001)。
