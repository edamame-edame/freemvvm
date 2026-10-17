# 設計文書から移管した検証項目

- 文書版: 1.0
- 状態: 試験設計資料・承認済み（2026-10-17）、未実行

各節は移管元・旧節を示す。外部要求のoracleは現行RQ、内部不変条件はDD/SDを根拠とする。下記旧文に未決案や過大なOOM保証があれば、現行RQの条件と後続のassertion表を優先する。数値の内部作業枠はunit-testで確認し、利用者保証の閾値にしない。

## 03-dispatcher.md / 6. 検証ケース

- 移管元: [ランタイム：実行基盤と終了](../13_detail_design/runtime-objects/runtime.md)

- Runtime作成thread上の操作、別threadからの`runtime_is_ui_thread`、UI専用APIの`UI_WRONG_THREAD`を確認する。
- workerから連続・同時postし、各callbackがUI threadで一度ずつ、同一Runtimeのqueue追加順に実行されることを確認する。
- UI callback内からpostし、現在のcallbackが戻るまで新callbackが始まらないことを確認する。
- 多数のpost callbackがある間も一turn一件の上限によりnative event loopへ制御が戻り、callback後の通知がBD-03の作業枠どおりに処理されることを確認する。
- wake pending中のpost追加、drain終了と同時のworker post、空queueへの最初のpostでwakeの重複・取りこぼしがないことを確認する。
- OOM、Closed、null callbackのpost失敗でclosure所有権がcallerに残り、成功後の実行/cancelでdestroyがUI thread上に一度だけ起きることを確認する。
- Closingとworker postを競合させ、同期点より前のpostはcancel、後のpostは`UI_CLOSED`となることを確認する。
- Runtimeが閉じた後のRuntime APIと、解放可能な残存handleの扱いをDD-LIFE-08に合わせて確認する。

## 04-callback-lifecycle.md / 8. 検証ケース

- 移管元: [イベントハンドリング：callbackと再入](../13_detail_design/events/callbacks.md)

- 登録・post成功時はCoreが所有権を引き受け、失敗時はcallbackもdestroyも呼ばれず呼出し側が後始末する。
- 登録解除、Property先行破棄、Binding解除、Command close、TextBox破棄、Runtime shutdownの各経路でclosureのdestroyがUI thread上で一度だけ起きる。
- callback自己解除後は次のinvocationが始まらず、現在実行中のcallbackは完了し、その後destroyされる。解除済みhandleのreleaseでdestroyは再実行されない。
- 同一Runtime内でcallbackが並列・再帰実行されず、callback中に作ったProperty通知はP3/P4に従って後続の配信回へ進む。
- 通知配信中の解除・追加は既存購読者順とP4の対象時点を守り、解除済みの後続購読者は呼ばれず、新規購読者は次回以降にだけ呼ばれる。
- notification callbackの言語例外後も他の購読者へ配信し、値を巻き戻さない。Validator・Command・TextBox callback例外は各種別の失敗意味論へ変換し、ABIを越えない。
- `destroy`からUI APIが呼ばれないこと、およびwrapperが言語側参照を一度だけ解放することを4言語で確認する。
- shutdown優先dispatchが実行中callbackを中断せず、未開始callbackを開始しない。破棄順はDD-02/03・各callback ownerの規則と一致する。
- 禁止規則は根拠となるcallback種別でerrorとなり、未証明の間接呼出しは誤ってerrorと断定しない。Property循環リスクとUI blocking候補はerrorでなくwarningとして区別される。
- callback markerが導入される場合、4言語でcallback entry pointと検知候補を解析でき、markerなしのclosure/callbackも同じRuntime動作を保つことを確認する。実行時にもmarkerを使う案なら、entry/exit計測とネストしたcallback列を別ケースで確認する。

## 05-boundary-values-errors.md / 5. 検証ケース

- 移管元: [オブジェクト管理：handle・値・error](../13_detail_design/runtime-objects/objects-values.md)

- 4言語のwrapperで、bool、符号付き64-bit、binary64、DECIMALの許容値を同じ意味で渡す。型不一致・整数範囲外は暗黙変換や切り詰めをせず拒否する。
- UTF-8入力で、空、マルチバイト、埋め込みNUL、不正sequence、null pointerと正length、長さ0を確認する。保持値が入力bufferの後続変更に影響されない。
- caller bufferで容量照会、正確な容量、過小容量、空値、埋め込みNULを試し、buffer不足時に出力bufferのbyte列が変わらず、必要byte数が返ることを確認する。
- getterの容量照会後に値を変更し、retryが新しい必要長を返せることを確認する。同一snapshotは保証しない。
- required out pointerがnull、無効handle、kind違い、wrong thread、closed Runtime、validation rejection、allocation failureで、statusと定義済みout値を確認する。
- `out_error`を省略、指定して成功、指定して失敗、error生成不能の各条件で、statusを維持しerror handleが0または所有handleになることを確認する。Error snapshotが後続操作で変化しない。
- 失敗するsetter/create操作が値・通知・登録状態を途中更新しないことを確認する。API個別の既承認callback side effectやCollection batch契約を共通rollbackと混同しない。

## 08-binding-graph.md / 5. 検証ケース案

- 移管元: [イベントハンドリング：通知とBinding伝播](../13_detail_design/events/notifications-binding.md)

1. OneTimeは初期source値を一度だけ反映し、handle・接続を残さない。OneWay / TwoWayは初期source値を同期してhandleを返す。
2. 初期同期時のkind違い、別Runtime、循環、入力元重複、validation拒否、資源確保失敗は値・グラフ・notification queueを変えない。
3. TwoWayグループの複数直接設定は最後に成功した直接setterが勝ち、失敗setterは勝者にならない。伝播更新は反射しない。
4. OneWay分岐は許可し、複数入力元・自己循環・間接循環およびTwoWay結合後に発生する循環を拒否する。
5. OneWay接続先の直接変更はsourceへ戻らず、次のsource変更で上書きされる。同一区間で両端を直接変更した場合はsource側が勝つ。
6. Binding伝播は外部Property callbackより先に終わる。callbackのProperty変更は次回配信へ進み、途中値を配信しない。
7. 明示解除・shutdown後の保留伝播は行わず、既存Property値を保持する。公開Binding handle releaseだけでは接続を解除しない。
8. 遅延validation拒否は入力元値を保ち、方向・拒否Property・immutable error snapshotを返す。後の成功伝播でBindingの現在エラーをクリアし、過去snapshotは変化しない。
9. 128件の作業枠に達した場合は完了境界でyieldして保留分を保持し、連鎖を停止しない。shutdown controlは残件より先に実行する。
10. 証拠条件を満たす循環では原因連鎖の未実行分だけを止め、独立連鎖・確定値を保つ。検出不能な長期反復を必ず停止すると扱わない。
11. WatchDog閾値を指定すると長期保留診断を一度記録し、無効化時はその診断だけを抑止する。WatchDog設定は作業枠のyieldや証拠に基づく循環停止を変更しない。
12. Binding error snapshotは循環・入力元重複・重複edgeをdomain 3の安定code 1～3で区別する。

## 09-validation-input.md / 8. 検証ケース案

- 移管元: [UIクラス：TextBox・Validation・IME編集](../13_detail_design/ui/text-editing.md)

1. BOOL/I64/UTF8/F64/DECIMALで型の一致したValidatorを登録でき、型違い・重複登録・wrong threadでは既存状態とclosure所有権が変わらない。
2. 登録時に既存値を呼ばず、以後は異値・同値candidateとも同期検証する。同値受理は値通知を増やさず、同値拒否は既存値を保つ。
3. Property Validator拒否・callback例外・不正status/error handleを区別し、setterをcommitせず、出力とerror handleをDD-VAL-04～06に従って扱う。
4. Property Validator callback中のUI状態変更・自己cancelを拒否し、例外/panicをABI境界の外へ出さない。
5. 明示cancel、Property破棄、shutdownでは新しいcallbackを始めず、in-flight callback完了後にdestroyをUI threadで一度だけ実行する。handle releaseだけではcancelしない。
6. TextBoxのkeyboard/paste/drop/accessibility候補が全文・selection byte位置・originとともにValidatorへ届く。登録なしでは候補をAcceptする。
7. `ACCEPT`のみ確定Property/Bindingへ進み、`INCOMPLETE`は編集値のみ、通常`REJECT`は編集値とselectionを保つ。各経路の状態・理由を取得できる。
8. IME composition中にValidatorを呼ばず、IME commitのReject候補を編集値として保持し、確定値とViewModelを変更しない。
9. ViewModel側ValidatorがTwoWay伝播を拒否してもTextBoxの入力表示・確定値を保ち、Binding errorを報告する。次の有効な伝播で再同期・error clearする。
10. 外部から異なる確定値が届くと編集中の値・状態・理由が初期化される。同値setterは入力途中の編集値を上書きしない。
11. 取得済みerror/reason snapshotは後続状態変化で不変。入力状態・理由・editing textの変化は初期通知なしのsubscriptionで観測する。

## 10-async-command.md / 10. 検証ケース案

- 移管元: [イベントハンドリング：非同期Command](../13_detail_design/events/commands.md)

1. 同Runtime BOOL条件Property、0（常時true）、異種kind、別Runtime、無効handle、wrong threadを試し、create失敗時にclosure所有権が移らない。
2. state/can-execute Propertyは読取専用で、生成時にnotificationを出さず、eligibility変化・state遷移と一致する。
3. execute不可・実行中の重複execute・operation record確保失敗はworkerを起動せず、ID出力0、state/error不変となる。
4. execute成功はIDを一度だけ発行し、start前にRUNNING/can-execute falseを確定する。start failureはexecute `UI_OK`/ID返却後にFAILEDへ移り、次回実行は新IDを使う。
5. cancelはrequested flagとCANCELLINGを先に反映し、worker終了を待たない。同一IDの重複要求は冪等で、stale IDは状態を変えない。
6. cancel要求後のworker successはSUCCEEDED、failureはFAILED、cancelを確認したworker報告はCANCELLEDとなる。
7. workerから直接Command/Property UI APIを呼ぶとthread制約で拒否され、`runtime_post`からの完了報告だけがUI thread上で適用される。
8. 過去ID・二重終端報告・close後報告は現在状態を変えない。failure snapshotはimmutableで、次operation開始時に現在snapshotが消える。
9. 進捗は全件配信を前提とせず、終端結果は中間progressと独立して一度だけ適用される。
10. command closeとRuntime shutdownはworker終了を待たず、新規execute/後着UI更新を止める。post受理前失敗とshutdownによるpost cancelの各経路でcallback/destroyとretain済みhandleが漏れず重複しない。
11. wrapper側workerがcancel tokenを照会し、worker終了まで必要な言語参照を保持し、終了後にreleaseする。Command本体・UI objectはworkerから直接使用しない。

## 11-ui-tree-lifetime.md / 6. 検証ケース案

- 移管元: [UIクラス：Tree・Layout・Geometry](../13_detail_design/ui/tree-layout.md)

1. 同一RuntimeのDetached WidgetをContainerへ追加し、parentから子を辿れるLogical順序と所属参照を確認する。
2. 既にparentを持つWidget、別Runtime Widget、self-parent、子孫をparentにする循環操作を拒否し、元のtreeと参照数を変えない。
3. Windowがcontent rootを一つだけ持つこと、StackPanelの追加位置と順序が保持されることを確認する。
4. remove後にchild handleからCore object/Propertyを利用でき、tree/描画/input対象から外れることを確認する。
5. detachされたsubtreeを別Containerへ追加し、内部の子順序とProperty状態を維持する。追加失敗ではDetachedのままである。
6. detachだけではBinding/Subscriptionが自動解除されず、明示解除・Property破棄・Runtime shutdownではDD-07/08に従うことを確認する。
7. parent破棄後、公開handle等が残るchildはDetachedとして有効、参照が残らないchildは一度だけUI thread cleanupされる。
8. tree変更をworker threadから呼んだ場合はwrong-threadで状態不変。shutdown後の変更はUI_CLOSEDで状態不変。
9. callback中にtree変更した際の再入・通知開始時点がDD-04/07と矛盾しないことを確認する。
10. Visual peer再生成失敗、focus/IME cleanup、template由来Visual ownerなど本書外境界を後続設計で検証する。

## 13-input-focus.md / 7. 検証ケース案

- 移管元: [UIクラス：InputとFocus](../13_detail_design/ui/input-focus.md)

1. 4言語でRuntime内active Window/active focusを一意に観測し、二つ目のactive Windowが先のwindow/focus stateを置き換える。
2. 有効focusable/enabled attached controlへの要求だけがFocusを移し、invalid、Detached、他Window、wrong-thread要求は状態を変えない。
3. Focus AからBへ移すと、commit済みstateをA.FocusLostが観測し、その後B.FocusGainedが呼ばれる。同じControlの再要求はduplicate eventを出さない。
4. Window deactivationでactive focusを解除し候補を保持、再activateで有効candidateを復元する。candidateがdetach/disable/release済みなら復元しない。
5. focused controlまたは祖先をdetach/破棄するとFocusLostを一度出し、旧Controlをactive focus/candidateに残さず、自動fallbackを行わない。
6. Keyboard inputはactive focusへ一度だけ配送する。focusなし/active Windowなしでは破棄し、後から再配送しない。
7. keyboard key eventはactive focusへ配送し、特定key/modifierに本書が暗黙のUI動作を割り当てていないことを確認する。
8. key down/upとtext commitを別eventとして扱い、TextBox commitのみA25～A28 Validator経路へ渡す。
9. BD-09/10どおり未commit preedit中のFocus離脱で編集内容を保ち、確定値・ViewModel値を変更しない。先着commitなら通常commitとなる。
10. callbackからのFocus request、Window deactivate、shutdownで現在callbackを打ち切らず、DD-03/04のqueue・destroy順を守る。

## 14-textbox-ime.md / 6. 検証ケース案

- 移管元: [UIクラス：TextBox・Validation・IME編集](../13_detail_design/ui/text-editing.md)

1. TextBox生成後、committed/editing textが初期値と一致し、selectionが有効な境界上にあり、初期通知を発生させない。
2. 通常編集で候補全文・post-edit selection・originを一度Validatorへ渡し、Accept/Incomplete/RejectでDD-09の規則どおり状態とBindingを更新する。
3. 日本語UTF-8、ASCIIとmultibyteの混在、結合文字列に対してselection/caret位置が不正なUTF-8 interiorを指さず、callback位置はbyte offsetで渡る。
4. IME Start/複数Update中はpreedit rangeだけが変わり、Validator・committed Property・ViewModelは変わらない。
5. Commitは完全候補を一度だけValidatorへ渡し、Accept/Incomplete/Rejectの各規則でcandidateとcomposition stateを一貫して終端する。duplicate/stale commitは二重適用しない。
6. Explicit Cancelは開始snapshotへ戻り、validator/Binding/committed Propertyの変更を起こさない。cancel後のinput state/reasonは選択した承認案に従う。
7. FocusLost前に受信したcommitとFocusLost後のstale commitをBD-24のAdapter順で区別し、後者がCore編集値を変更しない。
8. FocusLostはpreeditとselection/caretを保持し、再Focusで文字列を復帰しても旧session/candidate UIを再利用しない。
9. 外部から別値が確定するとA31どおりediting/input stateを更新し、古いIME sessionのeventは適用しない。同値setterは編集中の値を保つ。
10. TextBox detach/destroy/shutdownでsessionを閉じ、DD-04/09とA28に従ってcallbackを無効化・destroyする。遅延IME eventは破棄する。
11. candidate preparation/commit recoveryでallocation failureが起きても既存編集状態と候補を失わず、必要なdiagnosticを観測できる。

## 15-controls-v1.md / 6. 検証ケース案

- 移管元: [UIクラス：Window・基本Control・次段階Control](../13_detail_design/ui/controls.md)

1. 5具象Controlが同Runtime上で作られ、Widget/Container分類、Window single-root、StackPanel orderがDD-11制約と一致する。
2. StackPanel orientation/layout/AlignがDD-12に従い、未承認spacing/margin propertyを暗黙に作らない。
3. Text.ContentはUTF-8値として読め、OneWay updateで表示値/Accessibility valueが更新され、TextBox editing stateへ結合しない。
4. TextBox.Value TwoWayとediting/preedit状態を区別し、DD-09/14のAccept/Incomplete/Reject、IME focus loss、外部値更新に従う。
5. Buttonなし/Command設定/Command条件false/Command実行中/parent disabledを試し、execute回数とeffective enabled/accessibility stateが合う。
6. AccessibleNameは表示文字列から独立し、未設定を空欄として読み取れる。name fallbackは起きない。
7. disabled ControlはFocus要求・Button activation・accessibility stateで同じeffective enabled値を観測する。ancestor disabled時のpropagation案も確認する。
8. detached subtreeはDD-11に従いWindow/Layout/Input/Accessibility対象から外れ、再所属時にControl Core値・Property/Bindingが保たれる。
9. Command associationの変更、Button破棄、Window close、Command owner close/shutdownでA21～A24/DD-10のlifetime契約を壊さない。
10. Pointer activationとAccessibility invokeが共通semantic routeを通る。keyboard shortcutを本書が暗黙に付与していない。

## 16-platform-rendering.md / 6. 検証ケース案

- 移管元: [プラットフォーム：抽象化と共通protocol](../13_detail_design/platform/abstraction.md)

1. WindowsでCore Window/peer/surface生成から初回frame表示まで進み、Control stateとboundsがDD-11/12/15どおりRendererへ渡る。
2. Linux x64で同じControl snapshotをPlatform backendへ渡し、OS native controlへ意味状態を委譲せずframeを表示する。各OSの具体API選択自体は本書の検証合格条件にしない。
3. Property/Tree/Layout変更が複数回起きても未開始frameでdirtyを集約し、Layout完了前の古いboundsを新frameへ混在させない。
4. resize、minimize/suspend、surface restoreでCore Tree/Propertyを保持し、必要なLayout/frameを再生成する。
5. Surface作成失敗、graphics resource failure、present failureで部分snapshotを成功扱いせず、診断可能なdirty/failure状態を保つ。
6. OS pointer/keyboard eventがUI thread上で一度だけ正規化され、DD-13 route/Focus semanticsへ接続される。Keyboard shortcutを暗黙に追加しない。
7. TextBox focus、preedit/update/commit/cancel、FocusLost、detach、外部Value更新の各境界でOS sessionを適切に開始/終了し、stale session eventを適用しない。
8. Frame連続要求、通常post、Property通知の負荷中もDD-03 bounded turnを保ち、host loopへ制御を返す。
9. Runtime shutdownとsurface wake/present/IME eventを競合させ、現在のcallback/event終了後に優先shutdownし、未開始通常workをcancelし、destroy順を一度にする。
10. Window close intentでRuntime shutdownを暗黙発生させず、accept/decline/deferの選択後にnative peerとCore Windowが定義どおりの寿命を持つ。

## 18-wrapper-cpp.md / 8. 検証観点

- 移管元: [言語ラッパー：C++](../13_detail_design/wrappers/cpp.md)

- move後に元wrapperがreleaseしないこと、cloneが別tokenを一つretainして独立にreleaseできることを確認する。
- Borrowed viewの破棄がCore参照数を変えず、呼出しscopeを越えて保存する型がないことを確認する。
- UI thread上の最終release、worker上のrelease、finalizer queue enqueue失敗、明示close再試行、destructorからcleanup registryへの移管とdrainを分けて検証する。
- Runtimeをshutdownせず破棄した場合、shutdown後に子wrapperを解放した場合、残存Error handleを読む場合の状態を検証する。
- callback登録成功/失敗、初回callback時点、callback自己解除、owner close、shutdown、C++ exception、destroy一回性を確認する。
- Property notification exception後に利用者handlerを呼んでから他subscriberへ通知を続ける。handler未設定時の既定診断記録はAPI設計章で定める。
- status、known/unknown domain/code、error snapshotの保持、error allocation failure、UTF-8 multibyte/embedded NUL/buffer不足を検証する。
- UI-thread APIをworkerから呼ぶ場合に自動dispatchせず、post worker更新はUI threadで実行することを確認する。
- UI callbackからshutdownするとpendingを区別し、worker同期shutdownはcleanup完了まで待つ。停止loop条件は未決に追跡する。
- 例外有効/無効の両方を検証対象とする。最低C++標準、具体compiler matrix、target calling conventionはDD-22で確定するまで、配布可能header適合テストを完了扱いにしない。

## 19-wrapper-csharp.md / 9. 検証観点

- 移管元: [言語ラッパー：C#](../13_detail_design/wrappers/csharp.md)

- `ulong`/`uint`/`byte`のP/Invoke width、calling convention、out初期化、ABI major/feature queryをC headerとの相互コンパイル・interop testで確認する。
- callback registration/post後に強制GCを複数回行い、delegate、closure target、userdata stateがCore destroyまで生存する。
- registration failure、self-unsubscribe、in-flight disposal、owner shutdown、post cancelの各経路でGC root/`GCHandle`が一度だけ解放される。
- native callとmanaged Disposeをworker/UIから競合させ、leaseの競合とcallback再入でdeadlock/use-after-freeが起きない。
- finalizer threadからのrelease failure、cleanup ticket移管、UI safe-point retry、Runtime shutdown後drain、ticket allocation失敗を確認する。
- Exception handlerあり/なし、handler自身の例外、callback種別ごとのfailure、unknown native status/errorを確認する。
- UTF-8空/null pointer/embedded NUL/multibyte/buffer-too-small、Error snapshot保持、detail allocation failureをDD-05に照らして確認する。
- `win-x64`/`linux-x64`のNuGet publishでRID別native assetが選択されることはDD-22のpackage testで確認し、DD-19単独の完了条件にdistribution matrixを重複させない。

## 20-wrapper-rust.md / 9. 検証観点

- 移管元: [言語ラッパー：Rust](../13_detail_design/wrappers/rust.md)

- move、明示retain、borrowed lifetimeで、tokenの複製/消費がDD-HND-04と一致する。
- create/retain成功・失敗ごとにticketとtokenが一対一で移り、borrow scopeが終わっても元ownerのtokenが変化しない。
- explicit close failureではtoken/ticket/anchorをownerに保持し、Drop failureでは同じ組が一度だけregistryへ移る。ticketとRuntime anchorの解放順が逆転しない。
- UI-affine ownerが`Send`/`Sync`にならず、post入口・closureだけが要求条件の範囲でthread間移送できる。
- callback登録/postの失敗・成功でuserdata ownerがそれぞれ一度だけ破棄され、destroyはUI threadで一度だけ起きる。
- 明示close failureでownerが存続し、Drop failureで事前確保ticketへtokenが損失なく移る。ticket確保failure時にnative owned tokenを先に発行しない。
- Runtime shutdown、UI queue failure、Closed shell release、最後のRuntime anchorまでcleanup managerが寿命を保つ。
- `panic=unwind`でcallback panicとcallback state destructor panicがC ABI外へ出ないことを確認し、callback owner規則に沿う。`panic=abort`は別buildとしてprocess abortを許容する前提を明示する。
- unknown status/domain/code、Error snapshot有無、Error生成failure、UTF-8 byte列境界をDD-05/06と照合する。
- Windows x64/Linux x64のnative ABI連携はDD-22のtarget/package testで扱い、Rust crate単体の型確認と配布適合を混同しない。

## 21-wrapper-python.md / 10. 検証観点

- 移管元: [言語ラッパー：Python](../13_detail_design/wrappers/python.md)

- 各C ABI functionの`argtypes`/`restype`/calling convention/widthをheaderと機械的に照合し、32-bit/64-bit C type差を誤って混入させない。
- create/retain failure時にticketだけ戻り、成功時はtoken・ticket・Runtime anchorが一つのownerへ対応する。Python reference copyはnative retainしない。
- explicit close、context manager、close failure、繰返しcloseでtoken releaseが高々一回となり、失敗時にownerがopen状態を保つ。
- callable、`CFUNCTYPE` object、userdata rootがforced GC中もCore destroyまで生存する。register失敗とsuccessで強参照releaseが一回だけである。
- callback self-unsubscribe、in-flight destroy、shutdown、post cancel時にCoreとPython root registryの所有者が重複しない。
- object finalizerからtoken/ticketが非blockでmanagerへ移り、UI-affine native cleanupがUI threadで完了する。`__del__`が任意threadで走る条件、finalizerの遅延/失敗、ticket確保失敗を確認する。
- Python main/host thread終了で、`atexit`開始前にもUI threadが残留しないようRuntime lifecycle signalが届き、UI loopがshutdown後に停止する。callback gate、in-flight invocation、root解放とinterpreter teardownの順序を検証し、main/UI相互待ちがないことを確認する。non-daemon Python UI threadがある場合にもprocess終了を妨げないことを確認する。
- Qt `aboutToQuit()`相当通知では、UI threadのloop停止前にcleanupが完了し、後続の通常queue dispatchを待って停止したままにならない。
- main threadの未捕捉例外ではshutdown要求後も既存`sys.excepthook`のtraceback出力が維持され、BD-36/DD-03のthread/context dispatchを使う。workerからUI完了を待つ場合もcallback deadlockを作らず、wrapper所有worker例外もthread例外の報告を保つ。
- host UI loopの終了直前hook、Python main threadの未捕捉一般例外、`SystemExit`、wrapper所有workerの未捕捉例外、callback handler自身の例外を別経路で検証する。main例外hookは既存hookのtraceback報告を保ち、BD-36/DD-03に従ってthread/contextに適したdispatchを選ぶ。捕捉済み例外は自動終了を起こさない。
- abrupt termination (`os._exit()`、Python fatal error、未処理signalによるkill)を通常終了hookの保証範囲と混同しない。
- `ctypes.CDLL`のGIL release中に他worker callを行い、wrapper state race/deadlockがない。foreign thread callbackでthread-localを状態保存に使わない。
- worker変更を許すowner/registry stateを複数threadで競合更新し、lockによりtoken/ticket/callback ownershipと状態遷移が壊れないことを確認する。UI-affine stateはworkerから変更不能で、read-only/snapshot/value copyまたはdispatcher経由になることを確認する。
- free-threaded CPythonをsupport対象に含める場合に限り、同じthread access policyをfree-threaded buildでも検証する。GIL有無でRuntime UI affinityが変わらない。
- methodごとのbase chain-up policyについて、required/optional/not-required/dynamic-unknownを分け、wrapper-controlled overrideのmissing chain-up例外とBD-15 static diagnosticを確認する。`super()`以外の有効ancestor callを誤検出せず、全operatorへ一律のsuper requirementを付けない。
- `__del__`/`weakref.finalize`候補がresource tokenを一度だけmanagerへhandoffし、chain-up不足検知と混同しない。例外を呼出し側へ返せると誤認せず、owner復活やfinalizer遅延を含む条件を確認する。
- Python normal interpreter shutdownで、main threadからの終了要求後にnative UI loopが停止し、Python callbackへ新規entryしない。callback root/in-flight drainがinterpreter teardownより先に完了する規則と、終了時waitによるdeadlock不在を確認する。
- callback内の例外、handler自身の例外、`KeyboardInterrupt`/`SystemExit`候補をABI越境なしに処理する。
- UI-only APIをworkerから直接呼ぶとCore statusとwrapper挙動が一致し、worker updateはUI threadで反映する。
- Errorなし/あり/生成失敗、unknown status/code、UTF-8空/embedded NUL/buffer-too-smallをDD-05/06と照合する。
- BD-17どおりpure-Python wrapperとnative runtimeを別distributionに保つ。wheel、native loader、ARM board profileはDD-22で検証する。

## 22-build-distribution.md / 9. 検証観点

- 移管元: [ビルド・配布：toolchain・profile・package](../13_detail_design/build/toolchain-packages.md)

- 各profileでbuild hostとtargetの値が区別され、Windows/Linux nativeとARM cross-buildがBD-16の順序で判定される。
- ARM link/build成功がruntime/hardware verified状態へ自動昇格しない。
- 4言語packageで同一target native bytes、manifest identity、digest、ABI major、feature条件が一致し、言語ごとの再build/relabelがない。
- 1-byte binary change、version/digest/profile/feature差、同version別digestを検出し、immutable catalog/cache identityの上書きがない。
- JSON parserのduplicate key、invalid UTF-8/Unicode、unknown schema major/critical field、ambiguous signature scope、oversized inputを拒否する。
- known/trusted/current keyでは正規manifest signatureが検証され、unknown/revoked key、署名改変、artifact digest改変、stale trust条件はpolicyに応じfail closedとなる。
- wrapper package assembly後もnative artifact digestが保たれ、各上位 package boundary (wheel/RID/CMake/Cargo) の consumer smokeで正しいtargetを選ぶ。
- release/version testの変更影響としてruntime, wrapper, manifest schema, target profileが別々に追跡できる。
- 承認済みのRFC 8785 JCS・Ed25519・SHA-256について、RFC test vectorとpositive/negative vectorsをpackage verification testへ引き継ぐ。

## runtime-contract.md / 8. この段階での検証条件

- 移管元: [ランタイム：実行context・所有・終了](../12_design/runtime-objects/runtime.md)

1. 4言語から作成、retain、複数回の別参照のrelease、shutdownを同じ意味で実行できる。
2. 解放済み・異種・別Runtimeのhandle、null/length不整合を安全に拒否できる。既に解放した**同じトークン**の二重releaseを検出し、別の所有参照へ影響させない。
3. workerから直接UI操作すると`WRONG_THREAD`となり、postした操作はUI threadで実行される。
4. callbackの自己解除、shutdown時の未実行post、wrapper例外の各経路でuserdataが1回だけ破棄される。
5. buffer不足時に必要容量が得られ、文字列にNUL終端を要求しない。

## abi-spec.md / 5. ABI検証の最小集合

- 移管元: [共通ABI：言語非依存境界と互換性](../12_design/runtime-objects/abi.md)

- CヘッダをCコンパイラでコンパイルし、4言語wrapperからversion、feature、create/retain/release/shutdownを呼ぶ。
- 同じobjectを指す異なる二つの所有トークンを独立にreleaseでき、片方の二重releaseが他方へ影響しないことを確認する。
- 解放済み、種類違い、別Runtimeのhandleを拒否できることを確認する。
- UTF-8メッセージの容量照会とbuffer不足、callbackの登録成功/失敗時の所有権を確認する。
- バイナリ互換性の検証は同じmajorの旧ヘッダでビルドしたwrapperを新しいnative artifactに接続して行う。

## abi-spec.md / 7.5 検証する振る舞い

- 移管元: [共通ABI：言語非依存境界と互換性](../12_design/runtime-objects/abi.md)

1. 4言語から`BOOL`、`I64`、`UTF8`を読み書きし、値の一致、UTF-8の容量照会・再試行・埋め込みNULを確認する。
2. 型不一致、範囲外の整数、不正UTF-8、無効なbool、workerからの呼出しを安全に拒否し、値と通知を変えない。
3. 同一Core objectの異なる所有トークンをOBJECT値として設定しても、同一性に基づいて不要な通知を出さない。OBJECTの公開テストは循環防止規則が確定してから行う。
4. 検証失敗とAPI引数エラーを区別し、失敗時の`out_error`と出力値の初期化を4言語で同じ意味で扱う。

## abi-spec.md / 11.4 検証する振る舞い

- 移管元: [共通ABI：言語非依存境界と互換性](../12_design/runtime-objects/abi.md)

1. OneTime作成成功はBinding handleを返さず、初期同期の値だけが残る。OneWay/TwoWay作成成功は解除可能なhandleを返す。
2. グラフ循環、種類違い、別Runtime、初期Validation失敗では値・グラフ・通知を変えない。OneWay分岐は成功する。
3. 継続中のBindingで検証失敗が起きた場合、入力元は維持され、snapshotで拒否Property・方向・errorを取得できる。成功した再伝播ではエラーが消えて通知が届く。
4. Binding解除、自己解除、Property公開handleの先行release、shutdownで伝播とcallbackの寿命が契約どおり終わり、closureが一度だけ破棄される。

## abi-spec.md / 13.4 検証する振る舞い

- 移管元: [共通ABI：言語非依存境界と互換性](../12_design/runtime-objects/abi.md)

1. 4言語でBOOL / I64 / UTF8のvalidatorを登録し、受理・拒否・言語例外を区別する。UTF8の借用期間と埋め込みNULを確認する。
2. 拒否時には値の通知を出さず、直接setterへは`UI_VALIDATION_FAILED`を返す。Binding伝播時には第11節のsnapshotからdomain/code/messageを取得する。
3. 登録失敗・明示解除・Property先行破棄・shutdownでclosureがちょうど一回`destroy`される。callback中のUI変更は拒否される。

## abi-spec.md / 15.4 検証する振る舞い

- 移管元: [共通ABI：言語非依存境界と互換性](../12_design/runtime-objects/abi.md)

1. 4言語でCommandを起動し、UI callbackが速やかに戻り、実行中の二重起動を拒否する。実行条件と状態の変化がcan-execute Propertyへ反映される。
   同じUIイベントcallback内の連続2回呼出しと、別Controlからの実行中の呼出しも新しいoperationを作らない。終端後の次の呼出しは新しいIDで実行できる。
2. workerはtokenでキャンセル要求を確認し、UIへの進捗・完了は`post`経由だけで反映する。キャンセル要求後の正常終了は`SUCCEEDED`になる。
3. 古いoperation IDと二重報告は現在状態を変更しない。失敗のsnapshotは不変で、次の実行では現在の失敗情報が消える。
4. closeとshutdownはworkerをUI threadで待たず、closureを一回だけ破棄する。worker側の所有handleは終了時に解放できる。

## abi-spec.md / 17.4 検証する振る舞い

- 移管元: [共通ABI：言語非依存境界と互換性](../12_design/runtime-objects/abi.md)

1. 4言語で候補全文と選択範囲を受け、`ACCEPT`だけが確定PropertyとBindingへ伝播し、`INCOMPLETE`では編集値のみ、通常の`REJECT`では値・caretとも旧状態を保つ。
2. IME変換中に候補を破棄せず、確定時の`REJECT`は理由と候補をViewに保持する。プログラムからのsetterはView入力Validatorを経由しない。
3. 登録失敗・明示解除・TextBox破棄・shutdownでclosureが一度だけ破棄され、異常なcallbackはABI境界を越えない。

## abi-spec.md / 20.4 確認する振る舞い

- 移管元: [共通ABI：言語非依存境界と互換性](../12_design/runtime-objects/abi.md)

1. 4言語でTextBoxを作成し、異なる所有トークンから同じ確定値Propertyを取得できる。初期値・編集値・入力状態が一致し、生成で通知は出ない。
2. TwoWayでViewModelと接続し、`ACCEPT`だけが伝播する。モデル側の検証失敗ではTextBoxの値と表示を保ち、後から届く別の有効値では編集値・理由が更新される。
3. 未接続TextBoxをreleaseして破棄した後も、別途保持したPropertyは有効で、Validatorと入力購読のclosureは一度だけ破棄される。

## mvvm-semantics.md / 6. 検証する振る舞い

- 移管元: [オブジェクト管理：値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)

1. UI threadのsetter直後にgetterは新値を返す。workerからのsetterは`WRONG_THREAD`で値を変えず、`post`経由では更新できる。
2. 等しい値の再設定は通知なし。異なる値を同じイベント中に複数回設定しても通知は一件で、購読者は最終値を読む。
3. 二つのPropertyを更新した場合、最初に変更されたPropertyから通知される。同じPropertyの購読者は登録順に呼ばれる。
4. 通知callback中の設定は再帰的な通知を起こさず次の配信回に回る。自己解除後は再度呼ばれない。
5. 通知待ちの状態で`shutdown`した場合、callbackは開始されず、closureは一度だけ破棄される。

## mvvm-semantics.md / 8.4 検証する振る舞い

- 移管元: [オブジェクト管理：値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)

1. `OneTime`は作成時の一度だけ同期し、後続の変更には反応しない。
2. `OneWay`は初期同期後、sourceの最終値のみをtargetへ反映し、targetの直接変更をsourceへ戻さない。
3. `TwoWay`は初期同期でsourceを優先し、片側の直接変更をもう一方へ伝える。同じ処理区間で両側が直接変更された場合は最後の直接設定を優先し、反射による無限往復を起こさない。
4. 外部購読者がsource変更を受け取る時点で、同じ配信回のBinding伝播は完了している。伝播されたtargetの変更通知は次の配信回に届く。
5. 解除後の変更は伝播せず、両Propertyの値は保持される。型不一致の作成失敗では値も購読も変えない。

## mvvm-semantics.md / 10.4 検証する振る舞い

- 移管元: [オブジェクト管理：値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)

1. `execute`はUI callbackを長時間占有せず、実行中の二重起動を拒否する。
2. workerから直接UI Propertyを設定すると`WRONG_THREAD`になり、`post`経由の進捗はUI threadで反映される。
3. キャンセル要求後でもworkerが正常完了した場合は`Succeeded`となり、キャンセル確認済みの場合だけ`Cancelled`となる。
4. 過去のoperation IDを持つ遅延通知や、完了後の進捗は現在の状態を変更しない。
5. 画面を閉じてもUI threadはworkerを待たず、後着の結果が閉じた画面を更新しない。shutdown時に未実行postのclosureは一度だけ破棄される。

## mvvm-semantics.md / 12.4 検証する振る舞い

- 移管元: [オブジェクト管理：値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)

1. TwoWayの二辺以上のグループでは、同じ処理区間の最後の直接設定が全メンバーへ伝播する。
2. OneWayの分岐は許し、同じtargetグループへの二つ目の入力および有向循環は作成時に拒否する。失敗後の値とグラフは元のままである。
3. 入力先を直接変更しても入力元へ戻らず、その後の入力元変更で上書きされる。同じ処理区間に両端を変更した場合はOneWayの入力元が勝つ。
4. Bindingを解除した後の保留中の更新はその接続へ流れない。通知callbackで作った新しい変更は再帰配信されない。

## mvvm-semantics.md / 14.4 検証する振る舞い

- 移管元: [オブジェクト管理：値・モデル・更新整合性](../12_design/runtime-objects/model-values.md)

1. 直接のsetterが無効値を拒否した場合、Propertyの値と値の通知は変わらず、検証失敗が呼び出し元へ返る。
2. TwoWayで入力側は受け入れ、モデル側が拒否した場合、入力値を保持し、モデル値を維持し、Bindingエラーを通知する。次の有効入力で値が再同期してエラーが消える。
3. 入力が無効な間にモデル側を直接更新すると、通常の伝播で入力側が更新される。Binding作成時の検証失敗は既存の値・グラフ・エラー状態を変えない。
4. 検証callback自体の異常は通常の無効入力と区別され、ABI境界を越えない。解除後にはそのBindingのエラー通知が新しく始まらない。
5. 整数欄の途中の負号や時刻欄の`12:`はViewに残り、ViewModelへは流れない。禁止文字の貼り付けは編集前の値と選択状態を保持する。
6. IME変換中の文字列は候補として破棄されず、確定後に無効と分かった入力は理由を示してView側に保持する。プログラムからのsetterはView入力Validatorを経由せず、ViewModel側の検証は経由する。

## language-bindings.md / 6. 検証する振る舞い

- 移管元: [言語ラッパー：共通facade・所有変換・解析](../12_design/wrappers/language-wrappers.md)

1. C++、C#、Rust、Python 3で同じ初期値、更新順、通知、Validation結果を観測できる。
2. 言語側ViewModelの明示adapterからCore Propertyへ値を送り、UI Bindingを更新できる。既存クラスの通知に依存しない経路を確認する。
3. workerからCore Propertyを直接読む・書くと`WRONG_THREAD`になり、dispatcher経由の値の反映はUI thread上で行われる。
4. 自己解除、画面終了、shutdown、post失敗のそれぞれで、言語側callback参照を一度だけ解放できる。


## 承認履歴（今回）

| 日付 | 対象版・範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 1.0 / 試験文の所管移管 | 利用者 | 全案反映を承認。本文に未設計と記した項目の実装方式・実装完了は含まない |

## 改訂履歴（今回）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 1.0 | 2026-10-17 | 要求正本への対応、層に応じた責務・flow・所有・stateの本文へ改訂 | Codex |

## DD-06 互換性の確認ケース（移管）

- 移管元: [共通C ABI：宣言・検査・互換性](../13_detail_design/runtime-objects/abi.md)
- case: [TC-ABI-COMP-01/02](requirement-test-matrix.md#case-tc-abi-comp-01)

互換性の確認ケース:

1. ABI major 1向け旧headerで構築したC clientを、同majorの新minor/new patch runtimeへ接続する。
2. 新minor専用symbolを必要とするwrapperは、古いruntimeでsymbol解決前に必要minimum version/featureを確認し、未対応を明示する。
3. 新runtimeが返す未知status・未知error domain/code・未知featureを古いwrapperが未定義動作なく保持またはgeneric failureとして扱う。未知featureはfalse。
4. `get_version`のmajorとsymbol prefixが一致し、minor追加とpatch修正の説明がBD-28に合う。
5. 4言語wrapperで同じstatus分類、Error snapshot、`UI_PENDING`の受付済み意味を得る。
6. UI loop bridgeのnormal/control wakeはUI threadへ非同期scheduleされ、control wakeが実行中eventを中断しない。post成功後のclosure所有権はDD-03/04に従う。
