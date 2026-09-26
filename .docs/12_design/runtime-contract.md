# MVVM UIライブラリ Runtime契約（Slice 1）

- 状態: Review Draft（設計提案。承認前）
- 作成日: 2026-09-26
- 対象: Rust Core、C ABI、C++ / C# / Rust / Python wrapper
- 上位文書: `design-overview.md`、`mvvm-ui-library-requirements.md`、`adr-0001-rust-core.md`
- 対応するABI案: `abi-spec.md`

## 1. 範囲と判断の区別

ADR-0001が確定したのはRust Core、versioned C ABI、4言語wrapper、組み込みLinux対応である。本書のトークン方式、破棄規則、callback規則、version規則は**今回の設計提案**である。MVVM更新順序や画面制御は別スライスで定義する。

## 2. 用語と基本不変条件

| 用語 | 意味 |
|---|---|
| Runtime | UI thread、タスクキュー、handle表、停止状態を持つインスタンス |
| UI thread | Runtime作成時に紐づける単一スレッド |
| Handle | 所有参照1個ごとにABIが発行する非ゼロの64-bit不透明トークン。Core内のポインタ値ではない |
| Strong reference | `retain`で増え、`release`で減る公開参照 |
| Subscription | 通知を購読するためのhandle。解除時にcallbackを無効化する |
| Callback closure | 関数ポインタ、userdata、destroy関数の組 |

- 一つのRuntimeに紐づくUI状態はそのUI threadでのみ参照・変更する。異なるRuntime間でUI objectを共有しない。
- handleの値から内部アドレスや種類を推測しない。ゼロは無効handleである。
- Runtimeごとに登録するhandleの種類と所属を管理し、APIで不一致を検出する。
- 公開トークンはプロセス内で再利用しない。カウンタが尽きた場合は生成を失敗させる。`retain`は同一objectを指す**別のトークン**を発行する。これにより解放済みの所有参照と別の所有参照を区別する。
- `retain` / `release`は公開参照の管理であり、親子ツリーやBindingが保持する内部参照とは区別する。

## 3. 所有権と破棄

| 操作 | 参照の扱い | スレッド |
|---|---|---|
| Runtime/objectの生成 | 成功時に所有参照を1個返す | UI thread |
| handleを返すgetter | 特記なければ新しい所有参照を1個返す | 対象に応じUI thread |
| `retain(h)` | `h`を検証し、同じobjectへの別トークンの所有参照を1個返す | 任意 |
| `release(h)` | 呼び出し側の所有参照を1個消費する | 任意 |
| API引数のhandle | 呼び出し中だけ借用する。保持したい側が内部参照を取得する | APIごと |

- `release`で参照がゼロになっても、UI状態の最終破棄はUI threadで行う。workerからの`release`は破棄処理をキューへ渡す。破棄が完了する時刻を`release`の戻り値で保証しない。
- 先にchild handleを解放しても、親などが保持する内部参照があればobjectは残る。逆も同様。ツリーが所有する内部参照の細部はUI Tree仕様で定義する。
- Runtimeの公開参照を最後に解放する前に、UI threadで`shutdown`を完了する。`shutdown`は新規作業の受け付けを止め、未実行の投稿をキャンセルしてclosureを破棄し、購読とcallbackの無効化・破棄を完了させる。停止後のUI操作は`UI_CLOSED`を返す。
- Runtimeの停止後でも、残存するhandleの`release`とerror handleの参照操作は可能とする。残存handleは停止済みRuntimeの生存を内部的に保持するが、UI機能は使えない。
- 循環する内部参照を作らない。BindingやSubscriptionが必要とする参照方向は各サブシステム設計時に明示する。
- 各APIには、出力handleが所有参照か、入力handleが借用か、失敗時に出力値がどうなるかを記載する。

## 4. 文字列・配列

- 文字列はUTF-8の`(pointer, byte length)`。NUL終端を前提とせず、埋め込みNULはAPI別に許可可否を明記する。
- 入力pointerはその関数呼び出し中のみ借用する。非同期で必要ならCore側が呼び出し終了前にコピーする。
- 入力配列も`(pointer, element count)`で受け取り、要素型と要素の所有権をAPIごとに指定する。
- 出力文字列・配列は原則として呼び出し側が用意したbufferへコピーする。必要容量はbyte数または要素数で返し、実際に書いた長さを返す。bufferが小さい場合は不足を通知し、必要長を返す。
- 任意の長さがゼロならpointerはnullを許す。長さが非ゼロならpointerは有効でなければならない。
- callbackに一時的に渡す文字列・配列はcallbackが戻るまでだけ有効。保持するwrapperはその場でコピーする。

## 5. Threadとdispatcher

- Runtime作成を呼んだスレッドがUI thread。作成、shutdown、UI objectの参照・変更、購読登録・解除はそのスレッドに限定する。
- UI thread違反は`WRONG_THREAD`として返す。失敗した操作は状態を変更しない。
- `retain` / `release`、ABI照会、immutableなerrorの読み取り、dispatcherへの投入は任意のスレッドで呼べる。
- workerからのUI操作は`post(runtime, callback, userdata, destroy)`によりUI threadへ投入する。`post`は受け付け時点で戻り、実行完了を待たない。Runtimeはイベントループとの連携点を持ち、実行順は同一Runtimeへの受付順とする。
- UI thread上から`post`した場合も同じキューへ入れる。呼び出し中に即時実行しない。キューは一件ずつ処理し、同じRuntimeのUI callbackを並列実行しない。
- UI threadのイベントループ統合方式、負荷上限と公平性はPlatform仕様で定義する。async Commandの完了・キャンセル意味論はMVVM仕様へ送る。

## 6. Callbackと再入

- callback登録が成功した時点でCoreがclosureの所有権を引き受ける。登録が失敗した場合は引き受けず、`destroy`も呼ばない。
- Coreが所有したclosureについて`destroy(userdata)`を**ちょうど1回**、UI thread上で呼ぶ。通常の解除またはshutdown後、最後の実行が終了してから破棄する。
- 購読解除の成功後は新しいcallbackを開始しない。既に実行中のcallbackから自身を解除した場合は、callbackのreturn後にdestroyする。
- UI callback中に別のUI APIを呼ぶことは許す。その操作が生成する通知はキューへ積み、現在のcallbackへ再帰的に通知しない。通知キューの重複排除やBinding評価順はMVVM仕様で決める。
- callback内の例外は言語wrapperが捕捉し、ABI越しに伝播させない。戻り値がない通知callbackの例外はwrapperの診断経路へ記録する。Rust CoreもpanicがABIを越えないよう出口を封じる。
- `destroy`からのUI操作は許さない。wrapperは管理対象の言語参照を解除する用途に限定する。
- dispatcherに投入したclosureは、実行完了時または未実行のままshutdownした時にdestroyする。`post`失敗時の所有権は呼び出し側に残る。

## 7. Errorと失敗時の原子性

- public APIは`ui_status`を返す。失敗時に詳細がある場合は新しいerror handleを返す。メッセージはUTF-8、errorは不変で、任意スレッドで読み取れる。
- errorにはstatus、安定した数値コード、説明文、将来拡張可能なdomainを含める。人向け文言の完全一致をアプリの分岐条件にしない。
- 出力引数は成功時だけ有効とする。失敗時はhandleをゼロ、長さをゼロへ初期化する。ただしbuffer不足の必要容量とfeature照会結果は各APIで例外を明記する。
- API引数の検証失敗、型不一致、停止、スレッド違反の時に部分的な状態変更を残さない。実行中callbackの例外によるアプリ側副作用の巻き戻しは保証しない。
- error objectを作る余裕がない場合でも`ui_status`で失敗を返し、error出力をゼロにできる。

## 8. この段階での検証条件

1. 4言語から作成、retain、複数回の別参照のrelease、shutdownを同じ意味で実行できる。
2. 解放済み・異種・別Runtimeのhandle、null/length不整合を安全に拒否できる。既に解放した**同じトークン**の二重releaseを検出し、別の所有参照へ影響させない。
3. workerから直接UI操作すると`WRONG_THREAD`となり、postした操作はUI threadで実行される。
4. callbackの自己解除、shutdown時の未実行post、wrapper例外の各経路でuserdataが1回だけ破棄される。
5. buffer不足時に必要容量が得られ、文字列にNUL終端を要求しない。

## 9. 確認を求める判断

| ID | 提案 | 理由 |
|---|---|---|
| R1 | handleを非再利用の64-bit数値トークンにする | stale handleを検出し、Coreポインタを露出させない |
| R2 | Runtime作成スレッドをUI threadとし、worker releaseのみ遅延破棄する | 4言語で共通のスレッド規則にする |
| R3 | callbackの登録・解除・destroyをUI threadへ集める | GCや言語ランタイムの参照破棄点を決める |
| R4 | callback中のUI API呼び出しを許し、通知の再帰実行を抑える | CommandからViewModel更新を扱えるようにする |

## 10. 関連文書

- `design-overview.md`: スライスと全体責務
- `abi-spec.md`: Cの型、関数形、互換性案
- `mvvm-semantics.md`（後続）: 更新順、循環、validation、async Command
- `platform-rendering.md`（後続）: UI event loop統合
