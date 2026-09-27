# MVVM UIライブラリ Async Command進捗配信に関する上位判断案

- 状態: BD-26承認済み（2026-09-27）
- 対応: 中間進捗の集約と、終端結果との競合・反映規則
- 根拠: C1～C4、R2～R4、BD-03、BD-25。DD-00は未承認

## 既存の承認済み契約

- C1～C4により、Async CommandはUI threadからworkerへ処理を委譲し、operation IDで実行を識別する。workerはUI objectを直接操作せず、`runtime_post`でUI threadへ進捗・完了を送る。
- 同一Commandの`Running`または`Cancelling`中は既定で再実行を拒否する。終端後の実行は新しいoperation IDを持つ。
- 進捗は実行中のみ受け付け、完了後に到着した進捗は捨てる。画面終了後は画面向け結果を反映せず、Runtime shutdown時には保留callbackが破棄され得る。
- `mvvm-semantics.md` §10.2は、進捗の頻度制御・重複排除・callbackをまたぐ順序を後続設計に残している。
- BD-25により、remote/long-running validationはViewModel AsyncCommandと明示状態Propertyで表す。進捗表示はProgressBar等にBindingできる。

## BD-26 承認済み判断

1. **進捗は中間状態であり、全サンプルの配信を保証しない。** Runtimeまたはwrapperは、同一operationの未配信進捗を新しい値で置き換えて集約してよい。ProgressBar等の表示Propertyは最新の受理値へ追随する。
2. **完了・失敗・キャンセルの終端結果は進捗値と別に扱う。** 進捗集約で終端結果を置き換えず、現在operationに属する終端通知を一度だけ適用する。終端結果を適用した後に処理される同一operationの進捗は破棄する。
3. **競合時はRuntimeがUI thread上で処理する順序とoperation IDを基準にする。** 終端結果を先に適用した場合、キューに残る同一operationの進捗は捨てる。進捗を先に適用した場合はその最新値を反映した後、終端状態へ遷移する。過去operationの通知は進捗・終端とも無視する。
4. **アプリは、業務上すべての中間更新を保持したい場合に進捗通知をイベントログとして使わない。** その用途はアプリ側のデータ記録で扱い、UI進捗Propertyには最新状態を反映する。
5. **詳細設計で決めるもの**は集約の格納場所、dispatcher wake-upの抑制、頻度・時間閾値、進捗の型と値域、複数進捗チャネルの識別、shutdownとcancelの競合処理とする。上位判断では具体的な間隔や数値を固定しない。

## 理由とトレードオフ

- 高頻度のworker更新をUI threadへ逐一配信せずに済み、進捗表示のために入力応答性やBinding配信枠を圧迫しにくい。
- 最新値への集約は、ProgressBarや状態ラベルなど現在状態の表示に合う。一方、すべてのサンプルをUI上で観測する必要がある用途には向かない。
- 終端結果を中間進捗から分離することで、高頻度の進捗更新が完了・失敗・キャンセル状態を押し流すことを防げる。
- `runtime_post`自体はshutdown中の実行を保証しないため、終端結果がRuntime停止後もUIで観測できる保証はしない。

## この判断では決めないこと

- 進捗値の具体的なABI型、範囲、割合以外の単位・メッセージ表現。
- 複数進捗チャネル、履歴保存、ログ・TelemetryのAPI。
- dispatcher queueのサイズ、coalescing実装場所、頻度や時間の閾値。
- worker内部の通知順序を保証する追加sequence fieldの有無。現在の上位案はRuntime上のUI callback処理順序を採用し、必要なsequence規則は詳細設計で評価する。
- cancel要求と正常完了がworker側で競合した場合の結果分類。これはC3に従い、workerが報告する実際の終了結果を使う。

## 承認記録

- 利用者承認（2026-09-27）。上記1～5を上位判断として採用する。
- 中間進捗は最新値に集約でき、各サンプルの配信を保証しない。
- 終端結果は進捗値と別に保持・適用し、一度だけ終端状態に遷移する。
- UI threadで終端が進捗より先に適用されたとき、残存する同一operationの進捗を捨てる。
- 詳細閾値や具体型は後続詳細設計で定める。DD-00は未承認のため、DD-01以降へ進まない。

集約の格納場所、dispatcher wake-up抑制、配信頻度・時間閾値、進捗型・値域、複数channel、shutdown/cancel詳細は後続設計で決める。DD-00は未承認のまま維持し、DD-01以降へ進まない。
