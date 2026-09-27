# MVVM UIライブラリ 非同期Validationの初版範囲に関する上位判断案

- 状態: BD-25承認済み（2026-09-27）
- 対応: Core-managed async Validatorの有無、remote/long-running検証の表現
- 根拠: V1～V8、A17～A20、A21～A28、C1～C4、R2～R4。DD-00は未承認

## 既存の承認済み契約

- V1～V4はUI thread上の短時間同期Property検証、Binding伝播時の独立エラー状態、TwoWayの再同期、Binding作成時の原子性を定める。
- V5～V8は短時間のView入力Validatorを`Accept / Incomplete / Reject`で扱い、IME commit時の候補判定とViewModel側の不変条件検証を定める。
- C1～C4は非同期Commandを定義する。workerはUI objectを直接操作せず、進捗・完了結果をUI threadへpostし、operation IDで古い結果を識別する。
- BD-23／BD-24により、初版から日本語TextBoxとIME入力を対象にする。IME preeditはValidationやBindingへ流さず、commit後に既存Validatorへ渡す。

## BD-25 承認済み判断

1. **初版のCore Property ValidatorとView Input Validatorは、承認済みV1～V8どおり同期・短時間に限定する。** network、storage、長時間CPU処理を行うValidator callbackやCore-owned pending stateは初版に追加しない。
2. **remoteまたは長時間の判定は、アプリのViewModel側にAsyncCommandと明示的な状態Propertyを組み合わせて実装する。** 例として`Checking / Available / Rejected / Failed`状態や理由をViewModelに持たせ、Text、indicator、Buttonの実行可否へBindingする。AsyncCommandをProperty setterやIME commit callbackの内部で同期待ちさせない。
3. **非同期結果がどの入力値に対するものかはアプリ側で識別する。** value revisionまたはoperation IDを結果に関連付け、入力が変わった後に到着した古い結果をViewModel側で破棄する。C1～C4のAsyncCommand operation IDはCommand実行を識別する契約であり、独立した任意Validatorの自動世代管理を意味しない。
4. **非同期検証の`pending`はV5～V8の`Incomplete`と混同しない。** `Incomplete`は編集候補がまだ成立しない入力状態、pendingは非同期確認の完了待ちである。初版ではpending/errorの共通Property、複数validator結果の集約、validator callback ABIをCoreに設けない。
5. **将来Core-managed async Validationを追加する場合は独立した上位判断を行う。** 対象Property revision、同時要求の置換／併存、cancel、stale result suppression、複数エラー、shutdown中の結果破棄、diagnostic/transport errorの区別、wrapper ABIを先に決める。

## 例

ユーザー名入力後にサーバーで利用可能性を調べる場合、同期ViewInputValidatorは空欄・文字種・長さなどローカルで即時に判定できる規則だけを評価する。ViewModelはAsyncCommandでserver checkを開始し、状態とエラー理由をPropertyとして公開する。入力が変わった後に古い要求の結果が戻った場合は、入力revisionを照合して現在の表示へ反映しない。

同じく、日本語IMEのpreedit中はサーバー判定を始めない。IME commitがAcceptとなった値に対して、アプリがAsyncCommandを開始する。IME commit自体の同期的な範囲・書式検証はV5～V8に従う。

## 理由とトレードオフ

- UI thread上の短時間同期callbackという既承認契約を保てるため、TextBox入力、IME、Bindingの同期経路に非同期待機やpending/error競合を持ち込まない。
- C1～C4にはoperation state、進捗post、失敗、キャンセル、operation IDがあるため、アプリはこれを使ってremote checkのpending/resultを表示できる。
- async validation専用APIがないため、複数Control間で共通のpending/error表現や自動stale-result suppressionは提供されない。アプリ側での実装は多少重複する。
- 将来専用APIを追加するときは、取消競合、Property変更後のstale result、View/VM/Binding間のエラー所有権、言語wrapperへの非同期callback契約が必要になる。

## この判断では決めないこと

- AsyncCommandとViewModel状態Propertyを使うremote checkの具体型や命名。
- 共通validation error display、validation ruleの差し替えと再評価。
- 複数Propertyに依存するcross-field validationと、複数エラーの優先・集約。
- Core-managed async Validatorを将来追加する場合の具体ABI・wrapper/API、debounce、cache policy。

## 承認記録

- 利用者承認（2026-09-27）。上記1～5を初版の上位判断として採用する。
- 初版のbuilt-in Validatorは短時間同期に限定し、Core-managed async Validator/pending stateを入れない。
- remote/long-running checkはViewModel AsyncCommandと明示的状態Propertyで表す。
- stale resultの識別はアプリ側で行い、C1～C4 operation IDの既承認範囲を拡張しない。
- async pendingと`Incomplete`を区別し、将来の共通async Validation APIは別判断とする。

非同期Validation ABI、error aggregation、cross-property revalidationは後続判断とする。DD-00は未承認のため、DD-01以降へ進まない。
