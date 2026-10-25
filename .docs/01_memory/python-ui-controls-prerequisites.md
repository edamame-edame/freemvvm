# [Python wrapper / UI Controls prerequisites]

- Python wrapper surfaceを実装計画に載せる前に、DDの公開API一覧・source class一覧・unit inventoryを照合する。Python Runtime/Property完了はBinding/Commandの存在を意味しない。
- 0.0.1 release計画はPythonを含む4言語すべてにProperty/Binding/Command/UI Control vertical sampleを要求する。Python側にはRuntime/Propertyまであり、Binding/AsyncCommand/Control/Window facadeは未実装。
- `impl-python/runtime-wrapper`は`impl-rust/core-ui-controls-c-abi`を祖先に持つ。C# UI Controlsは`impl-csharp/ui-controls-wrapper`で別途完了済みだが、Python branchへmergeせず、要求・共通C ABI・Python DDをPython実装の根拠にする。
- UI Controls plan v0.1がBinding/Commandを「既存」と誤認したため、実装開始前にv0.2でBinding/AsyncCommandを先行依存unitとして追加した。計画scopeの承認はDD/test design/implementation approvalと別に管理する。
- Binding wrapperは承認済みRQ-07と共通C ABIを正本とする。ABI上OneTimeは成功時Binding handle 0、OneWay/TwoWayは所有handleを返し、handle releaseだけでは接続を解除しない（明示cancelまたはRuntime shutdown）。Python DD v1.7とwrapper test design v0.9は2026-10-26に文書全体が承認済み。`UNIT-WRAP-PY-BINDING-001`は実装開始承認済みで、fake ABI 29件・実C ABI consumer・Windows local `check`/`minimum`が成功、Windows/Linux CI待ち。CI成功まではunitをcomplete扱いしない。
- 根拠と現状: [Python UI Controls wrapper計画](../20_todo/python-ui-controls-wrapper.md)、[0.0.1 release計画](../20_todo/release-0.0.1.md)、[RQ-09 UI scope](../11_requirements/RQ-09_ui-scope.md)、[Python wrapper DD](../13_detail_design/wrappers/python.md)、[初期UI Control C ABI](../13_detail_design/runtime-objects/abi.md)。
