# Python Window peer / close-request wrapper

- 計画版: 0.1
- 状態: 計画・DD v2.0・test design v1.2承認済み。実装・Windows local gates・同一SHAのWindows/Linux CIが成功しunit完了（2026-10-26）
- 親計画: [Python Binding・Command・UI Controls wrapper](python-ui-controls-wrapper.md)
- 対象branch: `impl-python/runtime-wrapper`
- 対象unit: `UNIT-WRAP-PY-WINDOW-001`

## 目的

承認済みRQ-09-071と既存C ABIにあるWindow peerの明示open/close、およびclose-requestのAccept/Decline/DeferをPython facadeへ写す。既存Python `Window.close()`はowned handle解放を意味するため、peer終了を混同しない名前・所有契約を提案する。Core/Rust/C ABIやOS peerは変更しない。

## 根拠・現状

- 要求: [RQ-09 UI範囲](../11_requirements/RQ-09_ui-scope.md) RQ-09-001、RQ-09-003、RQ-09-071。RQ-09 v0.5は2026-10-17に全文承認済み。
- 上位設計: [ADR-0001](../04_approval/adr-0001-rust-core.md)、[UI Tree/Layout/Controls](../12_design/ui/tree-layout-controls.md)、[Platform abstraction](../12_design/platform/abstraction.md)。
- 詳細設計: [共通C ABI](../13_detail_design/runtime-objects/abi.md) §Window peerとclose request（受理済み1.3）、[UI Controls](../13_detail_design/ui/controls.md)、[Python wrapper](../13_detail_design/wrappers/python.md) v1.10（Window peer/close-requestは明示的に次unitへ除外）。
- ABI: 公開headerには`window_open/close`、`window_close_request_subscribe/defer/resolve`が存在する。close callbackはborrowed requestを受け、deferだけがowned requestを返す。Subscriptionは既存cancel APIを使用する。
- 実行環境境界: 現行host C ABIではWindow peer openが`UI_UNSUPPORTED`となる。公開C ABIにOS close intentを発生させる操作はないため、実peer表示やclose callbackの実発火を別process consumerの合格条件にしない。

## 提案scope

- `Window.open()`とpeer専用終了操作`close_peer()`。既存`Window.close()`はControl owned tokenのreleaseのまま維持する。
- Window単位のclose-request handlerを一つ登録し、所有型のSubscriptionを返す。二重登録は既存ABI statusをそのまま伝える。
- callback内だけ有効なborrowed request viewからAccept/Declineを即時選択するか、Deferしてowned requestを受け取る。deferred ownerは後刻resolveでき、明示close/releaseする。
- callback root、Window/Runtime anchor、Subscription cancelとtoken releaseの区別、C境界での例外捕捉を既存Python owner/callback規則へ接続する。
- Runtime shutdownは個別close vetoを待たず、Window peer/callback rootのcleanupは既存Core/ABI契約に委譲する。

## 対象外

Core/Rust/C ABI変更、OS固有peer・描画・実画面検証、close intentを生成するtest-only/public API、複数close subscriber、close decisionの合成、要求変更、既存Window content/Control Tree再設計。

## 実施順と承認境界

1. [x] `impl-python/runtime-wrapper`、unit台帳、Python `Window.close()`と既存C ABI/設計を確認。既存実装への差分mergeはしない。
2. [x] 計画0.1、Python DD v2.0、wrapper test design v1.2を利用者の「ok」（2026-10-26）により文書単位で承認。実装承認とは分離する。
3. [x] Python DD v2.0でAPI名、peer open/closeとhandle releaseの区別、borrowed/deferred request lifecycle、subscription/callback root、例外・shutdown境界を提案し、利用者承認を得た。
4. [x] Wrapper test design v1.2でfake ABI lifecycle casesと実C ABI consumerの制約を定義し、利用者承認を得た。設計承認と試験設計承認は別記録とする。
5. [x] `UNIT-WRAP-PY-WINDOW-001`をplanned登録し、利用者の実装開始承認後に実装を開始する。
6. [x] 実装開始承認後、fake callback shimのlifecycle/decision cases 11件を実行。実C ABI consumerでsubscription登録・cancel、openの`UI_UNSUPPORTED`とstatus伝達を確認し、OS close intent発火や画面表示は主張しない。
7. [x] 同じunitに実装・tests・consumerを揃え、focused gates、Windows local `tools/dev.py check`、`minimum`を実行し成功。実装revisionを日本語commit/pushし、同一SHAのWindows/Linux CI成功後にunitをcompleteにする。

## 完了条件

- Python Window peer/close-request DDとtest designが各文書全体で承認済み。
- open/peer closeとowned handle closeを混同せず、Accept/Decline/Deferとcallback root/Subscription lifecycleをPython APIから利用できる。
- fake ABI試験がcallback/owner edgeを確認し、実C ABI consumerが利用可能な現行API範囲だけを確認する。
- unitのsource/testとCI成功が同一revisionに対応し、unit台帳・試験記録・memoryを更新する。
- OS peer未実装による`UI_UNSUPPORTED`を、表示可能またはclose intent実発火済みとして誤認しない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-26 | 計画0.1 / DD v2.0 / test design v1.2 / UNIT-WRAP-PY-WINDOW-001 | 利用者承認 | 利用者「ok」。文書全体を承認しunitをplanned登録。実装開始は別承認 |
| 2026-10-26 | 実装開始 | 利用者承認 | 利用者「ok」により実装開始。fake ABI 11件、real C ABI consumer、Windows local check/minimum成功。Windows/Linux CIはcommit後に確認 |
| 2026-10-26 | 実装完了 | 完了 | 実装SHA `1b4e686c31cc66c843f27bc123ba7790fddd2189`のWindows/Linux CI成功。[run 40306768925](https://github.com/edamame-edame/freemvvm/actions/runs/40306768925)。unit台帳をcompleteへ更新 |
