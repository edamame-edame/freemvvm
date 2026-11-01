# 0.0.1 Input / Focus event pathの次工程計画

- 計画版: 0.2
- 状態: 計画0.2全文承認済み（利用者、2026-11-01）。unit inventory登録・実装branch準備・実装は別管理
- 対象version: `0.0.1`
- 対象: 承認済みInput/Focus要求を、Core UI処理とWindows native event sourceへつなぐ前の設計・実装unit計画
- Core APIs branch案: `impl-rust/core-input-focus`
- Windows Platform子branch案: `impl-rust/platform-adapter-windows-input`
- 関連計画: [0.0.1 release計画](release-0.0.1.md)、[Windows Platform Adapter計画](windows-platform-adapter.md)

## 背景

Windows close-request接続までのCore/Windows unitは同revision CIを通過し、PR #4でCore APIs branchへ統合された。現行Coreには`FocusManager`とFocus request/Window activation状態があるが、承認済みInput/Focus DDの`InputRouter`とroute dispatchは実装されていない。Windows `WndProc`も現在は`WM_CLOSE`、`WM_SIZE`、`WM_PAINT`を扱い、keyboard/text/pointer eventをCoreへ渡していない。

一方、承認済みDDには実装前に具体化が必要な境界が残る。Input event field/ABI表現とpublic focus event APIは未決とされ、RQ-09-068もpointer event種類の一部を候補扱いにしている。ここを既存の承認済み範囲だけで実装できるか先に確定する。

## 根拠

- 要求: [RQ-09 UI](../11_requirements/RQ-09_ui-scope.md)（keyboard/focus/input routing）、[RQ-06 threading/dispatch](../11_requirements/RQ-06_threading-dispatch.md)
- DD: [UI InputとFocus](../13_detail_design/ui/input-focus.md) 1.1全文承認済み、[Platform抽象化](../13_detail_design/platform/abstraction.md) 1.2全文承認済み、[Platform内部層](../13_detail_design/platform/internal-layers.md) 2.2全文承認済み
- 試験設計: [UI unit test](../32_unit_test/ui.md)（`UT-INPUT-001`、`UT-FOCUS-001`、`UT-FOCUS-002`）
- 実装状況: `crates/core-runtime/src/ui_focus.rs`はFocus状態管理、`crates/platform-windows/src/win32.rs`はWindow message workerを持つ。InputRouterおよびWindows入力event接続は未実装

## 推奨方針

設計境界を先に閉じる。照合の結果、RQ-09-068のhandled/bubblingを利用者が指定するにはInput event subscriptionとevent payloadを公開する必要があるため、既存公開ABIの後方互換追加として別featureを提案する。Input/Focus、Controls、共通ABI、unit-test、Windows internal-layer文書へ未承認改訂案を作成した。これらの文書承認後に実装unitを再計画し、Core API/C ABIとRust・C++・C#・Python wrapperを言語別子branch、Windows AdapterをPlatform child branchとして分割する。設計案の承認と実装開始承認は分ける。

特定keyの割当、Tab traversal、shortcut/action binding、mouse capture、未承認pointer event種別、IME compositionのOS固有詳細を実装都合で追加しない。実装可能範囲を確定できない場合はsource変更前にレビューへ戻す。

## 作業分割案

| unit | 責務 | 完了条件 |
| --- | --- | --- |
| 設計文書レビュー | Input/Focus、Controls、C ABI、unit test、Platform internal layersの改訂案を文書全体でレビューする | API・payload・既定動作・callback寿命・対象event・除外eventが承認される。承認前はunit inventory/実装を開始しない |
| `UNIT-CORE-091`候補（Core APIs） | private InputRouter/neutral event dispatch、Widget subscription、borrowed event ABI、Focus getter、handled/default action、pending Button pressを接続する | UT-INPUT/FOCUS追加案をfake eventで検証。Core/workspace gates・Rust minimum・文書検査が同revisionで成功 |
| Rust wrapper候補 | safe borrowed InputEvent view、closure subscriptionとcancel/lifetimeをCore C ABIへ対応付ける | Rust APIとdestroy/cancel/panic経路をwrapper unit testで検証 |
| C++ wrapper候補 | non-throw callback/event facadeとsubscription RAII/manual cancel境界をCore C ABIへ対応付ける | C++ callback exception containment・lifetime・public sampleをwrapper unit testで検証 |
| CSharp wrapper候補 | delegate root、InputEvent view、subscription dispose/cancelをCore C ABIへ対応付ける | delegate/userdata root、callback exception、dispose後destroyをwrapper unit testで検証 |
| Python wrapper候補 | callable root、borrowed event facade、subscription cancel/lifetimeをCore C ABIへ対応付ける | GIL/callback exception/GC root/destroyをwrapper unit testで検証 |
| `UNIT-PLATFORM-WINDOWS-007`候補 | Win32 workerからkey/text/primary pointer/focus eventを値copyし、generation付きsink経由でCore UI dispatcherへ順序付き投入 | Windows message/IME bridge fake tests、Windows target build、workspace gates、Rust minimum、文書検査が同revisionで成功。WndProcからuser callbackを呼ばない |

wrapper単位の正式unit ID・path mapping・サポートする各言語の出荷範囲は、設計承認後に`.docs/32_unit_test/`、implementation inventory、`tools/units.yaml`へ登録する。wrapper branchごとにCore APIs branchを直接親とする。

## 必須確認

- key down/upとText commitを別eventとして扱い、Windows messageをCore/user callbackから同期waitしない。
- stale Window generation、inactive/detached/disabled対象、Runtime shutdown時にeventを誤適用しない。
- Pointer hit-test座標、DIP変換、clip、target-to-Logical routeの担当境界を承認済み設計から一意に確定する。追加仕様が必要なら別途承認する。
- Focus callback/public event APIの有無を実装上の都合で決めない。
- Windows実機keyboard/mouse/IME動作はfake/CIと区別し、必要なGUI smokeを0.0.1 release qualificationへ記録する。

## 設計照合結果・レビュー提案（2026-11-01）

### 確認できた整合と不足

- RQ-09-009/010/046/047/049/055/056/059/068/070は承認済み。特にRQ-09-068はInputをLogical祖先へ配送し、handled指定で以後の配送と既定動作を止めることを初期共通基盤の要求としている。
- DD `ui/input-focus.md` 1.1はCore側の`InputRouter`/`RouteSnapshot`案、Focus遷移、keyboard/text区別を記載する一方、route方式を確認候補とし、subscription・handled/default action・公開ABIを基本Control DDへ委譲したままである。
- DD `ui/controls.md` 1.5はPointer activationとAccessibility invokeからButtonのsemantic activationを呼ぶ案を記載するが、Input event handlerを登録・解除し、handledを返す利用者向けの接点を定義していない。
- 共通ABI DDには引数なしcallbackのProperty subscriptionがあるが、Input event payloadとhandled stateを渡す契約には流用できない。
- UT-INPUT-001はhandledによる祖先配送・既定動作の停止、detach再検証、非再帰入力を明示している。現行CoreにはFocus状態管理だけがあり、InputRouter/route deliveryは未実装。Windows WndProcにもkeyboard/text/pointer接続はない。

### 推奨する最小の設計解決

`ui/input-focus.md`、`ui/controls.md`、共通C ABI DD、`ui.md`の文書単位改訂案を作り、次の初期契約をレビューする。

1. CoreをInput routingとFocus状態の唯一の正本とし、targetからLogical parentへbubbleする。各handlerの後に所属・有効状態を再検証し、detach等でrouteが無効になれば残りを止める。
2. 外部handlerがhandledを設定できる型付きInput event subscriptionを用意する。handledなら後続の祖先handlerとControlの既定動作（Button activation等）を止める。Property subscriptionの引数なしcallbackは使い回さない。
3. 初期共通eventはkeyboard key-down/up、committed text、Button activationに必要なprimary pointer down/upとする。pointer move/wheel/capture、特定key割当、Tab traversal、shortcutは採用しない。TextBoxの文字編集とIME sessionは既存TextBox DDに接続し、OS固有composition詳細をこのunitへ持ち込まない。
4. Focus状態の取得APIとFocus変更notificationを区別して定義する。RQ-09-049の順序保証を満たすため、notificationを公開する場合はFocusLost→FocusGained順と非再入配送を明記する。公開notificationを採用しない場合、要求の「変更後の状態を観測できる」をどう満たすかをDD上で明示する。
5. C ABI event payload、subscription解除・callback破棄、handledの変更可能期間、callback中のdetach/Focus変更、エラー時の既定動作、各wrapperへの対応を一緒に定義する。宣言だけのC snippetを実装済みAPIと誤認させない。

この案はRQ-09-068を満たすため公開event subscriptionを含むため、Coreだけに閉じた実装ではない。APIの詳細と対応範囲を文書全体で承認後、Rust Core、Rust/C++/C#/Python wrapperの影響を見積もって実装unitを再分割する。

### 設計改訂レビュー案

| 文書 | 改訂案 | 状態 |
|---|---|---|
| [UI InputとFocus](../13_detail_design/ui/input-focus.md) | target-to-Logical bubbling、handled、Focus eligibility loss、TextCommit、pending Button press、Focus getter | 1.2全文承認済み |
| [UI Controls](../13_detail_design/ui/controls.md) | WidgetごとのInput subscription、TextBox/Button default action | 1.6全文承認済み |
| [共通C ABI](../13_detail_design/runtime-objects/abi.md) | borrowed InputEvent handle、typed callback/getters/handled setter、feature ID 9、Focus getter | 1.5全文承認済み |
| [UI unit test](../32_unit_test/ui.md) | route/default/lifetime/focus disable/key-text/pointer cases | 1.7全文承認済み |
| [Platform internal layers](../13_detail_design/platform/internal-layers.md) | Windows native event copy/order/DIP/focus bridge | 2.3全文承認済み |

文書案でWidgetごとのsubscription、Focus getter（Focus callbackなし）、primary pointer down/upを採用する。key identity/modifierの固定数値表、primary button enum、各language wrapperでのborrowed event facadeの形、Windows native keyboard/IME message対応はレビュー未了。設計文書の承認後に実装unitとbranch依存を再計画し、改訂計画はcommit/push前に利用者レビューへ出す。

## ブランチ・Git

- 計画のレビュー承認前にcommit/pushしない。
- 設計文書と実装分割計画の承認後、0.0.1のCore APIs childを作り、`UNIT-CORE-091`を実装・検証・日本語commit/pushする。
- Core APIs branchからRust/C++/CSharp/Python wrapper childをそれぞれ新規作成する。対象言語を切り替える時に他言語wrapper branchを親にしない。
- Windows Platform childも最新Core APIs branchから独立して作る。Core sink/APIが確定してからnative input接続を実装する。
- 各unitのtest pass後に同unit branchへcommit/pushする。関連TODO全体の完了後、Core APIを親へ先に統合し、その後wrapper/Windows branchをまとめたmerge PRとして提出する。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-01 | 計画0.1案 | 利用者承認 | Windows close-request統合後の候補。計画承認は設計改訂案の承認・実装開始を意味しない |
| 2026-11-01 | 計画0.2案 | 利用者承認 | RQ-09-068照合後のevent subscription / wrapper splitと5文書の改訂案を反映。実装unit登録・実装開始は別管理 |
