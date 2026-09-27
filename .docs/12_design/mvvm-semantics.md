# MVVM意味論 — Property・Binding・Command・Validation（Slice 2）

- 状態: P1～P4、B1～B4、C1～C4、D1～D4、V1～V8、BD-03承認済み（2026-09-26）
- 作成日: 2026-09-26
- 上位文書: `mvvm-ui-library-requirements.md`、`design-overview.md`、`runtime-contract.md`
- 対象: Rust Coreが管理するObservable Propertyの値と変更通知

## 1. 範囲

第1段階ではPropertyと変更通知、第2段階ではBindingの基本意味論、第3段階では非同期Command、第4段階では変換のないBindingグラフの制約、第5段階では同期Validation（V1～V4）とView入力規則（V5～V8）を定めた。Coreが管理する状態の共通意味論であり、C ABIの具体的な関数形や各言語の型表現はまだ定めない。

依存Property、値変換と非同期検証、言語側で定義したViewModelの接続方法、コレクション通知は後続の段階で定める。Binding、Command、Validationの状態通知も本書のProperty通知規則に従う。

## 2. Propertyとスレッド

- Propertyは所属Runtime、保持する値、値の種類、変更判定規則を持つ。Coreが管理するViewModelのPropertyとUI要素のPropertyには同じ基本規則を適用する。
- 作成、読み取り、設定、購読、購読解除は所属RuntimeのUI threadで行う。worker上の計算結果を反映する場合は`runtime_post`でUI threadへ渡してから設定する。workerからの直接操作は`WRONG_THREAD`となり、値も通知も変えない。
- `retain` / `release`など、承認済みRuntime契約のスレッド例外はそのまま適用する。Property handleを保持できることはworkerから値を読めることを意味しない。
- setterの検証や値のコピーが失敗した場合は値を変更せず、通知も生成しない。setterの成功は値がCoreで確定したことを表し、購読callbackの実行完了までは保証しない。

## 3. 値の変更判定

- 各Propertyには宣言時に固定される値の種類があり、異なる種類の値は設定できない。setterで新しい値が現在値と等しい場合は成功するが、変更通知を生成しない。
- `bool`と整数は値、文字列はUTF-8の内容で比較する。object参照を値として扱う場合は、公開handleのトークン番号ではなく、同じCore objectかどうかで比較する。
- 浮動小数点値（NaNと符号付きゼロを含む）、カスタム値、null許容、文字列正規化、mutable objectの内部変更を変更判定へ反映する方法は、値型とABIの設計時に定める。ここでは異なる型を暗黙に変換しない。
- 同じ値を再設定しても通知されないため、「再読込」など値が同じでもイベントを発生させたい用途はPropertyの変更通知に依存しない。

## 4. 通知の配信（承認済み）

1. UI thread上のsetterは値を同期的に確定し、通知対象のPropertyをキューに記録して戻る。通知callbackをsetterの呼び出しスタック上で実行しない。
2. Runtimeは現在の外側のUI callback/API処理が戻った後、通常は次のUIイベントまたはdispatcher投稿の前に保留中の通知を配信する。連続配信がBD-03の有限作業枠に達した場合は、完了した配信回の境界でUIイベントループへ制御を返し、未配信分を保持して後続の処理区間で再開する。UI threadが処理中なら、その処理が戻るまで待つ。通知キューは同じRuntimeのUI threadでのみ処理する。
3. 一度の処理区間で同じPropertyが複数回変わった場合、保留中の通知は一件へまとめる。通知は変更のあったPropertyについて、その区間で最初にキューへ入った順に配信する。通知は「変更された」ことを伝え、旧値・中間値の履歴は保持しない。購読者は配信時点でPropertyを読み、現在値を得る。
4. 一つのPropertyへの購読者は登録順に呼ぶ。通知開始時点の購読者を対象とし、途中で解除された購読者は以後呼ばない。途中で追加された購読者は次回以降の通知から対象とする。
5. 通知callbackから別のPropertyや同じPropertyを更新できる。その変更通知は現在の通知へ再帰的に割り込まず、次の配信回へ送る。連続配信はBD-03の有限作業枠で区切り、非収束と判断された通知連鎖は診断して停止する。数値上限・検出方式・復旧APIは詳細設計で決める。

「処理区間」はUIイベントcallback、`runtime_post`のcallback、または外側のUI API呼出しが実行中の期間を指す。これらが入れ子になっても最も外側の処理が戻るまで通知を配信しない。通知callback内からの明示的な同期フラッシュAPIは本段階では設けない。

### 例

一つのUIイベント内で`Progress = 10`、続けて`Progress = 20`と設定した場合、2回目のsetterから戻った時点でgetterは20を返す。イベントcallbackが戻った後に通知は一件届き、そのcallbackから読める値は20である。workerからこの更新を行う場合は、値20を運ぶclosureを`runtime_post`し、UI threadのcallback内でsetterを呼ぶ。

## 5. 購読と終了

- 購読はSubscription handleを返す。購読時に現在値の通知を自動送信しない。初期値が必要な購読者はUI threadでgetterを呼ぶ。購読開始後の更新が必要なBindingは、その初期同期の手順を別途定める。
- 購読解除後に新しい通知callbackを開始しない。通知中の自己解除、`destroy`の実行スレッドと回数は`runtime-contract.md`に従う。
- `shutdown`時には保留中のProperty通知を配信しない。購読を無効化し、closureをRuntime契約に従って破棄する。停止後のProperty APIは`UI_CLOSED`を返す。
- 一つの購読者のcallbackで言語側の例外が起きても、wrapperが捕捉して診断経路へ記録する。他の購読者の配信と値の確定は取り消さない。

## 6. 検証する振る舞い

1. UI threadのsetter直後にgetterは新値を返す。workerからのsetterは`WRONG_THREAD`で値を変えず、`post`経由では更新できる。
2. 等しい値の再設定は通知なし。異なる値を同じイベント中に複数回設定しても通知は一件で、購読者は最終値を読む。
3. 二つのPropertyを更新した場合、最初に変更されたPropertyから通知される。同じPropertyの購読者は登録順に呼ばれる。
4. 通知callback中の設定は再帰的な通知を起こさず次の配信回に回る。自己解除後は再度呼ばれない。
5. 通知待ちの状態で`shutdown`した場合、callbackは開始されず、closureは一度だけ破棄される。

## 7. 承認された設計判断（2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| P1 | Core管理Propertyの読み書き・購読をUI threadに限定し、workerからは`post`を使う | 4言語wrapperのViewModel更新経路 |
| P2 | setterで値を即時確定し、等価なら通知しない | getterの観測値と更新コスト |
| P3 | 通知は外側の処理終了後に配信し、同一Propertyの複数更新を一件へまとめる。BD-03により有限作業枠に達した後は完了した配信回でUIへ譲り、残りを保持して再開する | 再入、進捗表示、中間値の観測、UI応答性 |
| P4 | 購読時の初期通知なし、解除後の新規callbackなし、shutdown時の保留通知は破棄 | Bindingの初期同期とlifetime |

## 8. 第2段階：Bindingの基本意味論（承認済み）

### 8.1 対象と更新方向

- Bindingは同一Runtimeに所属する、型が適合する二つのCore管理Propertyを接続する。作成・解除はUI threadのみ。型変換・異なるRuntime・言語側ViewModelの接続は本段階では対象外とする。
- `OneTime`は作成時にsourceの現在値をtargetへ一度コピーし、その後はどちらの変更も伝播しない。作成結果として恒久的な購読を保持しない。
- `OneWay`は作成時にsourceからtargetへコピーし、その後のsource変更をtargetへ反映する。targetへの直接変更はsourceへ戻さず、次にsourceが変わるまではtargetの直接変更を保持する。
- `TwoWay`は作成時にsourceを優先してtargetへコピーし、その後はいずれの側の直接変更も相手へ反映する。初期同期の方向はBinding作成時に指定したsourceからtargetに固定する。
- 初期同期時に値型不一致や値の設定失敗があればBinding作成全体を失敗させ、targetの値も購読も変更しない。成功時の初期同期はsetterと同じ変更判定を行い、必要なら通常のProperty通知を一件キューへ入れる。購読時の自動初期通知は行わないというP4と整合する。

### 8.2 更新タイミングと観測

- 直接のsetterが成功した直後には、そのProperty自身のgetterは新値を返す。Binding先への反映は現在の外側のUI処理が戻った後、Property通知の配信段階で行う。したがって、同じUI callback内で相手側のgetterを呼ぶと旧値が見える場合がある。
- 一つの処理区間でsourceが複数回変わった場合、最終値だけを伝播する。Coreは各配信回で、Bindingによる更新を外部の購読callbackより先に処理する。その配信回で伝播したtargetの通知は、再帰させず次の配信回へ回す。
- `TwoWay`の伝播が発生させた書き込みには更新元を付け、同じBindingを逆向きに再伝播させない。一つの処理区間で両端をアプリが直接設定した場合は、最後に成功した直接設定を優先し、その値を相手へ反映する。一般の複数Bindingをまたぐ循環と、変換が入る場合の収束は後続設計で扱う。
- workerからの設定はP1に従い、`post`したUI callback内で行う。Bindingはworkerで動作させない。

### 8.3 解除と所有権

- `OneWay`と`TwoWay`の作成はBinding handleを返す。明示的な解除後は新しい伝播を行わない。解除時点の両Propertyの値は保持し、以前の値へ戻さない。通知待ちの変更も、そのBinding経由では以後伝播させない。
- Bindingが参照するPropertyの所有関係は循環参照を生まないよう設計し、具体的な強参照・弱参照の方向とツリー破棄との関係はlifetime設計で確定する。Runtimeの`shutdown`ではBindingを無効化し、保留中の伝播を破棄する。
- `OneTime`は初期コピーの後にBinding handleを保持しない。OneWay / TwoWayのhandleを`release`するだけで自動解除するかは、意図しない更新停止を避けるため、lifetime設計で別途決める。明示解除は必ず提供する。

### 例

`ViewModel.Message`をsource、`Text.Content`をtargetとして`OneWay`を作成する。作成時に`Message`の値を`Content`へ同期する。UI callbackで`Message`をA、続けてBへ設定した場合、そのcallback内で読む`Message`はB、`Content`は旧値の場合がある。callback終了後の通知処理で`Content`はBとなる。Bindingを解除した後に`Message`をCへ設定しても`Content`はBのままである。

### 8.4 検証する振る舞い

1. `OneTime`は作成時の一度だけ同期し、後続の変更には反応しない。
2. `OneWay`は初期同期後、sourceの最終値のみをtargetへ反映し、targetの直接変更をsourceへ戻さない。
3. `TwoWay`は初期同期でsourceを優先し、片側の直接変更をもう一方へ伝える。同じ処理区間で両側が直接変更された場合は最後の直接設定を優先し、反射による無限往復を起こさない。
4. 外部購読者がsource変更を受け取る時点で、同じ配信回のBinding伝播は完了している。伝播されたtargetの変更通知は次の配信回に届く。
5. 解除後の変更は伝播せず、両Propertyの値は保持される。型不一致の作成失敗では値も購読も変えない。

## 9. 承認された設計判断（Binding、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| B1 | 初期同期はsourceを優先し、OneTime / OneWay / TwoWayの方向を8.1のとおり定める | 初期表示とTextBoxのTwoWay接続 |
| B2 | 伝播は外側のUI処理後、外部通知より前に実行し、中間値を伝えない | callback内での観測と表示更新 |
| B3 | TwoWayは更新元を追跡し、両端の直接変更は最後の成功した設定を優先する | 反射抑制と競合時の値 |
| B4 | 明示解除で以後の伝播を止め、現在値は保持する | 画面破棄と購読管理 |

## 10. 第3段階：非同期Commandの基本契約（承認済み）

### 10.1 実行と状態

- Commandの`execute`はUI threadで呼ぶ。非同期Commandの実行要求はworkerやホスト言語の非同期タスクを開始し、UI callbackを待たせずに戻る。Coreの`runtime_post`はUI更新用であり、worker自体はアプリまたは言語wrapperが管理する。
- 実行ごとにoperation IDを発行し、`Idle → Running → Succeeded / Failed / Cancelled`の状態を持つ。`Running → Cancelling → Cancelled / Failed / Succeeded`も許す。`Cancelling`はキャンセル要求済みで、workerの終了をまだ確認していない状態である。終端後の新しい実行では新しいoperation IDを使う。
- Commandの`can_execute`はアプリの実行条件と実行状態で決まり、既定では`Running`と`Cancelling`の間はfalseとする。実行不可のときの`execute`は失敗を返し、新しいoperationを開始しない。状態と実行可否の変化はUI thread上のProperty通知として観測できる。
- 実行中にもう一度`execute`しても、既定では二重起動しない。キューイングや複数同時実行は別のCommand種別または明示オプションとして後続設計で検討する。

### 10.2 進捗・完了・失敗

- workerはUI objectを直接操作しない。進捗値や完了結果を`runtime_post`でUI threadへ送る。投稿されたUI callbackはoperation IDを確認し、現在の実行に属する更新だけをCommandの状態やViewModel Propertyに反映する。画面側はProgressBarやメッセージ表示をこれらのPropertyへBindingできる。
- 進捗は実行中だけ受け付け、全サンプルの配信を保証しない。同一operationの未配信進捗は新しい値で置き換えて集約してよい。完了・失敗・キャンセルの終端結果は進捗値から独立して保持し、一度だけ適用する。UI threadで終端結果を先に適用した後の同一operationの進捗は破棄し、進捗が先ならその値を反映してから終端状態へ遷移する。過去operationの通知は無視する（BD-26）。具体的な集約場所、wake-up抑制、頻度・時間閾値、進捗型は後続詳細設計で定める。
- 完了時はUI threadで一度だけ終端状態へ遷移し、成功結果または失敗情報を記録する。worker側の例外やpanicをABI越しに伝播させず、wrapperまたはCoreの境界で失敗として変換する。失敗は診断情報とともに観測でき、画面へのエラー表示はアプリ側が選べる。
- `post`は実行を保証するものではない。投稿前にRuntimeが停止した場合の`UI_CLOSED`、投稿後のshutdownによる未実行closureの破棄はSlice 1の契約に従い、画面への進捗・完了表示を保証しない。

### 10.3 キャンセルと画面終了

- `cancel`はUI threadで要求し、`Running`なら`Cancelling`へ移る。workerへはスレッド間で安全なキャンセル信号を渡す。キャンセルは協調的であり、`cancel`の戻り時点でworkerが停止したとは限らない。同じoperationへの重複キャンセル要求は成功して状態を変えない。
- キャンセル後にworkerが正常終了した場合は`Succeeded`、失敗した場合は`Failed`、キャンセルを確認して中断した場合は`Cancelled`とする。実際に何が起きたかを終端状態で表し、キャンセル要求だけで結果を`Cancelled`へ決めない。
- 画面が閉じられたときは、画面が所有するoperationへキャンセルを要求し、画面向けの購読とBindingを解除する。アプリが画面外で所有するoperationは継続できる。画面の閉鎖はworkerの終了を同期的に待たず、画面破棄後に届いた結果は、その画面に反映しない。所有区分を表す具体的なAPIはlifetime設計で定める。
- Runtimeの`shutdown`は新たな投稿を止め、保留中のUI callbackを破棄する。進行中のworkerにキャンセルを要求するが、workerの終了をUI thread上で待たない。wrapperはworker終了まで必要な所有参照とclosureを保持・解放し、停止後のUI objectへアクセスしない。このため、shutdown後にUI上で終端状態を観測できるとは保証しない。

### 例

ButtonからSave Commandを実行すると、Commandは`Running`になり、Buttonの実行可否がfalseになる。workerは保存処理中の進捗を投稿し、UI threadで`Progress` Propertyを更新する。ユーザーがキャンセルすると`Cancelling`になる。workerがキャンセルを確認して終了したら`Cancelled`へ遷移する。画面が先に閉じられた場合はUIを待たせず、以後その画面を更新しない。

### 10.4 検証する振る舞い

1. `execute`はUI callbackを長時間占有せず、実行中の二重起動を拒否する。
2. workerから直接UI Propertyを設定すると`WRONG_THREAD`になり、`post`経由の進捗はUI threadで反映される。
3. キャンセル要求後でもworkerが正常完了した場合は`Succeeded`となり、キャンセル確認済みの場合だけ`Cancelled`となる。
4. 過去のoperation IDを持つ遅延通知や、完了後の進捗は現在の状態を変更しない。
5. 画面を閉じてもUI threadはworkerを待たず、後着の結果が閉じた画面を更新しない。shutdown時に未実行postのclosureは一度だけ破棄される。

## 11. 承認された設計判断（非同期Command、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| C1 | UI threadで起動してworkerへ委譲し、実行中の二重起動を既定で拒否する | Buttonの実行可否、UIの応答性 |
| C2 | 進捗・完了を`post`とoperation IDでUIへ反映し、終端状態は一度だけ確定する | 古い通知、エラー表示、進捗表示 |
| C3 | キャンセルは協調的な要求で、実際の終了結果を終端状態にする | キャンセル後の成功・失敗の見え方 |
| C4 | 画面終了は画面所有の処理へキャンセルを要求し、shutdownは全処理へ要求する。どちらもUI threadでworkerを待たず、後着の結果を閉じた画面へ反映しない | lifetimeと終了時の応答性 |

### BD-26 Async Command進捗配信（承認済み、2026-09-27）

- 中間進捗は最新値へ集約でき、各サンプルの配信は保証しない。業務上すべての更新を記録する用途には進捗通知を使わず、アプリ側の記録を使う。
- 完了・失敗・キャンセルは進捗値と独立した終端結果として一度だけ適用する。終端適用後の同一operation進捗および過去operationの通知を破棄する。
- 競合時はRuntimeのUI thread上でのcallback処理順序とoperation IDに従う。集約場所、wake-up抑制、頻度・時間閾値、型と値域、複数channelは詳細設計に残す。

## 12. 第4段階：Bindingグラフの制約（承認済み）

本段階は、変換のない同型Property同士のOneWay / TwoWay接続だけを対象にする。OneTimeは作成後に接続を保持しないためグラフに含めない。演算付きの依存Property、値変換、Validation、および異なるRuntime間の接続にはまだ適用しない。

### 12.1 接続と競合

- TwoWayで直接・間接につながったPropertyを一つの同期グループとして扱う。グループの任意のPropertyへの直接設定は、グループ内の他のPropertyへ最終値を伝える。一処理区間に複数の直接設定があれば、最後に成功した直接設定がそのグループ内で勝つ。個々のsetter直後のgetterはP2に従い、伝播はB2に従って処理終了後に行う。
- OneWayはグループから別のグループへの有向接続となる。各グループが受け取れるOneWayの入力元は最大一グループとする。同一入力元から複数の表示先へ分岐することは許す。複数の入力元を一つのtargetへ接続したい場合は、合成値を計算する単一Propertyをアプリ側で作り、そのPropertyを入力元にする。
- OneWayの入力先グループへアプリが直接設定した場合はB1どおり一時的にその値を保持し、次の入力元の変更時に入力元から上書きする。同じ処理区間に入力元と入力先の直接設定があった場合は、入力元からの伝播を優先する。この点はTwoWayグループ内で最後の直接設定を優先する規則と区別する。

### 12.2 循環と更新順

- OneWayの接続は同期グループ間で有向循環を作れない。グループ内へのOneWay接続、同じ向きの重複接続、入力元が二つになる接続は作成時に拒否する。TwoWayでグループを結合した結果、OneWayの自己循環・有向循環・複数入力が生じる場合も拒否する。
- 外側のUI処理が戻った後、変更のあった同期グループ内で値を揃え、OneWayの行き先へ進む。循環のない方向に伝播し、その配信回に処理対象となったPropertyに関する外部通知は、対応するBinding伝播の後に実行する。伝播で新たに変更されたPropertyの通知はB2どおり次の配信回とする。
- Property通知callbackによる新たな直接設定は次の配信回の入力とする。callbackが値を交互に変更し続ける場合はグラフの静的な循環制限だけでは止まらないため、配信回の上限・診断・イベントループへの譲り方は後続のRuntime/診断設計で確定する。

### 12.3 グラフ変更の原子性

- Binding作成は接続規則と型を検証し、初期同期に必要な値の確保を済ませてから、接続と初期値をまとめて反映する。検証または確保に失敗した場合は、グラフ・値・通知を変えない。解除もUI thread上で接続を外し、以後の伝播を止める。既存値を巻き戻さない。
- グラフを変更した処理区間で既に保留している更新は、解除済みの接続へ流さない。新しく接続したBindingの初期同期はB1に従い、保留中の変更がある場合は作成時点でのsource getterの値を使う。グラフ変更後の通常の通知で同じ値を二重に伝えない。

### 例

`ViewModel.Name`と`TextBox.Value`をTwoWayで結び、その同期グループから`Text.Content`へOneWayで分岐できる。`Text.Content`へ別のsourceからOneWayを追加する操作は、入力元が二つになるため失敗する。`Text.Content`から`ViewModel.Name`へ戻るOneWayも循環するため失敗する。

### 12.4 検証する振る舞い

1. TwoWayの二辺以上のグループでは、同じ処理区間の最後の直接設定が全メンバーへ伝播する。
2. OneWayの分岐は許し、同じtargetグループへの二つ目の入力および有向循環は作成時に拒否する。失敗後の値とグラフは元のままである。
3. 入力先を直接変更しても入力元へ戻らず、その後の入力元変更で上書きされる。同じ処理区間に両端を変更した場合はOneWayの入力元が勝つ。
4. Bindingを解除した後の保留中の更新はその接続へ流れない。通知callbackで作った新しい変更は再帰配信されない。

## 13. 承認された設計判断（Bindingグラフ、2026-09-26）

| ID | 承認済みの判断 | 主な影響 |
|---|---|---|
| D1 | TwoWayの連結Propertyを同期グループとし、最後の直接設定を優先する | 連鎖した双方向編集の値 |
| D2 | OneWayはグループごとに入力元を一つに制限し、循環を作る接続を拒否する | 複数sourceの表現方法と循環防止 |
| D3 | 同一区間でOneWayの両端を直接変更した場合は入力元を優先し、伝播後に外部通知を配信する | 競合時の表示と通知順 |
| D4 | Bindingの作成失敗は値とグラフを変えず、解除後は保留更新も伝播させない | 設定時の失敗とlifetime |

### BD-03 Runtime通知上限・診断（承認済み、2026-09-26）

- Runtime管理下の連続Binding伝播・通知配信には有限の作業枠を設ける。具体的な上限値・計測点は詳細設計で定める。
- 作業枠に達しただけでは通知を捨てず、循環とも断定しない。Binding伝播および配信回の途中では中断せず、完了した配信回の後でUIイベントループへ制御を返し、保留分を後続のUI処理区間で配信する。このとき次の通常UIイベントやdispatcher投稿が保留通知より先に実行され得る。
- 大量だが有限の更新と非収束反復を区別する。非収束と判断した場合は原因となる通知連鎖を停止し、原因・影響範囲・停止状態を診断する。他の連鎖を巻き添えにせず、確定済みのProperty値を巻き戻さない。検出・復旧の詳細は後続設計で定める。
- 任意のcallbackが将来収束するかを一般に確実には判定できないため、非収束を立証できない長期反復の停止は保証しない。UIへの譲りと長期保留の診断を適用する。
- 静的解析で対象ControlとPropertyを解決できる言語・箇所では、イベントcallback内でイベント元と同じControlのPropertyにアクセスする場合に警告する。読み取りと書き込みを区別し、同じPropertyへの書き戻しを強い警告とする。警告は循環可能性の提示であり、コンパイルエラーや無限反復の断定ではない。Pythonはlibrary compilerのlint/type check（BD-15）で型・Control identityを解決できる場合に対象とする。dynamic dispatchやreflection等で解決できないPythonコードの網羅的検出は保証しない。Runtime保護・diagnosticsは全言語で共通とする。

## 14. 第5段階：同期ValidationとView入力規則（承認済み）

本段階では、Viewの入力操作に対する軽量な検査と、同じ型のProperty間のBindingにおける同期検証を対象にする。型変換、非同期検証、サーバー検証、複数フィールドをまたぐ検証の再評価は後続の設計事項とする。

14.1～14.3のProperty/Binding検証はV1～V4、14.0を含むView入力規則はV5～V8として承認済みである。

### 14.0 責務の区別（追加提案）

| 層 | 対象 | 例 | 拒否時の扱い |
|---|---|---|---|
| View入力Validator | 入力操作で作る編集候補。UI threadで短時間に評価する | 整数の文字種、禁止文字、時刻の形式、一覧からの選択 | 編集候補を拒否するか、未完成の入力としてViewだけに保持する |
| ViewModelのProperty Validator | アプリが受け取る候補値。プログラムからのsetterにも適用する | 業務上の範囲、権限、必須項目、アプリ固有の整合性 | ViewModelの値を変えず、検証失敗を返す |

View入力Validatorは操作性と早期フィードバックを目的とし、アプリが受け取る値の唯一の防御境界にはしない。Viewを通らない値の設定もあるため、アプリの不変条件はViewModel側で検証する。

- View入力Validatorは、キーボード・貼り付け・ドロップ・アクセシビリティ経由の編集など、ユーザー起点の編集候補に同じ規則を適用する。TextBoxでは単一キーだけを調べず、変更後の候補文字列と選択範囲を評価する。選択Controlでは候補の項目を評価する。プログラムからのProperty設定にはView入力Validatorを暗黙に適用しない。
- 候補に対する判定は`Accept`（入力値として成立）、`Incomplete`（入力途中として許す）、`Reject`（その編集を採用しない）の三値とする。`Accept`はViewの値を確定して通常のBindingへ流す。`Incomplete`はViewの編集中の値として表示するが、ViewModelへの伝播を止める。`Reject`はViewの値・caret・選択範囲をその編集前の状態に保ち、値の変更通知を出さない。View側の入力状態は別途通知できる。
- `Incomplete`は、負号だけの整数入力や入力途中の時刻などに使う。未完成の文字列を即座に失わず、ViewModelには最後に受理された値を保持させる。`Accept`になったら同じ編集系列のView側入力エラーを消し、Bindingへ流す。`Reject`が続く場合も、入力方法に応じた理由をView側へ表示できる。
- IMEの変換中の文字列は確定したProperty値として扱わず、変換中にValidatorで候補を破棄しない。IME確定時に候補を評価する。確定候補が`Reject`となる場合は入力を黙って捨てず、`Incomplete`相当の編集値として保持して理由を表示し、ViewModelへは流さない。IME確定・Undo/Redo・選択操作との詳細な状態遷移は`input-focus-ime.md`とControl仕様で定める。
- 一覧内の選択制限は現在の候補集合に基づくView規則として扱える。候補集合の変更後の再評価、および複数Controlに共通する入力マスク・ロケール依存の時刻表記はControl/Collection仕様で定める。

### 14.1 検証と値の確定

- Propertyには任意の検証規則を付けられる。setterはUI thread上で候補値を検証し、有効ならP2どおり値を確定する。無効ならそのPropertyの値を変更せず、値の変更通知も生成しない。呼び出し元には通常のAPI引数エラーとは区別できる検証失敗と、機械可読のエラーコードおよび表示用メッセージを返す。具体的なABI表現は別途決める。
- 検証規則はUI threadで同期的かつ短時間に評価する。wrapperから渡す検証callbackの例外はABIを越えさせず、検証処理そのものの失敗として扱う。検証処理の失敗は入力が無効であるという判定とは区別して記録する。
- Binding経由の検証が成功し、値が現在値と等しくても、そのBindingの以前のエラーは消せる。この場合、値の変更通知は出さず、Bindingのエラー状態だけ変更通知を出す。直接のsetterの検証失敗は呼び出し元へ返し、本段階ではPropertyに永続的なエラー履歴を作らない。

### 14.2 Bindingでの失敗とエラー状態

- Bindingの伝播先が値を拒否した場合、伝播元に確定した値は巻き戻さない。伝播先は以前の値を保ち、Bindingのエラー状態に、拒否したProperty・方向・エラーコード・メッセージを記録する。失敗した伝播は逆方向へ反射させない。
- TwoWayの入力UI Propertyが文字列を受け入れ、ViewModel Propertyがその文字列を拒否した場合、TextBoxの入力は残り、ViewModelは最後に受理した値を保つ。この間はD1の同期グループに検証失敗による値の不一致がある。View入力Validatorが`Incomplete`とした場合も、Viewの編集値だけが先行する。次の有効な直接設定で再び同期し、Bindingエラーを消す。無効入力中に他の経路からViewModelが更新された場合は、通常のTwoWay伝播に従いTextBoxの入力が上書きされる。
- エラー状態は値とは別に観測でき、変化時はP3の通知規則でUI thread上に通知する。アプリはこの状態を入力欄のエラー表示などに結び付けられる。UIがエラーをどう描画するかはControl仕様で決める。
- 同じ処理区間で複数の伝播失敗が起きた場合、Bindingごとに最後の失敗を現在のエラー状態とする。エラーの履歴や複数の同時表示メッセージは本段階で定義しない。

### 14.3 初期同期・解除・終了

- Bindingの作成時にtargetの検証が失敗した場合は、D4の原子性に従ってBindingを作らず、targetの値も既存のエラー状態も変えない。作成失敗の理由を呼び出し元へ返す。
- Bindingを明示解除すると、そのBindingのエラー状態も終了する。各Propertyに別途登録された検証規則と、そのProperty自身の値は残す。`shutdown`時は保留中の検証結果通知を破棄する。
- 検証規則を後から差し替える場合の既存値の再検証と、異なるBindingから同じPropertyへ届くエラーの集約は後続設計で扱う。

### 例

`TextBox.Value`と`ViewModel.Name`をTwoWayで結び、Nameに「空文字列は無効」という規則を付ける。ユーザーがValueを空文字列にするとValueは空文字列のまま、Nameは以前の文字列を保ち、Bindingのエラー状態に検証失敗が入る。次に有効な文字列を入力するとNameへ反映され、エラー状態は消える。

時刻入力欄で`12:`まで入力したときは、View入力Validatorが`Incomplete`を返し、その文字列を画面には残すがViewModelの時刻は変えない。`12:30`となり`Accept`を返した時点でTwoWayの更新を再開する。禁止文字の貼り付けを`Reject`した場合は値を変えず、その貼り付けの理由をView側へ表示できる。

### 14.4 検証する振る舞い

1. 直接のsetterが無効値を拒否した場合、Propertyの値と値の通知は変わらず、検証失敗が呼び出し元へ返る。
2. TwoWayで入力側は受け入れ、モデル側が拒否した場合、入力値を保持し、モデル値を維持し、Bindingエラーを通知する。次の有効入力で値が再同期してエラーが消える。
3. 入力が無効な間にモデル側を直接更新すると、通常の伝播で入力側が更新される。Binding作成時の検証失敗は既存の値・グラフ・エラー状態を変えない。
4. 検証callback自体の異常は通常の無効入力と区別され、ABI境界を越えない。解除後にはそのBindingのエラー通知が新しく始まらない。
5. 整数欄の途中の負号や時刻欄の`12:`はViewに残り、ViewModelへは流れない。禁止文字の貼り付けは編集前の値と選択状態を保持する。
6. IME変換中の文字列は候補として破棄されず、確定後に無効と分かった入力は理由を示してView側に保持する。プログラムからのsetterはView入力Validatorを経由せず、ViewModel側の検証は経由する。

## 15. Validationの判断

### 承認済み（2026-09-26）

| ID | 判断 | 主な影響 |
|---|---|---|
| V1 | Propertyの同期検証に失敗したsetterは値を変えず、検証失敗を通常のAPIエラーと区別する | モデル値と呼び出し元のエラー処理 |
| V2 | Binding伝播の検証失敗では入力元を巻き戻さず、Bindingに独立したエラー状態を持たせる | TextBoxの無効入力表示 |
| V3 | TwoWayは検証失敗中に一時的な値の不一致を許し、次の有効な更新で同期する | D1の例外と再同期の挙動 |
| V4 | Binding作成時の検証失敗は原子的に拒否し、解除時にはそのBindingのエラー状態を終了する | 初期同期・lifetime |

### 承認済み（View入力、2026-09-26）

| ID | 判断 | 主な影響 |
|---|---|---|
| V5 | Viewに軽量な入力Validatorを置き、ユーザー操作の編集候補を`Accept / Incomplete / Reject`で判定する | 入力制限と途中入力の操作性 |
| V6 | `Incomplete`はViewの編集値に保持し、ViewModelへのTwoWay伝播を一時停止する | 編集中の時刻・整数と確定値の区別 |
| V7 | IME変換中は破棄せず、確定時に拒否される入力も理由とともにViewで保持する | 多言語入力と入力データの保持 |
| V8 | Viewの規則を通らない設定もあるため、アプリの不変条件はViewModelで検証する | 入力経路間の一貫性 |

## 16. 次に設計する項目

- Propertyと各言語で定義されたViewModelの接続契約、値型とABIの型表現。
- 値変換・非同期検証、依存Property、通知callbackが連続更新する場合の上限と診断、Binding handleの最終所有者。
- 非同期Commandの具体的なABI・wrapper、workerの所有と生成失敗、進捗の間引きと順序保証。
- 言語側ViewModelの検証callback、エラー表現と4言語wrapperの契約。
- View入力状態とBindingエラー状態の表示優先順位、IME・Undo/Redoの詳細。
