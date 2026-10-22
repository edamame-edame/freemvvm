# 適用前記録 / 03-dispatcher.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [03-dispatcher.md](../../13_detail_design/03-dispatcher.md)

---

# DD-03 UI threadと投稿キュー

- 状態: 内容承認済み（2026-10-05）
- 作成日: 2026-09-29
- 対象: UI thread識別、`runtime_post`受付・実行、queue wake-up、host event loopとの接続、通知処理との公平性
- 上位出典: `.docs/12_design/runtime-contract.md` R2～R4、`.docs/12_design/abi-spec.md` A1～A32、`.docs/12_design/mvvm-semantics.md` P1～P4・B1～B4・C1～C4、`.docs/12_design/BD-03_runtime-notification-upper-design.md`、`.docs/12_design/BD-26_async-command-progress-upper-design.md`
- 先行詳細設計: `.docs/13_detail_design/01-handle-registry.md` DD-HND-01～06、`.docs/13_detail_design/02-runtime-lifetime.md` DD-LIFE-01～08（承認済み）
- 後続: DD-04 Callbackと再入、DD-06 ABIシンボル、DD-10 非同期Command、DD-16 描画とPlatform adapter

本書のDD-DSP-01～08は2026-10-05に利用者承認済みである。R2～R4、P1～P4、BD-03/26、DD-01/02の承認済み契約を変更しない。未決境界は§7に記し、後続設計へ送る。C宣言や擬似コードは配布用ヘッダ、実装済みAPIを意味しない。

## 1. 範囲と前提

- Runtime作成時の呼出しthreadがUI threadであり、UI状態の参照・変更とcallback実行はそのthreadに限定する（R2/R3）。利用者確認済み方針（BD-36-01/02/04）として、shutdown要求はcallback/event中も許可し、現在の処理完了後に通常workより優先して実行する。
- `runtime_post`は任意threadから受け付け、同一RuntimeのUI threadで順に実行する。UI threadからのpostも即時実行しない（R2）。
- 通知・Binding処理はBD-03の有限作業枠に従う。post callbackや通常のUI eventと通知を際限なく連続処理してhost event loopを占有しない。
- DD-02はshutdownを`Running → Closing → Closed`とし、shutdown後のUI操作を`UI_CLOSED`とした。callback/event中のshutdown要求は現在の処理を完了してから優先実行する。非同期呼出しは完了を待たず、同期worker呼出しはUI thread上の完了を待つ。ただしcallback/event中は同期指定でも要求受付後に戻る。statusと完了観測の詳細は未決のため、本書ではAPI表現まで確定しない。
- ABI案には`runtime_is_ui_thread`と`runtime_post`があるが、host event loopへのwakeup/pump接続APIは定義されていない。ここでは抽象接続契約を提案し、具体的なABI関数・Platform別setupはDD-06/16へ送る。

## 2. 判断案

| ID | 提案 | 根拠・境界 |
|---|---|---|
| DD-DSP-01 | Runtime作成threadのidentityをRuntimeに記録し、以後のUI thread判定に使う。`runtime_is_ui_thread`は有効なRuntime handleがあれば任意threadから呼べ、呼出し元が異なれば`false`を返す。UI専用APIはhandle/token/Runtime検証（DD-HND-03）の後、RuntimeがClosedなら`UI_CLOSED`、Running中にUI thread以外から呼ばれたら`UI_WRONG_THREAD`を返す。 | R2を具体化する。Closed後のthread identity queryは状態変更をしない照会としてUI操作から区別する案。status優先順位はDD-HND-03のhandle検証を維持する。 |
| DD-DSP-02 | 各RuntimeにFIFOのpost queueを1つ設ける。複数threadからの同時postはqueueへの追加が直列化された順を受付順とする。queue追加が成功した時点でcallback/userdata/destroyの所有権をCoreへ移し、それ以前の失敗では呼出し側に残す。 | R2、ABI 2.2、DD-LIFE-06と整合する。並行呼出しの順は呼出し開始時刻ではなくqueue追加の順で決まる。 |
| DD-DSP-03 | 同じRuntimeの投稿callbackはUI thread上で一度に一つだけ実行し、再帰実行しない。UI thread上のpostも通常queueの末尾へ追加する。host event loopの一回のdispatch turnでは投稿callbackを最大一件実行し、そのcallback終了後に次turnをscheduleする。shutdown要求がpendingの場合は通常callbackよりshutdown制御処理を先に実行する。 | R2/R4の直列・非再帰実行を具体化し、hostの入力・描画eventへ制御を返す案。callback自身が長時間戻らない場合の応答性は保証しない。 |
| DD-DSP-04 | 外側のUI API、host event、またはpost callbackが戻った後、pending Property通知とBinding伝播をBD-03の規則で処理し、通常は次のpost callbackやnative UI eventより先に進める。通知の有限作業枠に達した場合は完了した配信回の境界でhost loopへ戻り、未配信通知を保持して後続turnで再開する。 | P3、BD-03の通常順序とyield例外を反映する。通知の詳細な評価順はDD-07/08が所有し、この文書では変更しない。 |
| DD-DSP-05 | post queue、pending notification、finalizer、shutdown control taskのいずれかに処理対象があるときRuntime wake-upを一つだけpendingにできるようにする。通常work用とshutdown優先work用の状態を区別し、shutdown要求があれば現在のdispatchが戻った後の次turnに優先taskをscheduleする。drain終了時のqueue確認とwake状態更新はenqueueと同期し、確認と競合して新しい仕事が入っても次turnが必ずscheduleされる。 | 余分なwake-upを抑えつつ、worker postやshutdown requestを取りこぼさない案。shutdown優先は実行中callbackを中断せず、未開始の通常workより前に処理する。 |
| DD-DSP-06 | CoreはOS/hostのevent loopを所有・実行せず、Runtimeに紐づく`UiLoopBridge`を介してUI threadへ非同期wakeを依頼する。bridgeは通常workのwakeとshutdown優先taskのwakeを区別でき、後者を現在のevent終了後の次処理機会へscheduleする。wakeは任意threadから安全に呼べ、呼出し中にRuntime/user callbackやcleanupを実行しない。 | 設計骨格のPlatform adapter分離とR2を反映する抽象契約。現ABIに接続hookがないため、bridgeの生成・登録方法とC ABIの追加有無はDD-06/16で決める。bridgeを用意できないhost loopでのRuntime作成/postの扱いは本案では未確定。 |
| DD-DSP-07 | post queueに固定件数上限は設けず、資源確保に失敗したpostは`UI_OUT_OF_MEMORY`で拒否する。拒否時はcallback等の所有権を移さず、受理済みqueueを変えない。BD-26の中間進捗は別途最新値へ集約できるため、全progress sampleを個別postしない。 | BD-26はqueue容量・progress集約場所を未確定としている。上限やbackpressure statusを導入せず、ABI既存statusで割当失敗を扱う提案。 |
| DD-DSP-08 | shutdown要求を通常postとは別の優先control taskとして扱う。callback/event中に要求された場合、現在実行中の処理は完了させ、その直後のdispatch機会でcleanupを開始する。非同期呼出しは要求受付後に戻り、完了を待たない。同期呼出しもcallback/event中はdeadlock防止のため受付後に戻る。workerからの同期呼出しはUI threadへdispatchし、cleanup完了まで待つ。cleanup開始時にRuntimeをClosingへ遷移させ、それ以前に受理された通常postのうち未開始のものはcancelし、callbackを実行せずdestroyをUI threadで一度だけ行う。Closing後のpostは`UI_CLOSED`で拒否し、closureの所有権を呼出し側に残す。 | BD-36-01/02/04で確認済みの方針をdispatcherへ反映する。優先制御は現在のeventを中断せず通常queueの待ち行列よりshutdownを先行させる。同期/非同期のABI・wrapper表現、statusと完了観測方法はDD-06等で整合させる。 |

## 3. Queueとwake-upの状態案

| 条件 | Runtimeの扱い |
|---|---|
| Runningかつqueueと通知に処理対象なし | wake要求なし |
| worker/UI threadから最初のpostを受理 | FIFO末尾へ追加し、既にwake pendingでなければbridgeへ一度だけ非同期wakeを依頼 |
| wake pending中に追加post | FIFO末尾へ追加し、wakeを重ねて依頼しない |
| UI threadがdispatch turnを開始しshutdown control taskがpending | shutdown control taskを通常workに優先して実行。既に実行中のevent/callbackは中断しない |
| UI threadがdispatch turnを開始しshutdown control taskなし | 投稿callbackを最大一件実行し、その後通知/Binding処理をBD-03に従って進める |
| dispatch後も仕事が残る | queue/通知を保持し、次turnをschedule。継続して同一turn内を占有しない |
| 仕事がなくなった | enqueueと同期した状態確認のもとwake pendingを解除 |
| shutdown request受理 | 優先control taskをpendingにし、現在のcallback/event終了後の次処理機会をschedule |
| Closing/Closed | 新しいpostを拒否。Close規則はDD-DSP-08とDD-LIFE-07に従う |

UI threadからpostされたcallbackもqueueに入るため、現在のcallback/API stackから再帰的に呼び出されない。FIFO順序は同一Runtime内の投稿callback間で保証し、別Runtimeやhost native eventとの全順序は保証しない。

## 4. Host event loopとの接続境界

本案のbridgeは、(1) workerから安全に呼べる非同期wake、(2) UI thread上でのdispatch turn実行、(3) UI thread外からuser callbackを実行しない、の三条件を持つ。Coreはhost loopをpollingしたり、別threadからUI callbackを直接実行したりしない。

ABI仕様の`runtime_create`にはbridge引数がなく、`runtime_post`は既に承認済みである。そのため本書は公開関数signatureを追加しない。DD-06/16では、hostがbridgeをRuntimeへ設定する時点・所有者・未設定時の失敗statusを決定し、4言語から同じ契約を利用できる具体的な接続面を示す。

## 5. 失敗と所有権

| 状況 | status案 | callback/userdata/destroyの所有者 |
|---|---|---|
| callback/destroyがnull、必須Runtime handleが無効 | `UI_INVALID_ARGUMENT` / DD-HND-03に従うhandle status | 呼出し側 |
| RuntimeがClosing/Closed | `UI_CLOSED` | 呼出し側 |
| post queue用資源を確保できない | `UI_OUT_OF_MEMORY` | 呼出し側 |
| queue追加成功 | `UI_OK` | Core。実行完了またはshutdown cancel時にUI threadでdestroyを一度呼ぶ |
| callback内の言語例外 | ABI外へ伝播しない | wrapperが捕捉し、通常のdestroyを一度行う（R4） |

bridge wakeの失敗は受理後に表面化させない契約を提案する。bridgeがwakeを保証できない場合は、所有権移転前にpostを拒否できる接続仕様が必要であり、具体statusはDD-06/16で整合させる。

## 6. 検証ケース

- Runtime作成thread上の操作、別threadからの`runtime_is_ui_thread`、UI専用APIの`UI_WRONG_THREAD`を確認する。
- workerから連続・同時postし、各callbackがUI threadで一度ずつ、同一Runtimeのqueue追加順に実行されることを確認する。
- UI callback内からpostし、現在のcallbackが戻るまで新callbackが始まらないことを確認する。
- 多数のpost callbackがある間も一turn一件の上限によりnative event loopへ制御が戻り、callback後の通知がBD-03の作業枠どおりに処理されることを確認する。
- wake pending中のpost追加、drain終了と同時のworker post、空queueへの最初のpostでwakeの重複・取りこぼしがないことを確認する。
- OOM、Closed、null callbackのpost失敗でclosure所有権がcallerに残り、成功後の実行/cancelでdestroyがUI thread上に一度だけ起きることを確認する。
- Closingとworker postを競合させ、同期点より前のpostはcancel、後のpostは`UI_CLOSED`となることを確認する。
- Runtimeが閉じた後のRuntime APIと、解放可能な残存handleの扱いをDD-LIFE-08に合わせて確認する。

## 7. 未決境界と承認記録

BD-36-01/02/04で確認済みのとおり、callback/event中のshutdown要求は現在の処理完了後に優先実行する。非同期呼出しは完了を待たず、同期worker呼出しは完了を待つ。ただしcallback/event中の同期呼出しは待機できないため受付後に戻る。同期worker待機中にUI loopが停止した場合の扱い、statusの意味、完了観測方法は未決でありDD-06/16とwrapper設計へ送る。

利用者は2026-10-05に本書の内容を承認した。DD-DSP-01～08を承認済みとして記録する。承認対象にはUI thread判定、FIFO post queueと所有権移転、callbackの直列・非再帰実行、一turnあたりの実行上限、通知作業枠後のyield、wake-up coalescing、抽象`UiLoopBridge`、queue容量とOOM時の扱い、shutdown優先control taskと停止時のcancelを含む。

未決事項は、同期worker待機中にUI loopが停止した場合の扱い、statusで受付済みと完了済みを表す方法、完了観測方法である。これらはDD-06/16および各wrapper設計で具体化する。本承認はDD-03に限り、DD-04以降の着手指示や実装承認を含まない。
