# C# UTF-8 Property wrapper

- 更新日: 2026-10-24
- 状態: 完了
- 対象: 既存C# `UiProperty<T>` facadeに`string`を追加し、承認済みのUTF-8・長さ付きbuffer契約に従ってPropertyの作成・取得・設定を行う。

## 背景と範囲

[C# Runtime wrapper基盤](csharp-runtime-wrapper.md)、[BOOL/I64 Property wrapper](csharp-property-bool-i64.md)、[F64 Property wrapper](csharp-property-f64.md)で、Runtime所有基盤とscalar Property facadeは利用可能になった。次は既存C ABIのUTF-8 Property関数をC# `string`から利用可能にする。Decimal64は独立した後続unitとする。

対象は`UiProperty<string>`の作成・取得・設定・close、strict UTF-8変換、fake ABIによるpointer/length・buffer再試行検査、および別process consumerによる実C ABI往復確認。C ABI/Coreの変更、Property通知・Binding、Unicode正規化、Decimal64対応は含めない。

## 承認済み根拠

- 要求: [RQ-02-001](../11_requirements/RQ-02_core-features.md#rq-02-001)、[RQ-02-002](../11_requirements/RQ-02_core-features.md#rq-02-002)、[RQ-02-017](../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-04-003](../11_requirements/RQ-04_abi-boundary.md#rq-04-003)、[RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-04-021](../11_requirements/RQ-04_abi-boundary.md#rq-04-021)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-001](../11_requirements/RQ-06_threading-dispatch.md#rq-06-001)。
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。既存versioned C ABIとRust Coreを利用し、値とbyte列をABI契約に従って受け渡す。
- 詳細設計: [C# wrapper DD v1.1](../13_detail_design/wrappers/csharp.md)、[共通ABI DD v1.2](../13_detail_design/runtime-objects/abi.md)のUTF-8 Property create/get/set、[値・モデルDD v1.2](../13_detail_design/runtime-objects/objects-values.md)のUTF-8 ownership・caller buffer規則。
- 先行実装: [C# F64 Property wrapper](csharp-property-f64.md)と[BOOL/I64 Property wrapper](csharp-property-bool-i64.md)のgeneric facade、`TokenOwner`、`CallLease`、result/error・close規則。
- 参照試験: [C++ UTF-8 Property test design](../32_unit_test/wrappers.md#ut-wrap-cpp-prop-utf8-001)と実装unit `UNIT-WRAP-CPP-003`は、埋め込みNUL、invalid UTF-8、getter buffer不足、release failureの既存検証例。
- 試験方針: [unit-test strategy](../32_unit_test/unit-test-strategy.md)、[wrapper test design](../32_unit_test/wrappers.md)、[release-test matrix](../30_release_test/requirement-test-matrix.md)。

## 実装提案

- `UiProperty<T>`の対応型へ`string`を追加し、既存の`TryCreate(runtime, initial)`、`TryGet()`、`TrySet(value)`、`TryClose()`、`Dispose()`を再利用する。UTF-8 byte数を明示的な`ulong` lengthでABIへ渡し、NUL終端文字列として扱わない。
- C# stringからは例外fallbackを無効にしたstrict UTF-8 encoderでbyte列を作る。不正なUTF-16 surrogateを含む入力は`UI_INVALID_ARGUMENT`として拒否し、native callを行わない。有効な入力はUnicode正規化・文字置換をせず、埋め込みNULを含めbyte列のまま送る。空文字列はnull pointer・長さ0で渡す。
- getterは`buffer = null, capacity = 0`で必要byte数を照会し、確保したcaller bufferへ再取得する。`UI_BUFFER_TOO_SMALL`なら更新された必要長で再確保して再試行し、成功するまで部分bufferをmanaged stringとして公開しない。C# string/arrayで表現できない長さは割当て前に拒否する。
- byte列は例外fallbackを無効にしたstrict UTF-8 decoderでstring化する。不正UTF-8がABIから返った場合は置換せず、managed側のABI異常として失敗を返す。buffer queryとreadの間の同一snapshotは新たに保証しない。
- create/setのnative error snapshot、getのraw status、失敗set時の値保持、thread/owner lease、close retryは既存wrapper規則を再利用する。C ABI/Core、通知・Binding、Decimal64は変更しない。

## 試験提案

実装承認後、コード変更前に[wrapper test design](../32_unit_test/wrappers.md)へ`UT-WRAP-CS-PROP-UTF8-001`と`UT-CONSUMER-CS-PROP-UTF8-001`を追加し、`tools/units.yaml`へ`UNIT-WRAP-CS-PROP-UTF8-001`を登録する。

- fake ABI focused testで空文字列、ASCII、日本語、補助平面文字、埋め込みNULをcreate/get/setし、受信byte列・長さ・復号結果を検査する。孤立surrogateはnative call前に拒否し、fakeから返すinvalid UTF-8は置換せず失敗することを確認する。
- getterは空byte列のquery、required lengthのquery、buffer-too-small後にrequired sizeが増えるケース、短いbufferを一部変更しないこと、retry後の成功、unknown/wrong-thread statusを検査する。create/set/get error、failed setによる値保持、failed close retryも確認する。
- 別process consumerから実C ABIへ接続し、空文字列・ASCII・日本語・補助平面文字・埋め込みNULを往復してUTF-8 byte lengthと内容が保たれること、PropertyとRuntimeを明示closeできることを確認する。
- focused test、consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書リンク・diff検査を同じ実装revisionで実行する。Windows/Linux CI成功後、計画・unit台帳・`.docs/01_memory/`へ結果とrevisionを記録する。

## 実施順

1. [x] 利用者が本計画のscope、strict conversion、getter buffer retry、invalid UTF-16/UTF-8の失敗扱いをレビューし承認した（2026-10-24、「ok，実装してください」）。
2. [x] 承認された計画をcommit `a0247ef`（「C# UTF-8 Property wrapper計画を承認記録」）で記録し、現在のC#作業ブランチへpushした。
3. [x] 実装開始承認後、`UT-WRAP-CS-PROP-UTF8-001`/`UT-CONSUMER-CS-PROP-UTF8-001` test designと`UNIT-WRAP-CS-PROP-UTF8-001` mappingをsource変更より先に追加した。
4. [x] C# facade/interops、strict UTF-8 conversion、fake ABI focused testsを実装し、実装commit `b6bfb3b` を作成・pushした。
5. [x] 実C ABI consumer、`tools/dev.py check`、`tools/dev.py minimum`、文書リンク検査、`git diff --check` がすべて成功した。
6. [x] Windows/Linux CI run [40052347620](https://github.com/edamame-edame/freemvvm/actions/runs/40052347620) が両OSで成功。unit台帳と `.docs/01_memory/` に結果を記録し、この完了記録をcommit/pushする。

## 実装・検証結果

- `UiProperty<string>`を既存C ABIへ接続し、strict UTF-8 encode/decode、長さ付きbyte列、buffer不足時の再確保・再試行を実装した。不正なUTF-16入力はnative call前に拒否し、不正UTF-8出力は置換せず失敗する。
- fake ABI focused testsと別processの実C ABI consumerで、空文字列・ASCII・日本語・補助平面文字・埋め込みNUL、buffer growth、失敗後の状態保持・close retryを確認した。
- 実装commit: `b6bfb3bc74c63565b73069bf729b460b63e8cf76`。CI: [Development bootstrap #40052347620](https://github.com/edamame-edame/freemvvm/actions/runs/40052347620)（Ubuntu/Windows success）。
- ローカル検証: `csharp-property-test`、`csharp-property-consumer-test`、`tools/dev.py check`、`tools/dev.py minimum`、`tools/check_docs.py`、`git diff --check` は成功。

## レビュー時の決定点

- 次unitをUTF-8 Propertyとし、Decimal64から分離するscopeでよいか。
- 不正なUTF-16 surrogateを含むC#入力はnativeへ渡さず`UI_INVALID_ARGUMENT`にし、invalid UTF-8 getter出力は置換せずABI異常として返す方針でよいか。
- getterのbuffer不足はrequired lengthを更新して再確保・再試行し、queryとread間の同一snapshotは保証しない方針でよいか。


## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-24 | 計画scope・strict conversion・getter retry・不正文字の扱い | 利用者 | 「ok，実装してください」により計画と実装開始を承認 |
