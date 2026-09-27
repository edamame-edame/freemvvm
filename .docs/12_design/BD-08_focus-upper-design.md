# MVVM UIライブラリ Focusの管理単位に関する上位判断案

- 状態: BD-08承認済み（2026-09-26）
- 対応: 要求仕様第12節「UI tree・Layout・Input・IME・Control」のうち、Focus対象とkeyboard入力先
- 根拠: Accepted設計骨格第3～6・8節、承認済みR2～R4、BD-05、BD-07。DD-00は未承認

## BD-08 承認された判断

**同一Runtime内のactive WindowとKeyboard FocusをRuntimeが一元管理し、active Window内の0個または1個のLogical Tree上Controlをactive focusとする。Keyboard／text入力はPlatformからCoreへ正規化された後、このactive focusへ渡す。Visual Treeのhit-test結果がFocus取得を要求する場合は、対応するLogical ControlへFocusを設定する。** OS/backend内のnative focus状態はAdapter経由でRuntimeへ同期し、別の独立したFocus source-of-truthを作らない。

| 項目 | 提案する上位契約 |
|---|---|
| 管理単位 | Runtimeはactive Windowを最大一つ、active focusを0または1つ持つ。Windowが非activeの間はactive focusを持たない。 |
| 非active Window | Windowは最後にFocusされていたControlを復帰候補として保持できる。これはactive focusではない。再active時に候補を復元する条件とタイミングは後続設計で定める。 |
| Focus対象 | Logical Tree上のControlとする。Visual Treeの内部要素は独立したFocus対象にせず、必要な場合は対応するLogical ControlへFocusを関連付ける。 |
| 入力先 | Runtimeのactive Window内のactive focusだけがkeyboardと確定textの入力先となる。active Windowがない、またはFocus Controlがない場合はCore Controlへkeyboard/textを送らない。Pointer hit-testはVisual Treeを使い、Focusを移す場合はBD-05の対応関係でLogical Controlへ結び付ける。 |
| Platform同期 | window systemのactivate/deactivateとnative controlのfocus通知はAdapterからRuntimeへ反映する。Coreのactive focus変更は必要に応じてPlatform側へ反映する。RuntimeとOSに独立したFocus状態を二重に持たない。 |
| treeからの除去 | Focus対象がWindowのLogical Treeからdetachまたは破棄された後、そのControlをFocus対象として残さない。FocusLostの具体順序と代替対象への移動は後続設計で定める。 |

## 判断範囲と後続項目

- Focusを移せるControlの条件（focusable、visible、enabled等）、Tab順、keyboard navigation、FocusScope、window deactivation時の復帰候補保持・解除は別途定める。
- FocusChanged callback/eventの公開、FocusLost／FocusGainedの順序、拒否可能なFocus移動、detach時の再Focus先は詳細設計に送る。
- IME compositionがFocus移動中にどう確定・取消されるか、TextBoxのselection/caret、A25～A28のValidator callbackとの順序はIME／TextBox設計で定める。
- Accessibility API上のfocus/state写像はAccessibility判断で扱う。
- 本案はRuntimeごとに一つのactive keyboard-focus domainを仮定する。複数seatから独立入力を同時に扱う要件が出た場合は、active focusをseatごとに持つ拡張を別判断する。

## 選択理由と限界

- active Windowとactive focusをRuntimeで一元管理すると、同じUI thread・dispatcherを共有するWindow間でkeyboard入力先が曖昧にならない。Window別の復帰候補はactive focusと区別できる。
- Logical ControlをFocus対象にすれば、templateでVisual Tree内部が置き換わってもアプリから見たFocus ownerが安定する。Visual hit-testは位置判定を担い、実際のControl意味論はCoreに残る。
- native focusとの同期は必要だが、両者を独立した状態として扱うと矛盾する。AdapterとCoreの同期契約、およびnative/custom controlの具体的な実装方式は後続設計が必要。

## 承認記録

2026-09-26に利用者承認。同一Runtime内のactive Windowとactive focusをRuntimeが管理し、active focusを0または1つのLogical Controlとする。非active Windowの復帰候補はactive focusと区別する。Keyboard／text入力はactive focusへ渡し、native focusはAdapter経由で同期する。Focus移動条件、keyboard navigation、IME、Accessibilityの写像は後続判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
