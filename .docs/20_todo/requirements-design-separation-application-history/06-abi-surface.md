# 適用前記録 / 06-abi-surface.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [06-abi-surface.md](../../13_detail_design/06-abi-surface.md)

---

# DD-06 ABIシンボルと互換性

- 状態: DD-ABI-01～10承認済み（2026-10-07）。Binding domain code catalog追記承認済み（2026-10-14）
- 作成日: 2026-10-07
- 対象: 公開C ABI symbol/version/feature、共通status/error識別面、UI loop接続、shutdown受付結果
- 上位判断: ADR-0001、ABI A1～A32、Runtime R1～R4、BD-27/28/36
- 先行詳細設計: DD-01～05（内容承認済み）
- 後続: DD-07 Property、DD-09 Validation、DD-10 Command、DD-16 Platform/Rendering、DD-18～23 wrapper・配布・検証
- 作業計画: [DD-06 作業計画](../dd06-abi-surface.md)

本書のDD-ABI-01～10は2026-10-07に承認済みである。A1～A32、BD-27/28/36、DD-01～05の承認済み契約を変更しない。C宣言とsymbol一覧は設計資料であり、配布用headerまたは実装済みAPIではない。後続設計に送った詳細は、本書の承認範囲に含めない。

## 1. 範囲と責務

DD-06は公開symbol群、version/feature discovery、共通status/error識別子、Runtimeとhost UI loopの接続口、およびshutdown要求の戻り値を扱う。機能別APIの操作意味論は、それを所有するDDの記載に従う。

| 領域 | DD-06の責務 | 別設計の責務 |
|---|---|---|
| symbol | major付きの外部名、stable symbolの互換規則 | 各機能APIの引数・操作意味論 |
| runtime version | MAJOR.MINOR.PATCHのABI queryとsymbol major対応 | pre-release、package version、配布とloader探索順 |
| feature | stable feature IDと未対応値の扱い | 各featureの内部動作とpackage制約表 |
| status/error | 共通statusの意味・数値安定性、domain namespace | Property/Binding/Validation/Command固有codeと結果意味論 |
| event loop | bridgeのRuntime接続・所有・wake callback | OSごとのevent loop実装とnative pump（DD-16） |
| shutdown | 受付と完了を区別する戻り値、同期/優先処理との整合 | UI loopが停止した場合のworker wait復旧・timeout（DD-16/wrapper） |

## 2. 承認済み判断

| ID | 判断 | 出典・判断範囲 | 状態 |
|---|---|---|---|
| DD-ABI-01 | 外部C symbolは`ui_abi_<major>_<group>_<operation>`形式とする。既存設計の`ui_abi_1_*`をABI major 1とし、ABI majorをsymbol prefixで識別する。C calling convention、固定幅整数、opaque handleの型・意味はADR-0001とA1～A3を維持する。 | A1/A3、ABI §2、BD-28。 | 承認済み |
| DD-ABI-02 | `get_version`はnative runtimeの安定版MAJOR.MINOR.PATCHを返し、`major`は当該symbol prefixのmajorと一致させる。既存major/minor出力に`out_patch`を追加する。ABI 1導入前に署名を確定し、初回stable番号やpre-releaseは本判断に含めない。 | A3、BD-28-01/02/05。3要素への拡張はBD-28のruntime version query詳細化。 | 承認済み |
| DD-ABI-03 | 一度公開したmajor内では、既存symbolのsignature/calling convention、意味、所有権、thread制約、status値の意味を変更・再利用しない。互換追加は新symbolまたは新feature IDとしてMINORで行い、PATCHはABI互換のbug/security修正に限る。破壊変更は別major prefixとする。 | A3、BD-28-02/03。新major併存と配布形態はDD-22へ送る。 | 承認済み |
| DD-ABI-04 | `has_feature(id)`は、そのruntimeがfeatureの全公開契約を提供する場合だけtrueを返す。未登録・未知IDは`UI_OK`とfalseを返し、IDは一度割り当てた後に意味変更・再利用をしない。古いruntimeに新symbolが存在しない可能性はfeature queryだけで解決できないため、wrapperは動的symbol解決または宣言minimum versionを先に確認してから呼び出す。 | ABI §2/3、BD-28-03。runtime loaderと探索順はDD-18～23。 | 承認済み |
| DD-ABI-05 | `ui_status`は32-bitの安定した数値分類とし、値と意味を一度公開した後は変更・再利用しない。追加statusは未使用値への追加だけとする。wrapperは未知statusをgeneric failureとして保持し、未定義動作にしない。 | A2/A8/A18、DD-VAL-04～06、BD-27-01。 | 承認済み |
| DD-ABI-06 | 初版のstatusを§3の表とする。`UI_COMMAND_EXECUTION_FAILED`と`UI_INPUT_FEEDBACK`はError snapshotが持つ結果分類であり、通常のAPI呼出し戻り値ではない。`UI_PENDING`はshutdown要求が受付済みだが未完了のとき返す非失敗statusとし、wrapperは通常のAPI failureとして扱わない。 | A2/A4/A18、BD-36、DD-03 §1/5/7、DD-05。既存status値との対応を示す。 | 承認済み |
| DD-ABI-07 | 共通error domainの初版を§4の表とし、`domain`・`code`は32-bit安定値とする。domain 0とcode 0は「未分類／具体理由なし」、標準domainは1～`0x7fffffff`、利用者定義domainは`0x80000000`～`0xffffffff`を割り当てる。コードはdomain内で一意・不変とし、messageを識別子に使わない。 | BD-27-01/02/05、A15/A18、DD-VAL-05。`validation_error_create`で利用者がdomain/codeを渡すA18と共存する。 | 承認済み |
| DD-ABI-08 | `runtime_set_ui_loop_bridge`をUI threadから一度登録できる公開hookとして設ける。Runtime生成自体はbridgeなしでも可能とし、登録成功後にCoreがwake closureを所有する。wake callbackは任意threadから呼ばれ、host loopに非同期scheduleを依頼するだけで、callback中にdispatch/cleanupを実行しない。 | DD-DSP-05/06、A4、R2。具体platform bridgeはDD-16。 | 承認済み |
| DD-ABI-09 | `runtime_shutdown`はRuntimeがClosedになる同期操作とする。UI thread callback/event内の呼出しは実行中処理を中断せず、優先control taskとして受付後に`UI_PENDING`で戻る。workerからの呼出しはDD-03に従いUI threadの完了を待ち、完了後に`UI_OK`を返す。shutdown開始前のUI thread外の呼出しはbridgeへdispatchする。 | A4、BD-36、DD-LIFE-07/08、DD-DSP-03/05/08。停止済みhost loop中の同期wait解除方法は未決として§6に残す。 | 承認済み |
| DD-ABI-10 | 利用者定義の失敗理由は`ui_status`へ個別値を追加せず、利用者domain内の`code`として表す。利用者はwrapperでenum等の名前付き定義を作れ、`auto`相当の採番も許す。ただし採番はprocess実行時ではなく宣言/build時に行い、公開済み値の再採番・再利用を避ける。全言語で同じcodeを使う場合は共通catalog/生成manifest等で割当てを共有する。 | A2/A18、BD-27、DD-ABI-05/07。enum構文とcode生成/manifestの具体形はDD-18～21へ送る。 | 承認済み |

## 3. 共通status

下表を初版の共通statusとする。`UI_PENDING`とError snapshot用statusの位置づけを明確にする。数値1～13は現`abi-spec.md`の候補に合わせ、14を追加する。

| 数値 | 名称 | 意味 |
|---:|---|---|
| 0 | `UI_OK` | API操作の成功または同期完了 |
| 1 | `UI_INVALID_ARGUMENT` | pointer/length不整合、無効なscalar値、引数制約違反 |
| 2 | `UI_INVALID_HANDLE` | 0、未知、または既に無効なhandle |
| 3 | `UI_WRONG_TYPE` | 有効なhandleのkind不一致 |
| 4 | `UI_WRONG_RUNTIME` | kind一致だが別Runtime所属 |
| 5 | `UI_WRONG_THREAD` | UI thread限定APIへの誤thread呼出し |
| 6 | `UI_CLOSED` | shutdownで閉じたRuntimeへの新規操作 |
| 7 | `UI_BUFFER_TOO_SMALL` | caller buffer不足。DD-VAL-03/06に従い必要byte数を返す |
| 8 | `UI_OUT_OF_MEMORY` | 操作に必要なmemory確保失敗 |
| 9 | `UI_INTERNAL_ERROR` | callback異常、panic等の内部・境界失敗 |
| 10 | `UI_UNSUPPORTED` | 未対応API/feature。未知feature query自体はfalseを返す |
| 11 | `UI_VALIDATION_FAILED` | 同期Property validationによる直接setter拒否 |
| 12 | `UI_COMMAND_EXECUTION_FAILED` | Async Command failure snapshot内のerror status。呼出し戻り値として使わない |
| 13 | `UI_INPUT_FEEDBACK` | TextBox input reason snapshot内のerror status。呼出し戻り値として使わない |
| 14 | `UI_PENDING` | shutdown requestを受け付けたが、callback/eventが戻ってからcleanupするため未完了 |

`UI_PENDING`はfailureではなく、呼出し側がshutdown完了済みとして扱ってはならない状態を示す。

## 4. Error domain namespace

| 数値 | domain | 初版の所有範囲 |
|---:|---|---|
| 0 | unspecified | 詳細理由・domainを提供できないerror |
| 1 | API/Runtime | 引数、handle/runtime、thread/lifecycle、Core/ABI資源失敗 |
| 2 | Validation | Property validatorとbinding validationに属する理由 |
| 3 | Binding | graph、接続、伝播のBinding固有理由 |
| 4 | Async operation | Command実行・完了報告に属する理由 |
| 5 | Input | View入力拒否、途中入力、IME確定候補の理由 |
| `0x80000000`～`0xffffffff` | user-defined | application/利用者が独自に割り当てるdomain namespace |

domain 1～5の数値は安定識別子として予約し、domain名は表示文言に使わない。標準codeは機能APIを所有するDDが割り当て、コード値を再利用しない。現時点で具体名まで承認済みの`HANDLE_TOKEN_EXHAUSTED`（DD-HND-06）は、API/Runtime domain内のcode 1とする。Binding domain codeは2026-10-14の承認により次表のとおり追加する。その他の機能固有codeは各機能の詳細設計で追加し、未定義のcodeをこの表で作らない。

| domain | code | 名称 | 意味 | 出典・状態 |
|---:|---:|---|---|---|
| 3 (Binding) | 1 | `GRAPH_CYCLE` | 追加接続またはTwoWay group結合後にOneWay graphが循環する | 利用者承認済み（2026-10-14） |
| 3 (Binding) | 2 | `TARGET_ALREADY_HAS_INPUT` | OneWayのtarget groupがすでに別のsource groupを持つ | 利用者承認済み（2026-10-14） |
| 3 (Binding) | 3 | `DUPLICATE_EDGE` | 同じsource groupから同じtarget groupへのOneWay edgeがすでにある。同一edge再登録時は複数入力元検査より優先して返す | 利用者承認済み（2026-10-14） |

利用者定義domain内のcodeは、利用者が名前付き定義で管理できるようにする。`auto`形式はenum memberの宣言順から値を付けるsource-level convenienceとして扱い、ABIは確定済みの`uint32_t domain`と`uint32_t code`を受け渡す。Python `enum.auto()`のような宣言順採番を使う場合、既存メンバーの挿入・並べ替えで過去codeが変化しない運用（append-only、または生成manifestで値を固定）が必要となる。実行ごとの順序・同時registrationに依存する採番は、保存済みerrorや別言語間の同値性を壊すため採用しない。

## 5. UI loop接続のABI宣言

次のC宣言は承認済み設計のsignatureを示し、配布用headerまたは実装済みAPIではない。

```c
typedef void (*ui_wake_callback)(void *userdata, uint32_t priority);

enum {
  UI_WAKE_NORMAL = 1,
  UI_WAKE_CONTROL = 2
};

ui_status ui_abi_1_runtime_set_ui_loop_bridge(
    ui_handle runtime,
    ui_wake_callback wake,
    void *userdata,
    ui_destroy destroy,
    ui_handle *out_error);
```

- 登録はUI thread専用で一度だけとし、Runtimeに既にbridgeが登録済み、またはshutdown開始後なら失敗させる。成功時にwake/userdata/destroy一組の所有権をCoreへ移し、失敗時は呼出し側に残す。
- `UI_WAKE_NORMAL`は通常post/通知のdispatch機会、`UI_WAKE_CONTROL`はshutdown等の優先control taskをhost loopへscheduleする指示とする。callbackは任意threadから呼べるが、同期dispatchせず直ちに戻る。
- CoreはRuntime APIやuser callbackをbridge callbackのstack上で実行しない。bridgeの`destroy`はRuntime close後にUI thread上で一度だけ呼ぶ。
- bridge未登録でもUI thread上の同期APIは使用可能とする。worker post・worker shutdownなどUI wakeが必要なAPIは、所有権移転前に`UI_UNSUPPORTED`で拒否する。
- host loopがwakeを受け入れた後に停止する競合、wakeの失敗通知、bridge交換、複数Runtimeのhost登録は未決またはDD-16へ送る。

## 6. Shutdown返却と未決事項

| 呼出し位置 | 返却status | 返却時点 |
|---|---|---|
| UI thread上、callback/event外 | `UI_OK` | cleanupが完了しClosedになった後 |
| UI callback/event内 | `UI_PENDING` | 現在の処理を中断せずpriority control taskを登録した後 |
| worker thread、bridge有効・UI loop稼働中 | `UI_OK` | UI threadのcleanup完了後（同期） |
| UI wakeが必要だがbridge未登録 | `UI_UNSUPPORTED` | 呼出し前。Runtime状態と所有権を変えない |

- `UI_PENDING`を返した後に完了を待つticket/future/通知APIは追加しない。Runtime closed状態や既存の終了通知で完了を観測できるか、API上の完了確認が必要かは未決。
- workerが同期shutdownを待っている間にhost UI loopが停止・破棄された場合の解除方法、timeout/cancel、およびUI loop停止とRuntime closeの順序は未決。DD-16およびwrapper側で継続レビューする。
- `runtime_shutdown`がcallback内で`UI_PENDING`を返しても、DD-DSP-08どおり実行中eventの後にcleanupし、通常postより優先する。cleanup開始時点で未開始の通常postをcancelし、destroyする。

## 7. 互換性・検証表

| 変更 | ABI version | 互換性条件 |
|---|---|---|
| 新function symbol/new feature | 同major、MINORを上げる | 既存symbolと動作を保持。新featureなしのruntimeでも旧wrapperが動く |
| ABI互換のbug/security修正 | PATCHを上げる | signature、status意味、所有権、thread契約を変えない |
| 既存signature/status意味/所有権/thread契約の破壊変更 | 新major prefix | 新major世代として分離。旧major support範囲はBD-22/DD-22で扱う |
| 既知status/domain/code/featureの追加 | 同majorで追加可能 | 数値と既存の意味を維持し、古いwrapperが未知値を安全に扱う |

互換性の確認ケース:

1. ABI major 1向け旧headerで構築したC clientを、同majorの新minor/new patch runtimeへ接続する。
2. 新minor専用symbolを必要とするwrapperは、古いruntimeでsymbol解決前に必要minimum version/featureを確認し、未対応を明示する。
3. 新runtimeが返す未知status・未知error domain/code・未知featureを古いwrapperが未定義動作なく保持またはgeneric failureとして扱う。未知featureはfalse。
4. `get_version`のmajorとsymbol prefixが一致し、minor追加とpatch修正の説明がBD-28に合う。
5. 4言語wrapperで同じstatus分類、Error snapshot、`UI_PENDING`の受付済み意味を得る。
6. UI loop bridgeのnormal/control wakeはUI threadへ非同期scheduleされ、control wakeが実行中eventを中断しない。post成功後のclosure所有権はDD-03/04に従う。

## 8. 未決境界と承認記録

- shutdown完了を追加query/ticketなしで観測できるか、またAPI上の完了確認が必要か。
- 利用者定義codeの自動採番を各wrapperのsource syntaxで行うか、共通catalog/生成manifestで行うか。既存codeの値を保つ具体策はDD-18～21で決める。
- bridgeがwakeを受けた後にhost loopが停止する競合、wake失敗通知、bridge交換、複数Runtimeのhost登録。
- 同期worker shutdown中にhost UI loopが停止した場合の待機解除。これはDD-16/wrapper設計へ送る。
- ABI symbolのmajor世代を同一artifactで併存させるか、stable runtime discovery/package metadataへどう反映するか（DD-22）。

DD-ABI-01～10は2026-10-07に承認済みである。未決境界とDD-07以降の機能固有API、DD-16のplatform loop実装、DD-18～23のwrapper/配布設計は各フェーズで確認する。

## Binding code catalog追記の承認履歴

| 日付 | 対象 | 承認者 | 改訂内容 |
|---|---|---|---|
| 2026-10-14 | Error domain catalog / Binding domain | 利用者 | Binding domain code 1～3を追加し、意味を固定。既存DD-ABI-01～10の判断は変更しない |

## 改訂履歴

| 日付 | 改訂内容 | 承認状態 |
|---|---|---|
| 2026-10-14 | Binding domain固有code 1～3を追加 | 利用者承認済み |
