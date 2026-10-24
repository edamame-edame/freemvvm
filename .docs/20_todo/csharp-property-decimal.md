# C# Decimal64 Property wrapper

- 更新日: 2026-10-24
- 状態: 完了
- 対象: C# `DecimalValue` と `UiProperty<DecimalValue>` を追加し、承認済みのDECIMAL canonical `uint64` ABIを通じて値を作成・取得・設定する。

## 背景と範囲

C# Runtime wrapperとBOOL/I64・F64・UTF-8 Property facadeは実装済み。次unitでは既存C ABIのDECIMAL Property関数をC#へ写す。Decimal64の算術、文字列parse/format、Binding、Validator、C ABI/Core変更は含めない。

## 承認済み根拠

- 要求: [RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-019](../11_requirements/RQ-02_core-features.md#rq-02-019)、[RQ-02-020](../11_requirements/RQ-02_core-features.md#rq-02-020)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)。
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。言語wrapperは既存versioned C ABIを使う。
- 詳細設計: [C# wrapper DD v1.1](../13_detail_design/wrappers/csharp.md)は`DecimalValue`の`ulong bits`と`TryFromDecimal`/`TryToDecimal`による正確な範囲変換を定める。[DECIMAL ABI設計](../13_detail_design/events/notifications-binding.md)はsign 1 bit、scale 5 bit、coefficient 58 bitのcanonical encoding、およびcreate/get/set symbolを定める。
- 先行実装: [C# UTF-8 Property](csharp-property-utf8.md)のgeneric facade・fake ABI・別process consumer、[C++ Decimal64 Property test](../32_unit_test/wrappers.md#ut-wrap-cpp-prop-decimal-001)のcanonical bits・境界値・error/release検査。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[release-test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装提案

- `DecimalValue`を不変の公開値型として追加する。内部値はcanonical `ulong` bits。`TryFromParts(bool negative, byte scale, ulong coefficient)`はscale 0..31、coefficient 0..2^58-1だけを受け付け、範囲外はnative呼び出しなしで`UI_INVALID_ARGUMENT`相当の結果を返す。`IsNegative`、`Scale`、`Coefficient`で構成要素を読み取れるようにする。
- `TryFromDecimal(decimal)`と`TryToDecimal()`はDDに従い、数値を丸めず正確に表現できる場合だけ成功する。`decimal`の係数を96-bit整数として読み、Decimal64の係数上限を越える場合は10の倍数を取り除いても範囲内に収まる場合だけscaleを調整する。表現不能なら失敗を返す。Decimal64から.NET `decimal`へはscale 28超を、余分なscaleを落としても値が変わらない場合だけ変換する。
- `UiProperty<DecimalValue>`は既存C ABIの`property_create_decimal`、`property_get_decimal`、`property_set_decimal`へcanonical bitsをそのまま渡し、値の丸め・`double`/string経由変換・wrapper cacheを行わない。owner lease、error snapshot、失敗set時の値保持、close retryは既存generic facade規則に従う。
- 同数値表現の比較・格納表現保持・通知抑止はCoreの責務とし、wrapperでは変更しない。C#側では受信したbitsをそのまま`DecimalValue`として保持する。

## 試験提案

実装承認後、製品source変更前に[wrapper test design](../32_unit_test/wrappers.md)へfocused fake ABI testと別process real C ABI consumer testを追加し、`tools/units.yaml`へ独立C# Decimal Property unitを登録する。

- 値型: scale 0/31、係数0/最大、正負ゼロ、最大係数を検査。scale 32と係数上限超過をnative callなしで拒否する。
- `System.Decimal`変換: 正確な値・正負・scaleを確認し、係数上限を超える値、scale 28を越えて縮約不能な値は丸めず失敗する。10の倍数縮約で正確に範囲へ入るケースと.NET `decimal`の負のゼロも検査する。
- fake ABI: canonical bitsがcreate/get/setで完全往復すること、raw getter status、create/set error snapshot、failed set後の前値維持、failed close retryを確認する。
- 別process consumer: 実C ABIで境界bitsと代表的なscaleのcreate/get/set/closeを実行する。Coreの同値性・通知意味論の再検証はこのwrapper unitに重複させない。
- focused test、consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書リンク・diff検査を実装revisionで実行し、Windows/Linux CI結果を計画・unit台帳・`.docs/01_memory/`へ記録する。

## 実施順

1. [x] 利用者がscope、`System.Decimal`との正確な変換規則、失敗結果をレビューし承認した（2026-10-24、「ok，実装を進めて」）。
2. [x] 承認された計画をcommit `6899039`（「C# Decimal Property wrapper計画を承認記録」）で記録し、現在のC#作業ブランチへpushした。
3. [x] `UT-WRAP-CS-PROP-DECIMAL-001`/`UT-CONSUMER-CS-PROP-DECIMAL-001` test designと`UNIT-WRAP-CS-PROP-DECIMAL-001` mappingを製品source変更前に登録した。
4. [x] `DecimalValue`とDECIMAL C ABI interop、およびfake ABI focused testsを実装した。canonical bits・System.Decimal正確変換・範囲境界を検査した。
5. [x] 実C ABI consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書リンク検査、`git diff --check` がすべて成功した。
6. [x] Windows/Linux CI run [40053093079](https://github.com/edamame-edame/freemvvm/actions/runs/40053093079) が両OSで成功。unit台帳・計画・`.docs/01_memory/`へ結果を記録し、この完了記録をcommit/pushする。

## 実装・検証結果

- `DecimalValue`はcanonical `ulong` bitsを保持し、parts範囲検査と`System.Decimal`とのexact-only変換を提供する。DECIMAL Propertyは既存ABIへbitsを直接渡し、wrapper側の数値同値判定・正規化・cacheを追加しない。
- fake ABI testsと別process real C ABI consumerで、sign/scale/coefficientの境界、System.Decimalの係数縮約・scale 28境界・変換不能値、Property create/get/set/error/closeを確認した。
- Coreは数値同値のsetで既存表現を保持するため、consumerは各canonical表現を別Propertyの初期値として検査し、setterでは異なる数値へ変更して検証する。
- 実装commit: `166a6a00a57b542e9c471979a65bf6a89c3e688c`。CI: [Development bootstrap #40053093079](https://github.com/edamame-edame/freemvvm/actions/runs/40053093079)（Ubuntu/Windows success）。
- ローカル検証: `csharp-property-test`、`csharp-property-consumer-test`、`tools/dev.py check`、`tools/dev.py minimum`、`tools/check_docs.py`、`git diff --check` は成功。

## レビュー時の決定点

- `DecimalValue`がcanonical bitsを保持し、数値同値判定をwrapperに重複実装しないscopeでよいか。
- .NET `decimal`変換は値を丸めず、係数超過時は末尾の10の倍数を除いて正確に範囲内へ収まる場合だけ成功する規則でよいか。
- DECIMAL値型とPropertyを一つの小unitにまとめ、parse/format・Binding・Validatorを含めないscopeでよいか。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | 計画scope・正確なSystem.Decimal変換・Property範囲 | 利用者 | 「ok，実装を進めて」により計画と実装を承認 |
