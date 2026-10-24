# C# F64 Property wrapper

- 更新日: 2026-10-24
- 状態: 完了（実装commit `a8f80fe`、Windows/Linux CI成功、2026-10-24）
- 対象: 既存のC# `UiProperty<T>` facadeに`double`を追加し、承認済みRQ-02-018のF64値表現をC#と実C ABIの境界で保つ。

## 背景と範囲

[C# Runtime wrapper基盤](csharp-runtime-wrapper.md)と[BOOL/I64 Property wrapper](csharp-property-bool-i64.md)により、Runtime所有基盤と`UiProperty<T>`の基本操作は実装済み。次は既存ABIにあるF64 create/get/setをC#から利用可能にする。UTF-8のmanaged文字列変換とDecimal64は独立した後続unitとする。

対象は`UiProperty<double>`の作成・取得・設定・close、ABI interop、fake ABIによる境界検査、および別process consumerによる実C ABI往復確認。C ABI/Coreの変更、Property通知・Binding・独自の値cache、Decimal64/UTF-8対応は含めない。

## 承認済み根拠

- 要求: [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-018](../11_requirements/RQ-02_core-features.md#rq-02-018)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)。
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。既存のversioned C ABIとRust Coreを利用し、CoreのF64同値判定をwrapperへ重複実装しない。
- 詳細設計: [C# wrapper DD v1.1](../13_detail_design/wrappers/csharp.md)、[共通ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)のProperty create/get/set規則。
- 先行実装: [C# BOOL/I64 Property wrapper](csharp-property-bool-i64.md)のgeneric facade、`TokenOwner`、`CallLease`、`UiResult<T>`、error snapshotとclose規則。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[release-test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装提案

- `UiProperty<T>`の対応型へ`double`を追加し、既存の`TryCreate(runtime, initial)`、`TryGet()`、`TrySet(value)`、`TryClose()`、`Dispose()`を再利用する。C ABIの`double`と.NET `double`を直接対応させる。
- create/set/getでF64のIEEE-754表現を数値変換・正規化せず境界へ渡し、getterの値をそのまま返す。wrapperに値cacheや同値判定を追加しない。
- Coreの規則によりNaN同士、同符号infinity同士、正負zeroは同値となる。同値setで既存格納表現を保持し通知しない意味論はCore/ABIの責務とし、このunitでは実C ABI consumerで代表値を確認する。
- UTF-8、Decimal64、通知・BindingおよびC ABI/Core変更は行わない。

## 試験提案

実装承認後、コード変更前に[wrapper test design](../32_unit_test/wrappers.md)へ`UT-WRAP-CS-PROP-F64-001`と`UT-CONSUMER-CS-PROP-F64-001`を追加し、`tools/units.yaml`へ`UNIT-WRAP-CS-PROP-F64-001`を登録する。

- fake ABI focused testで有限値、複数のquiet NaN bit pattern、正負infinity、正負zeroをcreate/get/setし、`BitConverter.DoubleToInt64Bits`とfake側の記録値でraw bit patternが保持されることを確認する。create/set/get失敗、unknown status、wrong-thread、close retry、lease中Disposeも既存owner規則に沿って確認する。
- 別process consumerから実C ABIへ接続し、有限値・infinity・NaN・signed zeroのcreate/get/set/closeを実行する。Core同値判定と既存格納表現保持の意味論を、C# wrapper独自の判定なしに確認する。
- focused test、consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書リンク・diff検査を同じ実装revisionで実行する。Windows/Linux CI成功後、計画・unit台帳・memoryへ結果とrevisionを記録する。

## 実施順

1. [x] 利用者が本計画のscope、F64 bit preservation方針、consumer検証範囲をレビューし承認した（2026-10-24、「ok、実装も進めて」）。
2. [x] 承認された計画をcommit `ef55b6e`（「C# F64 Property wrapper計画を承認記録」）で記録し、現在のC#作業ブランチへpushした。
3. [x] 実装開始承認後、`UT-WRAP-CS-PROP-F64-001`/`UT-CONSUMER-CS-PROP-F64-001`のtest designと`UNIT-WRAP-CS-PROP-F64-001` mappingをsource変更より先に追加した。
4. [x] C# facade/interopsとfake ABI focused testsへF64経路を追加した。
5. [x] 実C ABI consumerでfinite値、NaN payload、正負infinity、signed zeroと同値時の表現保持を検証した。focused/consumer tests、`tools/dev.py check`、`tools/dev.py minimum`、文書リンク検査、`git diff --check`が成功した。
6. [x] Windows/Linux CI成功後、unit台帳・計画・`.docs/01_memory/`を完了更新し、実装と検証記録を日本語でcommit/pushした。

## レビュー時の決定点

- C#の次unitをF64 Propertyとし、UTF-8/Decimal64から分けるscopeでよいか。
- C# `double`とC ABI `double`の間でbit patternを変換・正規化せず渡し、同値判定と保持意味論はCoreに任せる方針でよいか。
- 実C ABI consumerは意味論の代表値を確認し、bit patternの境界忠実性はfake ABI focused testで検査する分担でよいか。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | 計画scope・F64 bit preservation方針・consumer検証範囲 | 利用者 | 「ok、実装も進めて」により承認。実装開始も承認 |

## 検証結果

- 実装commit: `a8f80fe`（「C# F64 Property wrapperを実装」）。
- ローカル: `tools/dev.py csharp-property-test`、`tools/dev.py csharp-property-consumer-test`、Windows `tools/dev.py check`、`tools/dev.py minimum`、文書リンク検査、`git diff --check`が成功。
- GitHub Actions: [Development bootstrap run 40050770415](https://github.com/edamame-edame/freemvvm/actions/runs/40050770415)でWindows/Linux両job成功。
- 確認範囲: fake ABIでfinite値、複数quiet NaN payload、正負infinity、正負zeroのbit pattern往復、raw failure/error snapshot、wrong-thread、failed close retry。実C ABI consumerでNaN同士とsigned zero同値set後の格納表現維持を確認。
