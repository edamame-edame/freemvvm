# MVVM UIライブラリ Input eventの責務境界に関する上位判断案

- 状態: BD-07承認済み（2026-09-26）
- 対応: 要求仕様第12節「UI tree・Layout・Input・IME・Control」のうち、Platform Input AdapterとCore Inputの境界
- 根拠: Accepted設計骨格第3～6節、承認済みR2～R4・A25～A28・BD-05・BD-06。DD-00は未承認

## BD-07 承認された判断

**OSやdevice固有のraw inputはPlatform Input Adapterが受け取り、共通形式へ正規化してCore Inputへ渡す。Core InputはUI threadで共通の入力意味論を処理し、対象OSごとのControlが異なる動作を独自に決めない。** native controlを利用する場合も、ControlからCoreへ届く入力結果を共通の意味論へ接続する。

| 責務 | 提案する上位契約 |
|---|---|
| Platform Input Adapter | pointer device、keyboard、window system、IME等から届くplatform eventを受け、Coreが扱う中立な入力eventへ変換する。必要な時刻、modifier、device情報を保持する。 |
| Core Input | 正規化されたeventをRuntimeのUI threadで処理し、Visual Treeの配置情報やFocus状態を参照してControlへ渡す。共通Controlの既定動作と入力検証の接続を管理する。 |
| 座標と値 | pointer座標はCore Layoutが定める座標系へ変換する。座標原点、scale、整数pixelへの丸め、event structやABI表現は別途定める。 |
| TextBox入力 | 編集候補全文・選択範囲・入力元の受渡しとAccept/Incomplete/Reject、IME確定後の拒否でも入力を失わないA25～A28を維持する。 |
| native control | backendがnative controlからOS eventを受ける構成であっても、Coreが公開するControlの共通意味論と検証規則をbackend固有動作で上書きしない。native controlか独自描画かの選択は別判断。 |

## 判断範囲と後続項目

- 本判断はraw inputから共通Core Inputへの入口を定める。pointer target選択、keyboard focus、capture、event bubbling／tunneling、default action、gesture、keyboard navigation、IME compositionの状態遷移は定めない。
- どの種類の入力eventを初期公開するか、eventの順序・時刻・キャンセル、native controlとCoreの編集状態同期、Platformごとの対応機能は後続のInput/Focus/IME判断で扱う。
- callbackを呼ぶスレッドはR2～R4を維持する。workerからUI objectを直接操作しない規則、長時間処理は非同期へ委譲する規則も変更しない。

## 選択理由と限界

- 設計骨格はInput・Focus・IMEをCoreの共通機能とし、入力deviceやIMEをPlatform Adapterに分離している。raw eventを各Controlが直接扱うとOS間で挙動が分かれるため、変換点をAdapterに置く。
- Coreで入力意味論を保つことで、同じViewModel・Control操作をC++、C#、Rust、Python wrapperから一貫して使える。
- Inputの正規化だけでは、pointerとkeyboardでどのControlに届けるか、native IMEがどの状態を持つかは決まらない。それらはfocus・IMEとrendering方式の判断を経て定める必要がある。

## 承認記録

2026-09-26に利用者承認。Platform Input Adapterがraw eventを共通形式に正規化し、Core InputがUI threadで共通Controlの入力意味論を処理する。A25～A28のTextBox入力検証契約を維持する。focus、event routing、IME状態遷移は後続判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
