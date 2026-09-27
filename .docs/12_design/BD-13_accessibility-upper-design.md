# MVVM UIライブラリ Accessibility共通意味論に関する上位判断案

- 状態: BD-13承認済み（2026-09-26）
- 対応: 要求仕様第5・7・12節にある初期ControlのAccessibility情報とPlatform連携
- 根拠: Accepted要求仕様第5・7節、BD-05、BD-08、BD-11、BD-12。DD-00は未承認

## BD-13 承認された判断

**各`Widget`がPlatform中立なAccessibility意味情報をCoreに持ち、Platform Adapterがその意味情報をOSのAccessibility APIへ写像する。初期版ではWindow、StackPanel、Text、TextBox、Buttonについてrole、name、state、value（該当時）、focus、supported actionを公開する。Accessibility treeはLogical Treeを基礎とし、template等のVisual内部要素を既定では独立Widgetとして露出しない。** 独自描画したControlも同じAccessibility意味論を提供し、OSのnative controlや描画pixelから意味を推測させない。

| Control | 初期版で公開する意味情報 | 操作の接続 |
|---|---|---|
| Window | window role、name、state、active/focus状態 | Runtimeのactive Window／Focusと同期（BD-08） |
| StackPanel | group/container role、name、state、enabled/focusability情報、子の順序 | Logical Treeの子順をAccessibility treeへ反映。container自体のFocus可否・keyboard操作は後続細則 |
| Text | static text role、name、state、enabled/focusability情報、表示文字列に対応するvalue | 読み取り専用。独立actionなし |
| TextBox | editable text role、name、state、現在の編集value、focus、enabled/disabled、validation state | Focus・keyboard・IME・commit後Validationは共通Core InputとA25～A28を通す |
| Button | button role、name、state、focus、enabled/disabled | accessibility invokeは通常のButton activation／Command実行経路へ接続。実行中の再入はA21～A24を維持 |

## 推奨契約

- 各`Widget`はPlatform中立なrole、name欄、state、focusability/current focus、enabled/disabled、supported keyboard operationを公開する。意味上必要な場合にvalueとactionを加える。名前を付けない要素は空欄を表せるが、名前欄自体は全Widgetで観測できる。初期Controlで必要な意味情報を4言語から同じ概念で設定・取得できる。
- `AccessibleName`は表示文字列から独立した意味情報として扱い、初期縦断シナリオでは明示設定・取得を検証する。表示Textからのfallback、label-for関係、name合成規則は後続判断とする。
- Coreは意味情報とLogical Tree上の公開階層のsource-of-truthとなる。FocusやEnabled等の変化はCore stateの変更に追従する。
- Platform AdapterはCoreの意味情報・階層・状態変化をOS Accessibility APIへ写像し、OSから来た対応操作をCore InputまたはControlの既存操作経路へ戻す。Platformごとのnode/property名は共通契約へ持ち込まない。
- Visual Treeのtemplate部品は標準ではAccessibility treeへ独立公開しない。後に複合Controlや独立した意味を持つ領域が必要になれば、明示的に公開する拡張規則を設ける。
- 実装・検証の着手順はBD-12のWindows → Linux x64 → 組み込みLinux → その他に揃える。独自描画の初期ControlはWindows上で最初のAccessibility bridgeを検証し、Linux backendへ順に追加する。

## 選択理由と範囲

- BD-12の独自描画ではOSが画面上のControl roleやnameをnative controlから得られない。そのため、Coreが意味情報を持ち、AdapterがAccessibility APIへ明示的に公開する責務境界が要る。
- Logical Treeを基礎にすると、Control templateやrendererのVisual内部構成を変えても、アプリが組み立てた意味上の階層を安定させられる。Visual内部要素は必要性が確認されるまで露出させない。
- 初期縦断シナリオにあるaccessibility name取得をWindowsから検証し、同じCore意味論をLinuxへ展開できる。OS固有のAPI名やannouncement/event protocolを共通Control契約にしない。
- 本判断は各Widgetが持つ共通意味論と初期Controlの範囲を提案する。特定規格への完全適合や認証を宣言するものではない。

## この判断では決めないこと

- C ABIの具体struct、各言語のAPI名、動的property通知のevent payloadと順序。
- OSごとのAccessibility API type、version差、AT-SPI/UI Automation等のbridge実装詳細。
- labelとcontrolの関連付け、name fallback/合成、説明・help text、localization。
- role/state/valueの全列挙、TextBoxのselection/caret詳細、Textの読み上げ単位。
- hidden/offscreen/disabled nodeの公開規則、複合Control、virtualized collectionのtree表現。
- screen reader、switch access等の具体製品試験、規格適合基準、実機ごとの支援範囲。
- Tab order、keyboard navigation、event routingの追加規則。BD-08／BD-07の後続として別判断する。

## 承認記録

2026-09-26に利用者承認。各WidgetがCore内にPlatform中立なAccessibility意味情報を持ち、Platform AdapterがOS APIへ写像する。初期版ではWindow／StackPanel／Text／TextBox／Buttonのrole、name、state、focusability/current focus、enabled/disabled、supported keyboard operationを共通概念として公開し、該当するvalueとactionを扱う。Accessibility treeはLogical Treeを基礎にし、template由来のVisual内部要素は既定で独立公開しない。Button actionとTextBox操作は既存のCore Input／Command／Validation経路へ接続する。API詳細、name fallback、OSごとのrole/state写像、AT検証範囲は後続判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
