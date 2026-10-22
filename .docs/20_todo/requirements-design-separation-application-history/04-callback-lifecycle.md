# 適用前記録 / 04-callback-lifecycle.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [04-callback-lifecycle.md](../../13_detail_design/04-callback-lifecycle.md)

---

# DD-04 Callbackと再入

- 状態: 内容承認済み（2026-10-07）
- 作成日: 2026-10-05
- 対象: callback closureの所有権、登録解除、実行中callbackの寿命、destroy、再入、言語例外の封じ込め、イベントメソッドで事前検知する内容と識別メタデータ案
- 上位出典: `.docs/12_design/runtime-contract.md` R3～R4、`.docs/12_design/abi-spec.md` A9～A12・A18～A20・A28、`.docs/12_design/mvvm-semantics.md` P3～P4、`.docs/12_design/language-bindings.md` L4、`.docs/12_design/BD-03_runtime-notification-upper-design.md` BD-03
- 先行詳細設計: `.docs/13_detail_design/01-handle-registry.md` DD-HND-01～06、`.docs/13_detail_design/02-runtime-lifetime.md` DD-LIFE-01～08、`.docs/13_detail_design/03-dispatcher.md` DD-DSP-01～08（いずれも承認済み）
- 後続: DD-06 ABIシンボル、DD-07 Property、DD-09 Validation、DD-10 非同期Command、DD-14 TextBoxとIME、DD-18～21 各言語wrapper、追加候補DD-24 イベントメソッド制約と事前検知

本書のDD-CB-01～09は利用者承認済み（2026-10-07）。既承認のR3/R4、A/P/L判断、DD-01～03を変更しない。callback種別ごとの意味論は所有する上位判断・後続設計に従う。イベントメソッド制約の詳細ルールと各言語toolingは追加候補DD-24へ送る。C宣言や擬似コードは配布用ヘッダ、実装済みAPIを意味しない。

## 1. 範囲と前提

- `Callback closure`は関数ポインタ、`userdata`、`destroy`関数の組を指す。成功した登録で所有権をCoreへ移し、失敗時は呼出し側に残す（R3、A10～A12）。
- Coreが所有するclosureの`destroy(userdata)`はUI thread上でちょうど一度実行し、実行中callbackが戻る前にclosureを破棄しない（R3）。
- Runtime内のcallback実行はUI threadに集約し、同一Runtime内でcallbackを並列または再帰実行しない（R2/R4、DD-DSP-03）。callback内から許されるUI操作はcallback種別ごとの規則に従う。
- Property通知の非再帰配信、登録順、解除・追加時の対象範囲はP3/P4に従う。具体的な通知評価順とBinding動作はDD-07/08が所有する。
- `destroy`からUI APIを呼ばない。各wrapperはclosureが有効な期間に言語側callback参照を保持し、`destroy`で解放する（R3、L4）。

## 2. 判断案

| ID | 提案 | 根拠・境界 |
|---|---|---|
| DD-CB-01 | callback登録・postが成功した時点で、関数ポインタ・userdata・destroyを含むclosureの所有権をCoreへ移す。失敗時は所有権を移さず、Coreはcallbackもdestroyも呼ばない。 | R3、ABI 2.2、DD-DSP-02。各登録APIの成功条件や出力handleは所有者の詳細設計に従う。 |
| DD-CB-02 | Core所有closureは`Active → Disabled → Destroying → Destroyed`の一方向状態を持つ。所有者の解除・破棄・shutdownでDisabledにし、その後は新しいcallback invocationを開始しない。既に開始したcallbackは完了させる。 | R3/R4、P4、DD-LIFE-04/05/07、DD-DSP-08を共通の寿命規則にする。公開handleの有効性・releaseの効果はDD-01/02に従う。 |
| DD-CB-03 | 購読callbackから自身を解除する操作は許可する。解除はclosureをその場で破棄せず、現在のcallbackを完了させてからdestroyを一度だけ実行する。解除後に同じ配信回でまだ開始していないcallbackは、実行開始前にActive状態を確認し、無効なら呼ばない。 | R3/R4、A11～A12、P4。自己解除を許さないValidatorやTextBox入力callbackには個別の承認済み制約を適用する。 |
| DD-CB-04 | Property通知callback内のProperty更新は許可するが、そこから発生する通知callbackを現在のcallbackへ再帰的に呼び出さず、次の配信回へ送る。同じPropertyの購読者はP4の登録順で呼び、配信中の解除・追加はP4の対象範囲に従う。 | P3/P4。通知の重複排除、Binding伝播順、有限作業枠とyieldはBD-03およびDD-07/08が所有する。 |
| DD-CB-05 | callbackの再入可否はcallback種別別に適用し、R4を一律の再入許可と解釈しない。通常のpost/event callbackは許されたUI APIを呼べるが、Validatorは同一RuntimeのUI状態変更や自己解除を行えず、TextBox入力callbackもUI状態変更・自己解除を行えない。Command start callback固有の禁止事項も維持する。 | R4、A19～A20、A28、およびABI §15のCommand規則。各callback種別固有の戻り値・状態更新・エラー意味論はDD-07/09/10/14へ送る。 |
| DD-CB-06 | 言語wrapperはcallbackからの言語例外をC ABIへ伝播させない。通知callbackの例外は診断経路へ記録して、そのcallbackの後始末を行い、他の通知対象の配信を続ける。戻り値を持つcallbackは各callback種別で定めた失敗値へ変換する。Rust panicも公開ABI境界を越えない。 | R3/R4、ABI §4・§9・§13・§15・§17、P3～P4。status、error object、診断payloadの具体形は後続設計で定める。 |
| DD-CB-07 | closureをDisabledにした後、in-flight invocationがなくなったUI-thread safe pointでdestroyを一度だけ実行する。destroyの呼出し中は同じRuntimeのUI APIを禁止し、wrapperは言語側参照の解放に限定する。 | R3、L4、DD-LIFE-04/07/08。cleanupのSubsystem間順序はDD-02/07/10/14/16の所有規則に従う。 |
| DD-CB-08 | callback/event entry pointを識別する言語別markerを設け、診断ツールがそのメソッドと到達可能な処理を調べる案を検討する。共通化する場合も識別対象や診断分類の範囲にとどめ、解析・計測の実装能力は言語ごとに異なってよい。 | BD-03のevent callbackに対する静的警告を拡張する候補。検知ルール、構文、全言語で必須にするかは未決。 |
| DD-CB-09 | イベントメソッド内の検知候補を「仕様上禁止する違反」「誤動作リスクの警告」「追加調査が必要な候補」に分類し、ルール一覧と各言語での診断精度を後続設計で定める。 | callback型ごとの制約はA19～A20、A28、ABI §15等に分散し、応答性・循環リスクまで含めると範囲が広い。DD-04では候補の棚卸しと責務分離までを提案する。 |

## 3. Closure状態遷移案

| 現状態・事象 | 次状態 | callback / destroy / 所有権 |
|---|---|---|
| CallerOwnedで登録・postが失敗 | CallerOwned | Coreはcallback・destroyを呼ばない。呼出し側が後始末する |
| CallerOwnedで登録・postが成功 | Active | Coreがclosureの所有権を引き受ける |
| Activeで解除・owner破棄・shutdown | Disabled | 新規invocationを開始しない。保留中の通知・workは所有元の規則に従って破棄する |
| Disabledでcallbackが実行中 | Disabled | 実行中callbackはreturnまで完了する。自己解除は待ち合わせしない |
| Disabledかつin-flight invocationなし | Destroying → Destroyed | UI thread上でdestroyをちょうど一度呼ぶ。以後closureを参照しない |
| Destroyedのhandleを別途release | Destroyed | 公開handleの消費だけを行う。destroyを再実行しない |

### 実行中に解除された場合

購読callbackが自分自身を解除すると、解除時点でSubscriptionをDisabledにする。解除APIは現在のcallback終了を待たずに戻る。Coreはcallbackから戻った後にin-flight状態を解き、同じ配信回に残る対象のうち既にDisabledとなったものは呼ばず、対象closureをUI thread上で破棄する。公開Subscription handleは別途releaseされるまで無効shellとして残す（A11、DD-LIFE-04）。

## 4. callback種別ごとの再入境界

| callback種別 | callback内で許されること | 禁止・留保すること |
|---|---|---|
| `runtime_post` / 通常UI event | R4とDD-03の範囲でUI APIを呼べる。postは同じqueueへ入り、即時・再帰実行しない | shutdownはBD-36とDD-03の優先control taskに従い、現在event終了後に実行。shutdownの戻りstatus詳細はDD-06等へ送る |
| Property / Binding通知 | snapshot/getter等、通知契約で許されたUI APIを呼べる。Property更新は次の配信回へ送る | callback実行中の再帰通知、P3/P4を超える通知順の追加決定 |
| Property Validator | 候補計算、許された読取専用参照、validation error生成 | 同一RuntimeのUI状態変更、登録解除、shutdown、自己解除（A19～A20） |
| Command start | Command start契約内でworkerを開始し、速やかに戻る | 同じCommandの再実行・close、callback内の完了報告（ABI §15、DD-10へ） |
| TextBox input Validator | 候補の判定、許可済み理由の生成 | 同一RuntimeのUI状態変更と自己解除（A28、DD-14へ） |

この表は承認済みの種別別制約をまとめる提案である。個別callbackの成功・拒否、operation状態、診断状態の意味は各所有設計を変更しない。

## 5. イベントメソッドで事前検知する候補

「イベントメソッド」は、Coreから呼ばれるevent handler/callbackのentry pointを指す。検知器は全候補をエラーとして扱わず、設計上禁止される操作と、許可されるが危険を知らせる警告を分ける。

| 分類 | 検知候補 | 根拠・扱い |
|---|---|---|
| 禁止違反として検出 | Validator callbackから同一RuntimeのUI状態を変更する、登録を解除する、shutdownする | A19～A20で禁止。直接呼出しで確認できる違反はerror候補。間接呼出しを証明できない場合はunknownとして扱う |
| 禁止違反として検出 | TextBox入力callbackからUI状態を変更する、または自身を解除する | A28で禁止。callback型が分かる場合のerror候補 |
| 禁止違反として検出 | `destroy`からUI APIを呼ぶ | R3。直接検出できる呼出しをerror候補とする |
| 禁止違反として検出 | Command start callbackから同じCommandを再実行・closeする、callback内で完了報告する | ABI §15のcallback契約。DD-10で検査ルールを具体化する |
| 強い警告 | event元と同じControlのPropertyへの書込み、特にイベントによって変化した同一Propertyへ書き戻す処理 | BD-03の静的警告方針。許可/禁止の断定ではなく、通知反復の可能性として示す |
| 警告 | event method内の長い同期処理、sleep、blocking wait/join、同期I/O、長い反復 | UI threadを占有・停止させる可能性。ただし現行上位設計に一律禁止規則はないためwarning候補であり、しきい値・検出範囲は未決 |
| 情報 | 解析器が解決できないdynamic dispatch、関数ポインタ、動的callableを通る経路 | 禁止違反を証明できないことを表示し、未知経路だけを理由にerrorとしない案 |
| 調査候補 | callbackから発生する間接的なevent/notification循環、callback呼出しグラフの再帰 | 静的解析だけで閉じたグラフを証明できない場合がある。BD-03のRuntime非収束検知は引き続き必要 |
| 調査候補 | closureがUI ownerを強参照captureすることによる参照循環・長期保持 | 言語ごとにcapture/GC/所有規則が異なる。必要性と誤検知リスクをwrapper設計で評価する |
| 調査候補 | 非同期関数・coroutineをevent methodへ直接登録すること | 戻り値、実行context、例外・キャンセルの意味が言語ごとに異なる。現時点で一律の禁止/対応は決めない |

この候補表は調査範囲を示し、個々の未決候補を禁止規則に昇格させるものではない。エラーにする規則には承認済み根拠または新しい設計判断を付け、警告には危険の程度とfalse positiveの扱いを定める必要がある。特に長時間処理・blocking・captureの検査は言語ごとの構文・解析能力が異なる。

## 6. callback対象メソッドの識別案

利用者は各言語でcallbackに設定するメソッドを検知するため、次のような言語機能を候補として挙げた。

| 言語 | 候補 | DD-04での位置づけ |
|---|---|---|
| C++ | `[[...]]`または処理系attribute | event/callback entry pointと、禁止操作を解析器が識別できるかを検討する |
| C# | attribute（`[ ... ]`） | method metadataを解析・診断へ利用できるかを検討する |
| Rust | proc-macro | annotated itemを検証または解析用情報へ展開できるかを検討する |
| Python 3 | decorator。必要ならJITまたはwrapperでentry/exit計測を埋め込む案も候補 | callable markerの静的解析と、実行時計測のどちらが目的を満たすかを検討する |

この案では、再帰を検知したい範囲と計測位置を分けて扱う。

1. **静的解析・診断**: callback entry pointを明示し、禁止API呼出し、BD-03のProperty access、blocking処理候補などを調べる。markerを付けなくても既存コードが動作し、検査精度向上のための任意情報として扱う。
2. **callback境界の実行時計測**: wrapperが生成するcallback thunkのentry/exitでcallback IDや深さを計測する。これはCoreからの再呼出しや同じentry pointへの再帰的なdispatchを検知する補助になり得るが、callback method内部の通常のメソッド呼出し全体は見えない。Coreへ伝える方法、callback IDの寿命、ABI公開面の要否はDD-06で検討する。
3. **メソッド本体の実行時instrumentation**: annotated methodのbody自体にentry/exit処理を埋め込む方法。直接メソッド呼出しも追える可能性がある一方、言語ごとに実現方式が大きく異なるため、DD-04では共通の必須動作にしない。

したがって、marker単独で再帰を抑止・検知できるとは扱わない。BD-03のRuntime非収束通知検知はmarkerの有無に依存させず、marker利用による追加の静的/実行時診断を各言語toolingで検証する案とする。

属性の具体構文、対象とするcallback種別、annotationを必須にするか、動的なlambda/closureや既存メソッドをどう扱うか、entry point計測とmethod-body instrumentationのどこまで必要かは未決である。C++ compiler attribute、C# attribute、Rust proc-macro、Python decorator/JITはそれぞれ異なるため、検知対象の分類は共有しつつ、各言語で実現可能な解析・計測の範囲を後続のwrapper/tooling設計で定める。

## 7. 例外・destroyと所有権

- callbackを呼ぶwrapper thunkは、言語側の例外を捕捉してcallback種別ごとの規則に変換する。通知callbackの場合は診断記録後、他の対象購読者への配信を続ける。値の確定やcallback以前のアプリ副作用を巻き戻さない。
- Validator等の戻り値を持つcallbackは、例外をそのAPIの承認済み失敗表現へ変換する。候補値の確定可否や出力値の初期化は該当するABI契約に従う。
- Rust callback実装のpanicはABI境界で捕捉する。Coreの継続安全性が損なわれた場合にRuntimeを停止状態にするかは、既存のpanic方針およびRuntime設計に従い、ここでは追加しない。
- `destroy(userdata)`はcallback所有権を解放する一度限りの後始末である。destroy thunkもwrapperが言語例外をABI外へ出さない。destroyからUI APIを呼ばない。
- destroyをどのUI-thread safe pointで行うかは、現在のcallback終了後かつ所有元の解除処理と矛盾しない位置とする。独立したSubscription間のdestroy全順序はこの文書で保証しない。

## 8. 検証ケース

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

## 9. 未決境界と承認記録

DD-CB-01～09は2026-10-07に利用者承認済み。承認範囲には、イベントメソッド内で事前検知する内容の候補分類と、言語markerを任意メタデータとして静的解析・wrapper計測を区別する方針を含む。

各言語wrapperの具体的な例外/Result表現、診断payload、callback/destroy thunkのC ABI実装はDD-06およびDD-18～21で定める。Property購読とValidationの個別処理順はDD-07/09、Commandのoperation lifecycleはDD-10、TextBox input callbackはDD-14で補う。検知ルール全体と各言語のparser/analyzer/tool integrationは範囲が広いため、DD-04では候補整理までとした。DD-24「イベントメソッド制約と事前検知」は計画上の追加候補であり、未着手・未承認である。未決事項を確定仕様や実装指示として扱わない。
