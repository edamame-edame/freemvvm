# 適用前記録 / BD-36_runtime-shutdown-dispatch-upper-design.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [BD-36_runtime-shutdown-dispatch-upper-design.md](../../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md)

---

# Runtime shutdownのcallback中dispatchに関する上位判断

- 状態: 方針確定（利用者指示・API別の表現と完了観測は未決）
- 作成日: 2026-09-29
- 対象: UI callback/event中のshutdown要求、UI event queueへのdispatch、caller threadからの実行依頼
- 根拠: `.docs/12_design/runtime-contract.md` R2～R4、`.docs/13_detail_design/02-runtime-lifetime.md` DD-LIFE-07

## 1. 利用者が示した方針

利用者は2026-09-29に、callback中のshutdownを許容し、現在のUI eventを途中で打ち切らないため、event終了後すぐ次に処理できるよう優先度を付けてshutdown処理を積みたいと示した。現在のeventが完了してからshutdownを実行することは許容される。呼出しthreadによる切替はlibrary内で行い、shutdown本体は最終的にUI thread上で同期実行する方針である。さらに、非同期呼出しではshutdown完了を待たず、要求受付後に戻ると明示された。

## 2. 判断案

| ID | 判断案 | 状態・補足 |
|---|---|---|
| BD-36-01 | callback/event中のshutdown要求を許可する。呼出し中のevent/callbackは最後まで実行し、cleanupはその処理から再帰実行せず、現在のevent終了後、通常のqueued workより優先して次の処理機会に実行する。 | 利用者確認済み（2026-09-29）。現在のeventを途中で打ち切らず、通常queueの後方で長く待たせない。 |
| BD-36-02 | shutdown cleanup本体はUI thread上で同期実行する。workerからの要求はUI loopへdispatchして実行する。thread切替とdispatchはlibrary内部で行う。 | 利用者確認済み（2026-09-29）。Runtime作成threadをUI threadとするR2を維持する。 |
| BD-36-03 | callback/event中かを判定するにはthread IDだけでなく、libraryが管理するdispatch/callback実行中のcontextも用いる。UI thread IDはworkerからの要求のmarshal判定に使い、現在のeventを完了させるべき状態はdispatch contextで判定する。 | 実装上の提案・未確認。host event loopが管理する利用者event内まで判定できるかはadapter契約で確認する。 |
| BD-36-04 | 呼出しモードにより戻り方を分ける。非同期呼出しはthread/contextによらず要求受付後に戻り、cleanup完了を待たない。同期呼出しはUI thread上の通常contextならその場で完了し、worker threadならUI threadへdispatchしたcleanupの完了まで待つ。ただしcallback/event実行中は現在の処理を完了させる必要があるため、同期指定であっても待たずに要求受付後に戻る。 | 利用者確認済み（2026-09-29）。callback中の戻りstatusが受付済み/未完了をどう表すか、UI loop停止時に同期workerをどう扱うかは未決。 |

## 3. 同期性の境界

callback内からshutdown要求を積み、その場でcleanup完了まで同期waitすると、同じUI thread上で現在のcallbackが戻れず、shutdown処理も始められない。nested event loopで待つ方式は、shutdown前に別eventを再入させ、今回避けたい複雑さを再導入する。

そのため、次の動作を推奨する。

- 非同期呼出し: queue受付が成功した時点で呼出し側へ戻り、shutdown cleanup完了を待たない。cleanupはUI threadで実行する。
- 同期呼出しのcallback/event中: queue受付が成功した時点で呼出し側へ戻る。shutdown cleanupは現在のevent終了後、優先処理としてUI threadで同期実行する。呼出し側へは要求受付済み・cleanup未完了を示す必要がある。
- 同期呼出しのworker thread: shutdown requestをUI loopへdispatchし、cleanup完了までworker callerを待たせる。UI threadがpumpしない場合の待機・timeout/cancel方針はwrapper/API設計で明示する。
- UI thread上でcallback/event処理中ではない呼出し: UI thread上でshutdown cleanupを同期実行する案とする。

同期呼出しのcallback中はcallerから見た戻りが非同期になる。`runtime_shutdown`の現ABIは`ui_status`のみを返すため、statusを「request受付成功」と読む条件、shutdown完了の観測手段、最後のRuntime handleを閉じる時点をDD-06と全言語wrapper設計で揃える必要がある。

## 4. DDフェーズへの反映

- DD-02 DD-LIFE-07は、通常shutdownの状態遷移とcleanupを定める。callback内要求の優先dispatchは本判断に従い、戻り値/完了観測はDD-06とwrapper設計で決める。
- DD-03は、shutdown requestを通常workより優先して現在のevent直後に実行するdispatcher契約とdispatch context検出を詳細化する。
- DD-04は、現在実行中callbackの終了後にclosure destroyをUI threadで一度だけ行う順序を詳細化する。
- DD-06/16およびwrapper各文書は、wait、pending/accepted status、完了通知のABI・言語別表現を同じ契約に合わせる。

## 5. 残るAPI設計課題

残る検討事項は、同期worker待機中にUI loopが停止した場合のtimeout/cancel、および受付済み・完了済みを表すstatusと完了観測方法である。これらはDD-06/16とwrapper詳細で決める。BD-36-01/02/04の方針確認は完了しており、DD-03の詳細化を進めてよい。

上記の未決事項はAPI境界の設計課題として残す。BD-36-01/02/04の利用者方針をDD-03の前提に反映する。
