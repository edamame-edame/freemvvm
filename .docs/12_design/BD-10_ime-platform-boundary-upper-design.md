# MVVM UIライブラリ IME Platform境界に関する上位判断案

- 状態: BD-10承認済み（2026-09-26）
- 対応: 要求仕様第12節「IME compositionの詳細」のうち、Platform IME sessionとCore TextBox編集状態の責務境界
- 根拠: Accepted設計骨格第3～6・8節、承認済みR2～R4、V5～V8、A25～A28、BD-07～BD-09。DD-00は未承認

## BD-10 承認された判断

**Platform AdapterはOSのIME session、変換候補UI、native IME APIとの接続を所有する。Core TextBoxはアプリから観測できる編集文字列・selection/caret・入力状態を所有し、Platform AdapterはIMEからのpreedit／commit／cancelを共通eventへ変換して渡す。** OS固有のIME objectや候補windowをCore handleやABIへ露出させない。

| 責務 | 承認された上位契約 |
|---|---|
| Platform Adapter | OS IME sessionの開始・終了、native candidate UI、OS固有eventの受信を担う。TextBoxのcaretや入力位置に候補UIを配置するため、必要な座標・selection情報をCoreから受け取る。 |
| Core TextBox | 編集値、selection/caret、preedit状態、入力状態と理由を保持し、4言語から共通APIで観測できるようにする。Platform固有候補windowを状態のsource-of-truthにしない。 |
| Preedit更新 | Adapterは変換途中の文字列と編集範囲をCoreへ渡す。CoreはTextBox表示用の編集状態を更新するが、候補Validatorで確定判定せず、確定Property／ViewModelへ伝播しない（V7、A26）。 |
| Commit | Adapterのcommit eventで最終候補を受け取った時に、CoreがA25～A28の入力Validatorへ完全な候補・selection・入力元を渡す。Accept／Incomplete／Rejectの処理は承認済み規則に従う。 |
| Focus移動 | BD-09に従いPlatform sessionを終了し、未commit文字列と編集位置はCore TextBoxが保持する。再Focus時にAdapterは新しいsessionを開始し、OS候補状態は再生成される。 |
| ABI境界 | CoreとAdapter間の内部event shape、C ABI callback、candidate UIの座標形式、IME libraryの選定は詳細設計／Platform判断へ送る。 |

## 選択理由と範囲

- 組み込みLinux、Windows、macOSではnative IME APIとcandidate UIの仕組みが異なる。これらをPlatform Adapterへ閉じ、Coreに共通の編集値・入力状態を置くと、アプリ側のTextBox契約を揃えられる。
- A25～A28はView入力callbackとTextBox編集状態の読み取り・購読を既に定めている。BD-10はそれをPlatform IMEと接続する責務を明確にし、新しい入力Validator意味論を追加しない。
- 本判断は明示cancel／Escapeの意味、commit eventとFocusLostが競合した場合のPlatformごとのevent順序、文節候補・変換候補の描画・候補選択UI、surrounding textを使うIME APIを決めない。
- Text shaping、Unicode grapheme境界、RTL、candidate windowの詳細位置決めは別判断で扱う。

## 承認記録

2026-09-26に利用者承認。Platform AdapterがOSのIME session、変換候補UI、native API連携を所有し、Core TextBoxが編集文字列・selection/caret・入力状態を所有する。Preeditは編集表示状態のみを更新しValidator／ViewModelへ流さず、commit時にA25～A28へ渡す。Focus離脱時はBD-09に従って未commit文字列と編集位置を保持する。明示cancel、commitとFocusLostの順序、ABI・Platform API詳細は後続設計とする。DD-00の承認やDD-01以降へ進む判断ではない。
