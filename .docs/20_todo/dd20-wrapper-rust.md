# DD-20 Rust safe API 作業計画

- 状態: 完了（DD-RS-01～15承認済み、2026-10-12）
- 着手日: 2026-10-12
- 対象: Rust safe wrapperの所有型・借用型、thread-safety、dispatcher、callback、panic境界、error・shutdown表現
- 上位出典: Accepted ADR-0001、Accepted要件、承認済みL1～L4、A1～A32、BD-01/16/20/27/28/36
- 先行詳細設計: 承認済みDD-01～19
- 対応する詳細設計: `.docs/13_detail_design/20-wrapper-rust.md`

## 目的

Rust利用者へraw pointerやC ABI handleの操作を通常公開せず、共通C ABIを安全に包む層の責務と境界をレビューする。Rust Coreを重複同梱せず、BD-20に従って`-sys` crateとsafe wrapper crateを分ける。所有権、UI thread制約、callback panic、失敗可能なtoken releaseをRustの型とcleanup規則へ写す。

この作業は詳細設計レビュー案までとし、crate名、Rust最低version/edition、公開型・メソッド名、API signature、bindgen採否、link/load方式、artifact取得featureの具体名は確定しない。計画済み詳細設計完了後に作るAPI設計章の保留方針を維持する。

## 作業手順

1. ADR、要件、L1～L4、A1～A32、BD-16/20/27/28/36、DD-01～19の本文状態と判断IDを確認し、承認済み契約と未決事項を分離する。
2. raw `-sys` crateとsafe wrapper crateの責務を分け、raw ABI型・関数・pointerをsafe層がどう隠すかを提案する。Rust Coreやnative runtimeをcrateへ重複同梱しないBD-20の境界を保つ。
3. owned token、borrowed view、retainによる複製、Runtimeとの関連をRust所有型へ対応付ける。owned wrapperは暗黙複製させず、borrowed viewはscopeを越えて保存できない案とする。
4. UI-affine handleの`Send`/`Sync`制約と、worker-safe dispatcher/post経路の制約を別々に提案する。markerの実装例は説明用に留め、正確なRust version/editionや公開型名を確定しない。
5. `Drop`が失敗を返せない点に対して、明示closeの結果返却と、Drop時のrelease失敗を失わない言語内cleanup ticket/registry案を整理する。ticketをowned token取得前に確保し、DD-LIFE-06の「enqueue失敗時にtokenを消費しない」を維持する。
6. workerからdispatcherへ渡す値とclosureの`Send + 'static`要件、登録失敗時の所有権、UI thread上の実行・破棄をDD-03/04に照らして提案する。UI thread上だけで登録するcallback closureへ一律の`Send`を要求しない。
7. callback trampoline/userdataの所有権移転と一度だけのdestroyを示し、safe Rust closureからraw contextへ渡るunsafe区間を局所化する。
8. callback panicとuserdata destructor panicをC ABI境界へ伝播させない方針を整理する。`panic=unwind`での捕捉範囲と`panic=abort`では捕捉できない限界を分け、callback固有のfailure意味論や共通handler APIは新設しない。
9. `ui_status`・immutable Error snapshotからのRust `Result`相当の失敗表現、unknown status/codeの保持、UI thread誤用、shutdown pending/completedの区別を提案する。公開名・型は例示に限る。
10. Rust公式一次資料を参照し、normal/unwind/abort、unsafe FFI境界、`Send`/`Sync`、Dropの検証観点と残課題を記録する。
11. 本計画を先に保存した上で`.docs/13_detail_design/20-wrapper-rust.md`へ`DD-RS-*`判断ID付き未承認レビュー案を作成する。承認前は計画・状況文書を完了へ変更しない。

## 対象外と停止条件

- C ABI、DD-01～19で承認済みの寿命・callback・dispatcher・error意味論の変更
- crate名、registry/feed、最低Rust version、edition、公開API名と最終signature
- bindgen対手書き、header配布、Cargo build script、static/dynamic linking、native runtime loading、artifact取得の具体実装
- Rust Coreのcrateへの同梱、target runtimeの再ビルド。BD-20に反する統合はしない
- ARM/Linux board profileおよびproduction runtime supportの確定。BD-16/DD-22のverification gateに従う
- callback panic時の全機能共通failure動作、diagnostics payload、利用者handler登録API
- loop停止中の同期shutdown解除や完了観測APIの追加。DD-03/06/16の未決として維持
- `Send`/`Sync`のunsafe実装を実際に許可する判断。内部同期とC ABI契約が証明されない型にunsafe implを提案しない

## レビューで確認する点

1. safe crateはunsafe FFIを隠す高位境界となり、raw `-sys` crateと責務を混ぜないこと。
2. owned tokenはmove-only相当、複製は明示retain、借用は呼出し範囲に限定すること。
3. UI-affine handlesは原則`Send`/`Sync`にせず、worker postだけに必要なclosure移送条件を課すこと。
4. `Drop`はpanicせず、release失敗tokenを事前確保cleanup ticketへ移管し、成功まで所有権を失わないこと。
5. callback/userdata ownership transfer、destroy一回性、panic containmentがDD-04と一致すること。
6. callback中のshutdown要求は受付後に戻り、worker同期shutdownはloop稼働時に完了を待つ境界を保つこと。
7. API名等の公開surfaceは最終決定せず、後続API設計章へ送ること。

## 承認記録

2026-10-12にDD-RS-01～15が利用者承認済みとなり、`.docs/13_detail_design/20-wrapper-rust.md`へ承認結果を反映した。crate/API名、MSRV/edition、cleanup registry実装、callback固有panic挙動、Cargo/link/load詳細は未決として維持する。
