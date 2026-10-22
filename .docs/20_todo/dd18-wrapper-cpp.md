# DD-18 C++ wrapper 作業計画

- 状態: 完了（DD-CPP-01～15承認済み、2026-10-12）
- 着手日: 2026-10-12
- 対象: C++ wrapperの所有handle、RAII、callback保持、明示解除、例外/error変換、thread affinityとRuntime shutdown
- 上位出典: Accepted ADR-0001、Accepted要件、承認済みL1～L4、A1～A32、BD-01/27/28/36
- 先行詳細設計: 承認済みDD-01～17（DD-17のOS固有Accessibility API/ABI未決は対象外）
- 対応する詳細設計: `.docs/13_detail_design/18-wrapper-cpp.md`

## 目的

C ABIを直接扱う利用者にtoken・closure・thread契約の誤用をさせず、C++で所有参照の寿命を読み取れるwrapper案を定める。L1～L4、DD-01～10の承認済み契約をC++の型と操作へ対応させ、API固有意味論を所有するDD-07～17との接続境界を示す。

## 作業手順

1. ADR、Accepted要件、`language-bindings.md` L1～L4、`abi-spec.md` A1～A32、関連BD判断とDD-01～17の状態・承認IDを確認する。
2. handle tokenのowned/borrowed区別、retainによるclone、releaseとUI-thread finalizerの関係をC++ object model案にする。DD-02のworker finalizer enqueue失敗でtoken未消費となる規則と、C++ destructorのnoexcept性が両立するかを明示する。
3. Runtime/Property/Binding/Subscription/Command/Widget等のwrapper objectについて、所有・借用、move/copy、close/dispose、Runtimeより先に解放すべき依存を表にする。具体Control関数名や未定義ABI symbolは追加しない。
4. callback userdataの確保、Coreへの所有権移転、callback中のC++ exception捕捉、destroyでの一度限りの解放、自己解除とin-flight寿命を提案する。
5. `ui_status`、immutable Error snapshot、未知status、任意`out_error`、UTF-8/DECIMALの型変換をC++ error/value surfaceに対応づける。暗黙の数値変換やABI外へのexception伝播を防ぐ。
6. UI-thread限定操作、worker dispatcher、shutdownのUI-thread/callback/worker別返却、`UI_PENDING`をwrapper surfaceへ写し、UI loop停止時の未決を維持する。
7. 正常・失敗・競合時の検証ケースと、未決事項の後続担当（DD-02/06/07/16/22～24等）を記す。
8. 判断ID付きの未承認レビュー案を作り、参照リンクとstatusを確認する。利用者承認後に限り、本書・本計画・フェーズ一覧・状況メモを完了更新する。

## レビュー判断の予定範囲

- move-only owned handleと明示的なretain/clone、およびborrowed viewの表現
- handle destructor / 明示close / UI-affineな最終破棄の失敗可能性
- callback closureのcapture保持・例外遮断・destroy規則
- Error/statusをC++の戻り値・例外へ対応させる方式
- UI thread APIとworker post/同期shutdownの見分けやすさ
- Runtime、Subscription、Binding、Window等の明示終了と寿命順序

## 対象外と停止条件

- 配布用headerや実装済みAPIとしてC宣言を確定すること、未定義symbol/API名を確定すること
- DD-02/06/07/09/10/11～17の承認済み意味論を変更すること
- C++標準バージョン、namespace、最終的な公開型名、build/package方式を決めること（必要なら候補と判断依存のみ示す）
- DD-17のUIA/AT-SPI binding実装、Accessibility provider lifetime/thread option
- pending-cleanup registryの具体実装・APIを、本書の承認だけで確定すること。DD-LIFE-06との接続方式は承認済みだが、registry詳細は追加API設計で具体化する

特にDD-LIFE-06は、worker releaseのUI finalizer enqueueに失敗した場合tokenを消費しない。利用者承認により、C++ wrapperがOwnedHandleごとに事前確保したcleanup nodeを持ち、destructorから失敗tokenをlanguage-local registryへ移して再試行する案とした。native DD-02/ABI契約は変更しない。registryのthread safety、Runtime pinning、wake、shutdown後drain、library unload寿命は後続API設計へ送る。

## 完了・承認記録

利用者は2026-10-12にDD-CPP-01～15を承認した。例外無効buildの非throw result経路、例外有効buildの利用者設定callback exception handler、C++内pending-cleanup registryを追加し、DD-18を完了した。追加API具体化は計画済み詳細設計DD-00～22完了後に整理・作成する方針のため保留する。DD-19はこの全体計画に沿って開始した。

## 完了条件

- L1～L4、DD-01～10の所有権・thread・callback・error・shutdown契約との対応が追跡できる。
- owned tokenとborrowed argumentを型/操作で区別し、retain/releaseとcallback closureの所有権移転が一貫している。
- destructor、明示close、finalizer enqueue失敗、callback内shutdownの失敗・戻り時点が記述されている。
- C++例外がABI境界を越えず、status/errorの情報を安全に保持する方針がある。
- C++標準・型名などの未決事項とDD-19～21へ再利用する共通論点が分離されている。
- 利用者承認後に判断結果を本書と計画/status文書へ反映する。
