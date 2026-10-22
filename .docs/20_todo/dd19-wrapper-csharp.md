# DD-19 C# wrapper 作業計画

- 状態: 完了（DD-CS-01～14承認済み、2026-10-12）
- 着手日: 2026-10-12
- 対象: C# P/Invoke、opaque handle wrapper、Dispose/lifetime、callback delegate/userdata保持、thread affinity、status/error変換
- 上位出典: Accepted ADR-0001、Accepted要件、承認済みL1～L4、A1～A32、BD-01/18/27/28/36
- 先行詳細設計: 承認済みDD-01～18
- 対応する詳細設計: `.docs/13_detail_design/19-wrapper-csharp.md`

## 目的

C#から共通C ABIを安全に呼び、managed object/delegateをABIへ露出せず、GC・callback・Dispose・Runtime shutdownの寿命を明確にする。RID別native assetを同一NuGet packageへ含める承認済みBD-18を維持するが、package ID/TFM/loaderの完全な配布仕様はDD-22に送る。

## 作業手順

1. ADR、Accepted要件、L1～L4、A1～A32、BD-18、DD-01～18の状態と承認IDを照合する。DD-18で追加したnon-throw result、callback exception handler、language-local cleanupの要求を共通前提として整理する。
2. `ui_handle`のuint64固定幅とC#表現、owned/borrowed lifetime、clone/retain、P/Invoke中にDisposeと競合した場合のtoken pinningを提案する。`SafeHandle`と独自ownerの候補を比較し、解放失敗契約との矛盾を明示する。
3. Persistent callbackのmanaged state、static delegateまたは代替trampoline、GC root (`GCHandle`等)、登録成功/失敗、destroy一回性を状態遷移として提案する。
4. `IDisposable`による明示解除、Runtime shutdown、finalization fallbackとcleanup reaperの責務を整理する。finalizerだけでUI-affine cleanupを保証しない。
5. status/error snapshotからC#のResult/exception surfaceへの変換、unknown code、UTF-8、callback exceptionの利用者handler接続を提案する。DD-18の例外無効要求はC#へ機械的に当てはめず、C#で必要な明示error surfaceを考える。
6. UI thread専用API、worker dispatcher、shutdownの`UI_PENDING`/同期完了をC# surfaceへ写す。DD-03/06/16の未決loop停止動作をここで補完しない。
7. P/Invoke declaration generation style、TFM、assembly resolver、RID native loadingのうち、DD-19が決めるwrapper境界とDD-22/BD-18が所有する配布・loader判断を分ける。
8. 正常・失敗・GC・Dispose競合・shutdown時の確認ケースを作成し、一次資料を参照して未決事項を割当てる。
9. `.docs/13_detail_design/19-wrapper-csharp.md` に判断ID付きの未承認レビュー案を作成する。承認後のみ計画、フェーズ一覧、状況メモを完了更新する。

## 対象外と停止条件

- C ABI symbol/signatureやDD-01～18承認済み意味論の変更
- NuGet package ID/feed、正式TFM、完全RID/support matrix、native filename・依存bundle、load resolverの最終採用（BD-18とDD-22）
- callback exception handlerの全言語共通API形。C#では利用者差し込み要件とmanaged exception containmentだけを提案する
- C# version、unsafe/function-pointer要件、source-generated interop採用を未承認前提だけで確定すること
- Accessibility providerやUIA/AT-SPIのC# binding選定（DD-17の未決）

## 完了条件

- owned/borrowed handle、P/Invoke pinning、Dispose、callback root/destroy、thread/shutdown/errorがDD-01～18と整合する。
- callbackまたはP/Invoke中のGC/Dispose競合でuserdata/delegate/handleを途中解放しない。
- worker finalizer queue失敗をC# GC finalizerだけで処理せず、retry可能な言語側所有経路を示す。
- SafeHandle採用候補のReleaseHandle制約と失敗可能releaseを照合し、判断境界を明示する。
- BD-18の同梱packageと初期RIDを追跡し、DD-22へ残すpackage・TFM・loader事項が見える。
- 利用者承認後に判断結果を本書・計画・フェーズ/statusへ反映する。

## 完了・承認記録

利用者は2026-10-12にDD-CS-01～14を記載どおり承認した。承認範囲と未決事項を `.docs/13_detail_design/19-wrapper-csharp.md` に反映し、DD-19を完了した。SafeHandleの採用、TFM、interop generator、native loader、cleanup registry実装と最終API名は後続に残す。利用者の次フェーズ指示によりDD-20を開始する。
