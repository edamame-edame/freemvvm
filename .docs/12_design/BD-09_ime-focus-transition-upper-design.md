# MVVM UIライブラリ Focus移動時のIME compositionに関する上位判断案

- 状態: BD-09承認済み（2026-09-26）
- 対応: 要求仕様第12節「IME compositionの詳細」のうち、TextBoxからFocusが移る時点の未確定composition
- 根拠: 承認済みV5～V8、A25～A28、BD-07、BD-08。DD-00は未承認

## BD-09 承認された判断

**FocusがTextBoxから離れる時点でIMEがまだcommitしていない文字列（preedit）がある場合、Platform IME sessionは終了するが、Coreはpreedit文字列と編集位置をTextBoxの未確定編集値として保持する。Focus移動だけを理由にcommitやViewModelへのBinding伝播を行わない。** 後でFocusが戻った場合は文字列をTextBoxへ戻して編集を続けられるようにするが、Platform固有の変換候補やcomposition sessionそのものの復元は保証しない。

| 状況 | 提案する契約 |
|---|---|
| Focus離脱前にIME commit済み | A25～A28に従い確定候補を検証する。`Accept`は通常の値確定とBindingへ進み、`Incomplete`／`Reject`は既承認規則に従って編集値に保持する。 |
| 未commitのpreeditがあるままFocus離脱 | Coreはpreedit文字列と対応するcaret／selection位置を保持し、IME sessionを終了する。commit扱いせず、候補Validatorを確定候補として実行せず、ViewModelへ流さない。 |
| 同じTextBoxへFocusが戻る | 保持文字列を編集値として再表示し、IME sessionは新しく開始する。OS候補一覧、変換学習状態、候補segmentなどsession固有状態の復元は保証しない。 |
| TextBoxからdetach／破棄 | detachではFocusとIME sessionを終了し、Core objectが生きている間は未確定編集値を保持する。object自体の破棄後の永続保持は保証しない。A28のcallback解除とA32のclosure破棄規則を守る。 |

IME commit eventがFocusLostより先にCoreへ届いた場合は通常のcommitとして処理する。commit前にFocusLost/session終了が届いた場合はpreedit保持規則を適用する。event orderingの具体保証はPlatform adapter設計で定める。

## 選択理由と範囲

- 承認済みV7／A26は変換中の入力を捨てず、確定後に拒否された文字列も理由とともにView側へ保持する。Focus移動だけでpreeditを黙って破棄すると、この入力保持方針と利用者の期待を損ねる。
- Focus離脱時に強制commitすると、ユーザーが未選択の候補を確定してViewModel値まで変更する可能性がある。commitを伴わない編集値保持なら、未確定文字列を失わず、VM値を勝手に変えない。
- この判断はFocus transfer時のpreedit文字列保持を定める。明示的なIME cancel／Escape、Undo/Redo、複数TextBox間のIME context、caret/selectionの全状態、Platformごとのevent orderingとIME APIは後続設計に送る。
- IME候補UIはPlatformごとに異なるため、Focusを失ったTextBoxにOS候補windowを残さない。Coreに保持するのは編集内容と位置の情報であり、変換候補のUI状態ではない。

## 承認記録

2026-09-26に利用者承認。Focus移動時に未commitのpreeditがあれば、IME sessionを終了しても文字列と編集位置はTextBoxの未確定編集値として保持し、commitやViewModelへの伝播は行わない。再Focus時は文字列を編集状態へ戻すが、OS候補やsession自体の復元は保証しない。明示cancel、Undo/Redo、Platform event順序は後続設計とする。DD-00の承認やDD-01以降へ進む判断ではない。
