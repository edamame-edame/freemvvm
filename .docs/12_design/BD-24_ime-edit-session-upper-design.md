# MVVM UIライブラリ IME edit sessionに関する上位判断案

- 状態: BD-24承認済み（2026-09-27）
- 対応: IMEのpreedit、commit、明示cancel、FocusLostとの競合
- 根拠: V5～V8、A25～A28、BD-09、BD-10、BD-23。DD-00は未承認

## 既存の承認済み契約

- BD-09はTextBoxからFocusが外れた時の未commit文字列と編集位置をCoreに保持し、commitやViewModel伝播は行わないとする。
- BD-10はnative IME sessionとcandidate UIをPlatform Adapter、編集文字列・selection/caret・入力状態をCore TextBoxが所有するとする。
- V7／A26はpreeditを候補Validatorで破棄せず、commit後に判定する。拒否されたcommitも理由とともにViewに保持する。
- V2／A18はView側が値を受け入れてもViewModel Propertyが拒否した場合、View値を巻き戻さずBinding errorを保持する。
- BD-23は初版から日本語UI文字列の表示・編集を保証候補に含め、IME経路をtarget別に検証する。

## BD-24 承認された判断

1. **IME sessionはTextBoxごとに識別し、session内のevent順を保つ。** Platform Adapterが開始時にsession identityを割り当て、preedit update、commit、明示cancelを同じsessionに結び付ける。終了済み／置き換え済みsessionの遅延eventはCore編集値へ適用しない。
2. **Preedit updateは表示用の未確定編集値だけを更新する。** preedit中はViewInputValidator、Property Validator、Binding伝播を実行せず、Core TextBoxだけが編集中の内容と位置を持つ。
3. **Commitは一度だけ確定候補として処理する。** Coreはactive preedit範囲をcommit文字列で置換した完全なTextBox候補を作り、A25～A28のViewInputValidatorへ渡す。Acceptは編集値を確定してBindingへ進める。IncompleteはTextBoxに保持してBindingを止める。Reject相当のcommitも破棄せず、理由付きの未受理編集値として保持する。ViewModel側の検証拒否は既承認V2に従ってBinding errorにする。
4. **明示cancelはpreedit開始直前のTextBox編集値へ戻す。** cancel前に保存した本文、caret、selectionを復元し、commit、Validator、Bindingを発生させない。候補windowを閉じるだけのキー操作はcancelとみなさず、AdapterがOS入力意味に基づいてsemantic cancel eventを送る。
5. **FocusLostは明示cancelと区別する。** BD-09を維持し、FocusLostでactive sessionを終了しても最後のpreedit文字列と編集位置はTextBoxに保持し、commitやValidatorを実行しない。再Focus後は新sessionとし、OS候補windowや変換内部状態の復元は保証しない。
6. **commitとFocusLostの競合はPlatform Adapterの受信順で一意にする。** FocusLost処理より前に受信したcommitは通常commitとして完了する。FocusLostが先ならsessionを閉じてpreeditを保持し、そのsessionから後着したcommitは無視する。Adapterは各target上でこの順序を再現できることを検証する。

## eventとCore stateの対応

| event | Core編集状態 | Validator／Binding | session |
|---|---|---|---|
| Start | composition開始範囲と直前編集snapshotを記録 | 実行しない | 新規開始 |
| Update | preedit範囲だけ置換し、caret/selectionを更新 | 実行しない | 継続 |
| Commit | commit文字列でpreedit範囲を置換し、最終候補を評価 | 一度評価。AcceptのみBindingへ進む | 終了 |
| Explicit Cancel | 開始直前の編集値・位置へ復元 | 実行しない | 終了 |
| FocusLost | 最後のpreedit編集値・位置を保持 | 実行しない | 終了 |
| stale event | 現在のTextBox状態を変更しない | 実行しない | 破棄 |

## 選択理由と影響

- preeditは表示内容であってcommit値ではないため、入力途中にValidatorやViewModelを動かさない既存規則を保つ。
- explicit cancelをsnapshot復元と定義すると、adapterごとのnative stateをCore契約へ漏らさずに済む。
- FocusLostはユーザーによる明示cancelとは異なる。BD-09のpreedit保持を残すことで、未確定文字を失わず、誤commitも行わない。
- session identityとevent順の契約は、candidate UI終了後やFocus移動後に古いeventが到着したとき、新しいTextBox状態を書き換えることを防ぐ。
- Windows／Linux x64／embedded LinuxでIME APIと利用可能IMEが異なるため、共通semantic eventが各profileで実機検証できる必要がある。実際のOS API、IME module、candidate UI座標変換、timestamp/counter形式は詳細設計へ送る。

## この判断では決めないこと

- Unicode grapheme cluster単位のcaret movement、selection affinity、word navigation、undo/redo。
- composition attribute、複数preedit segment、候補window内の候補選択・paging。
- surrounding text取得、dead key、handwriting、音声入力、アクセシビリティ経由の入力。
- event identityのABI表現、各OS APIの呼び順、IMEごとの例外・fallback。

## 承認記録

利用者は2026-09-27にBD-24を承認した。IME sessionを識別し、preedit中はValidator／Bindingへ流さない。Commitは完全候補に一度適用し、明示cancelはpreedit開始時の編集値・位置へ戻す。FocusLostは明示cancelと区別し、BD-09どおりpreeditを保持してsessionを終了する。commitとFocusLostの順序をAdapter受信順で扱い、終了済みsessionからの遅延eventは無視する。Undo/Redo、caret詳細、OS APIは後続判断とする。

BD-24は上位判断として承認済み。undo/redoやOS固有IME APIは後続判断とする。DD-00は未承認のため、DD-01以降へ進まない。
