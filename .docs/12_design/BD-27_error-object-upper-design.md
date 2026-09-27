# MVVM UIライブラリ Error objectの安定性境界に関する上位判断案

- 状態: BD-27承認済み（2026-09-27）
- 対応: error objectの機械判定コード、表示用message、原因・診断情報の責務
- 根拠: A2、A15、A18、V1～V4、C1～C4、BD-03。DD-00は未承認

## 既存の承認済み契約

- A2、A15、A18により、error objectはstatus、domain/code、messageを持ち、snapshotとして一定の寿命で参照される。
- Validation rejection、API argument error、Runtime/FFI failure、Binding failure、Async Command failureは同じ意味ではない。呼び出し元がそれぞれを区別する必要がある。
- Async Commandのworker exception/panicはABI越しに伝播せず、境界で失敗情報へ変換する（C2）。Binding validation failureも通常のAPI引数エラーやvalidator処理失敗と区別する（V1～V4）。
- BD-03はRuntime通知連鎖の停止・保留をdiagnosticとして観測可能にするが、その具体形式・購読方法は後続設計に残している。

## BD-27 承認済み判断

1. **機械分岐にはdomain＋code＋statusを使い、message文字列を条件判定に使わない。** domainはエラーを出したライブラリ領域、codeはそのdomain内の安定した理由、statusはAPI呼び出しの成否・分類を表す。意味を維持する追加は可能にし、既存codeの意味の変更・再利用はしない。
2. **messageは説明用であり、ABI上の安定識別子ではない。** messageの文面、言語、詳細度はminor更新やlocalizationで変わり得る。アプリが利用者向けに表示してよいが、機械処理や保存済みデータの識別には使わない。
3. **原因情報と診断情報を分ける。** 公開causeは元の失敗を理解するための任意の連鎖で、各要素もdomain/code/statusを保つ。診断情報はstack/backtrace、native OS detail、Runtime連鎖情報などの補助情報であり、存在・完全性・機密性を前提にしない。公開ABIでは、診断を機械契約として扱わない。
4. **error objectは生成時点のsnapshotとして不変に扱う。** 呼び出し後にmessageやcodeが別operationの結果で書き換わらない。保持期限、release方法、cause/diagnosticの取得方法はA15/A18の既定寿命契約を維持し、具体handle/APIは詳細設計で定める。
5. **初版では標準domain/codeの最小分類を固定し、全エラー一覧は詳細設計に送る。** 上位層はValidation、Binding、Runtime/API、Async operationの領域を識別できることを保証する。各領域の個別code、native OS codeとの対応、message catalog、cause depth/diagnostic size limitはDDで定める。

## 理由とトレードオフ

- messageの変更を許しながら、アプリがdomain/code/statusで安定した分岐を組める。
- causeを補助情報に限定すると、native OSや言語exceptionをそのまま公開ABIへ固定せず、各wrapperで安全に変換できる。
- diagnosticを非契約情報と明示することで、stack trace等の有無がtargetやbuild modeによって異なっても、通常のエラー処理を壊さない。
- 初版のdomain境界だけ先に固定し、完全なerror catalogやmessage localizationはControl・Platform実装を見ながら決められる。

## この判断では決めないこと

- 完全なdomain/code一覧、数値割当、reserved range、ABI表現。
- cause連鎖の上限深度、循環防止、diagnostic payloadの上限と権限管理。
- OS error code、Rust source location、stack/backtrace、panic payloadをどのtargetで取得するか。
- messageのlocalization、利用者向けvalidation表示の文言と文面規約。
- 各language wrapperでの例外型・Result型対応、error objectの具体取得・release API。

## 承認記録

- 利用者承認（2026-09-27）。上記1～5を上位判断として採用する。
- 機械判定をdomain/code/statusで行い、messageは安定識別子にしない。
- 公開causeと任意diagnosticを区別し、diagnosticを通常処理の前提にしない。
- snapshotの不変性と既存A15/A18寿命契約を維持する。
- 初版domain境界のみ上位決定し、code catalog/API表現はDDへ送る。

domain/code catalog、ABI表現、cause depth、diagnostic payload等は後続設計で定める。DD-00は未承認のため、DD-01以降へ進まない。
