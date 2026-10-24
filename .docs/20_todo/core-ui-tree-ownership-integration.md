# Core UI TreeとHandleRegistryの内部所有統合

- 計画版: 0.1
- 状態: A案および関連DD/test文書全体承認済み（利用者、2026-10-24）。Core実装進行中
- 対象version branch: `version/0.0.1`
- Core作業branch: `impl-rust/core-ui-tree-ownership`
- 対象: Logical Treeのparent-child所有とRuntime object cleanupの接続

## 見つかった設計境界

[UI Tree詳細設計](../13_detail_design/ui/tree-layout.md) 1.1は、親が所属中の子を強く所有し、子から親への参照を非所有とする。attachは親子・Runtime・循環・必要資源の検証後に一括commitし、detach後も外部所有者があるsubtreeはDetachedとして利用可能にする。

[Object handle詳細設計](../13_detail_design/runtime-objects/objects-values.md) 1.2の`CoreObjectControl`はpublic参照数とcall期間中のpin数を持つ。最後のpublic参照が解放されるとcleanup ticketを予約し、Runtime UI object storeからstateを破棄する。ここにはTree parentが保有する内部strong referenceの数え方と、内部所有中の子を新しいpublic ownerとして返す方法が定義されていない。

そのままでは、子のpublic handle解放がTree所有中のstateを早期破棄するか、pinを内部所有として流用して設計上のcall-duration pinの意味を崩す可能性がある。これは実装の都合だけで確定しない。

## 採用する解決方針（A）

`CoreObjectControl`にpublic reference、internal owner、in-flight pinを区別して追跡する内部owner契約を追加する。TreeCoordinatorはattach準備時に子のinternal ownerと必要cleanup reservationを確保し、全検証・準備成功後にだけparent-child関係をcommitする。detach時は親子関係を先に一貫して更新し、internal ownerを解放する。内部ownerが残る間はpublic handleが0でもUI stateを維持する。

cleanup ticketは、public referenceとinternal ownerがともに0となった場合に一度だけ有効化する。in-flight pinが残る場合は既存のpin規則に従ってcleanupを遅らせる。owner作成時に必要資源を予約し、予約失敗ならTreeを変更しない。Treeの子snapshotからpublic handleを返す場合は、live internal ownerを根拠に独立したpublic ownerを発行する経路を設ける。

利用者は2026-10-24にA案を選択した。A案は[承認済みTree設計](../13_detail_design/ui/tree-layout.md)の親strong/子weakとDetached subtreeの外部保証を実現しつつ、[承認済みObject handle設計](../13_detail_design/runtime-objects/objects-values.md)のcleanup reservation・pin規則を保つ。具体的な型名や関数名は実装詳細として扱い、外部APIは追加しない。

## 代替案と比較

| 案 | 内容 | 評価 |
| --- | --- | --- |
| A（推奨） | Registry controlにinternal owner countを追加し、public ref/pinと分けてcleanup条件を管理 | 設計上の所有区分を保てる。cleanup reservationの取得・移管と、internal ownerからのpublic owner再発行をDDへ明記する必要がある |
| B | 親が既存HandleRegistryのretained tokenを内部ownerとして保持する | Registryのrelease経路を再利用しやすいが、内部所有とpublic handle参照が同じcountに混ざる。親state破棄時にtokenをUI thread上で確実にreleaseする所有型も必要 |

## 承認後の作業範囲

1. 完了: A案に沿うobjects-values / tree-layout / UI単体試験文書の改訂案を作成し、2026-10-24に全文承認を得た。
2. 承認済み計画・設計・test文書をcommitし、`tools/units.yaml`へCore implementation unitを登録する。
3. CoreObjectControl internal owner、Tree Coordinator、所有参照からのpublic handle再発行を実装する。
4. focused test、workspace test、`tools/dev.py check`、`tools/dev.py minimum`を実施し、実装unitごとにcommitする。
5. 0.0.1 version branchへCore child branchを統合する。別エージェントのWebEngine branchはこの変更に含めない。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-10-24 | 0.1 / Tree内部所有とHandleRegistry cleanupの接続案 | 計画・DD・test文書全文承認済み | 利用者の「ok」により承認。Core実装へ進行 |
