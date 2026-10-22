# 未設計レビュー / 要求に戻す変更・補足案

- 文書版: 0.2
- 状態: 全31件承認済み・RQ正本適用済み（2026-10-17）

`REM-RQ-*`はレビュー時の仮IDである。全31件を正式RQへ対応付け、現行280要求として承認・適用した。引用はレビュー時点の文面を保持し、現行の文言・段階・履歴は以下の正式RQを正本とする。class/method等の手段は要求本文へ移さない。

## 正式RQへの適用

| レビューID | 現行要求 |
| --- | --- |
| REM-RQ-01 | [RQ-04-029](../../11_requirements/RQ-04_abi-boundary.md#rq-04-029) |
| REM-RQ-02 | [RQ-02-022](../../11_requirements/RQ-02_core-features.md#rq-02-022) |
| REM-RQ-03 | [RQ-09-062](../../11_requirements/RQ-09_ui-scope.md#rq-09-062) |
| REM-RQ-04 | [RQ-09-063](../../11_requirements/RQ-09_ui-scope.md#rq-09-063) |
| REM-RQ-05 | [RQ-09-064](../../11_requirements/RQ-09_ui-scope.md#rq-09-064) |
| REM-RQ-06 | [RQ-06-012](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-012) |
| REM-RQ-07 | [RQ-03-013](../../11_requirements/RQ-03_language-support.md#rq-03-013) |
| REM-RQ-08 | [RQ-07-017](../../11_requirements/RQ-07_binding.md#rq-07-017) |
| REM-RQ-09 | [RQ-06-013](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-013) |
| REM-RQ-10 | [RQ-09-065](../../11_requirements/RQ-09_ui-scope.md#rq-09-065) |
| REM-RQ-11 | [RQ-09-066](../../11_requirements/RQ-09_ui-scope.md#rq-09-066) |
| REM-RQ-12 | [RQ-09-067](../../11_requirements/RQ-09_ui-scope.md#rq-09-067) |
| REM-RQ-13 | [RQ-09-068](../../11_requirements/RQ-09_ui-scope.md#rq-09-068) |
| REM-RQ-14 | [RQ-09-069](../../11_requirements/RQ-09_ui-scope.md#rq-09-069) |
| REM-RQ-15 | [RQ-09-070](../../11_requirements/RQ-09_ui-scope.md#rq-09-070) |
| REM-RQ-16 | [RQ-09-071](../../11_requirements/RQ-09_ui-scope.md#rq-09-071) |
| REM-RQ-17 | [RQ-11-022](../../11_requirements/RQ-11_platform.md#rq-11-022) |
| REM-RQ-18 | [RQ-09-072](../../11_requirements/RQ-09_ui-scope.md#rq-09-072) |
| REM-RQ-19 | [RQ-11-023](../../11_requirements/RQ-11_platform.md#rq-11-023) |
| REM-RQ-20 | [RQ-10-014](../../11_requirements/RQ-10_accessibility.md#rq-10-014) |
| REM-RQ-21 | [RQ-03-014](../../11_requirements/RQ-03_language-support.md#rq-03-014) |
| REM-RQ-22 | [RQ-03-015](../../11_requirements/RQ-03_language-support.md#rq-03-015) |
| REM-RQ-23 | [RQ-03-016](../../11_requirements/RQ-03_language-support.md#rq-03-016) |
| REM-RQ-24 | [RQ-03-017](../../11_requirements/RQ-03_language-support.md#rq-03-017) |
| REM-RQ-25 | [RQ-12-022](../../11_requirements/RQ-12_build-distribution.md#rq-12-022) |
| REM-RQ-26 | [RQ-12-023](../../11_requirements/RQ-12_build-distribution.md#rq-12-023) |
| REM-RQ-27 | [RQ-11-024](../../11_requirements/RQ-11_platform.md#rq-11-024) |
| REM-RQ-28 | [RQ-04-030](../../11_requirements/RQ-04_abi-boundary.md#rq-04-030) |
| REM-RQ-29 | [RQ-12-024](../../11_requirements/RQ-12_build-distribution.md#rq-12-024) |
| REM-RQ-30 | [RQ-12-025](../../11_requirements/RQ-12_build-distribution.md#rq-12-025) |
| REM-RQ-31 | [RQ-09-073](../../11_requirements/RQ-09_ui-scope.md#rq-09-073) |

<a id="rem-rq-01"></a>
## REM-RQ-01 F32変換の外部結果

- 現行要求: [RQ-04-002](../../11_requirements/RQ-04_abi-boundary.md#rq-04-002)
- 関係案: [F32の提供経路・数値変換](01-value.md)
- 承認境界: 初期五型Propertyの拡張は含めない。

> 単精度・倍精度を明示変換できること。精度一致必須または最近接偶数丸めを指定でき、有限値の範囲外を拒否すること。NaN payloadの保存は保証せず、符号付きzeroとinfinityの意味を保つこと。

<a id="rem-rq-02"></a>
## REM-RQ-02 文字列と数値の明示converter

- 現行要求: [RQ-02-017](../../11_requirements/RQ-02_core-features.md#rq-02-017)、[RQ-02-019](../../11_requirements/RQ-02_core-features.md#rq-02-019)
- 関係案: [F32の提供経路・数値変換](01-value.md)
- 承認境界: 将来意向の採用判断。初期release必須へ変換しない。

> 次段階でUTF8と整数・十進値の明示parse/formatを提供すること。初期の文法はlocale非依存の符号・数字・十進点とし、表現不能を黙って丸めないこと。Bindingの暗黙型変換は追加しないこと。

<a id="rem-rq-03"></a>
## REM-RQ-03 Collectionの型とsnapshot

- 現行要求: [RQ-09-019](../../11_requirements/RQ-09_ui-scope.md#rq-09-019)、[RQ-09-050](../../11_requirements/RQ-09_ui-scope.md#rq-09-050)
- 関係案: [Collection・Selectionの所有、batch、snapshot](02-collection.md)
- 承認境界: schemaの固定等の実現構造はDDへ。

> 次段階Collectionの列に既存の五値種別を指定できること。可視範囲と安定ID・revisionを持つ一貫したsnapshotを保持し、元Modelの後続変更に影響されず読めること。

<a id="rem-rq-04"></a>
## REM-RQ-04 Collection通知と再入

- 現行要求: [RQ-09-052](../../11_requirements/RQ-09_ui-scope.md#rq-09-052)
- 関係案: [Collection・Selectionの所有、batch、snapshot](02-collection.md)
- 承認境界: 既存の原子性を具体化する新しい外部順序・制限。

> 同じ一括更新の集合変更通知を選択変更通知より先に観測し、両方から同じ確定revisionを読めること。通知中の再入一括更新は拒否し、後続処理へ依頼できること。

<a id="rem-rq-05"></a>
## REM-RQ-05 Selection縮退・差替え

- 現行要求: [RQ-09-020](../../11_requirements/RQ-09_ui-scope.md#rq-09-020)、[RQ-09-051](../../11_requirements/RQ-09_ui-scope.md#rq-09-051)
- 関係案: [Collection・Selectionの所有、batch、snapshot](02-collection.md)
- 承認境界: gesture/navigationの追加を含めない。

> 複数選択を保ったままSingleへ縮退する変更は拒否すること。削除後currentを自動補充せず、対象集合を変更する場合は別の選択状態として明示できること。

<a id="rem-rq-06"></a>
## REM-RQ-06 終了要求の完了観測

- 現行要求: [RQ-06-006](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-006)、[RQ-06-007](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-007)
- 関係案: [Host境界・shutdown完了観測・cleanup drain](03-runtime.md)、[Rust公開型・receiver・registry・panic](11-rust.md)
- 承認境界: host event境界・停止通知を接続する利用条件を明記する。

> 受付済み終了要求の未完了・完了・host実行不可と理由をthreadに依存せず観測できること。観測参照の破棄で終了要求を取り消さず、停止から同UI hostが復旧した場合は明示再試行できること。

<a id="rem-rq-07"></a>
## REM-RQ-07 初期unload範囲

- 現行要求: [RQ-03-001](../../11_requirements/RQ-03_language-support.md#rq-03-001)、[RQ-03-002](../../11_requirements/RQ-03_language-support.md#rq-03-002)、[RQ-03-003](../../11_requirements/RQ-03_language-support.md#rq-03-003)、[RQ-03-004](../../11_requirements/RQ-03_language-support.md#rq-03-004)
- 関係案: [Host境界・shutdown完了観測・cleanup drain](03-runtime.md)、[C# token owner・interop・Dispose・load](10-csharp.md)
- 承認境界: 所有・回収保証を弱めず、loaderの対象範囲を明示する。

> 初期版の通常利用ではnative libraryをprocess終了まで保持し、active callback/ticketがある状態のhot unloadとcollectible managed unloadを提供対象に含めないこと。

<a id="rem-rq-08"></a>
## REM-RQ-08 WatchDog設定適用時点

- 現行要求: [RQ-07-013](../../11_requirements/RQ-07_binding.md#rq-07-013)、[RQ-07-014](../../11_requirements/RQ-07_binding.md#rq-07-014)
- 関係案: [通知・非収束・Command・エラーcatalog](04-event.md)
- 承認境界: 内部履歴budgetの数値は要求へ昇格しない。

> 診断閾値の変更は既存の保留連鎖にも適用し、最初に保留した時点からの経過に対して評価すること。無効化で連鎖の値や保留作業を捨てないこと。

<a id="rem-rq-09"></a>
## REM-RQ-09 Command進捗値

- 現行要求: [RQ-06-011](../../11_requirements/RQ-06_threading-dispatch.md#rq-06-011)
- 関係案: [通知・非収束・Command・エラーcatalog](04-event.md)
- 承認境界: 現在の進捗方針へ公開値APIを補足する案。

> Commandの単一channelの進捗を明示報告・読取りできること。0～1の正規化値と未定を区別し、中間sampleの全保存は保証しないこと。終端結果を進捗の集約で失わないこと。

<a id="rem-rq-10"></a>
## REM-RQ-10 TextBox編集と位置の操作

- 現行要求: [RQ-09-005](../../11_requirements/RQ-09_ui-scope.md#rq-09-005)、[RQ-09-038](../../11_requirements/RQ-09_ui-scope.md#rq-09-038)
- 関係案: [編集API・Unicode位置・IME回復](05-textbox.md)
- 承認境界: キー割当・Undo/Redoは追加しない。grapheme navigation採用は独立に選択できる。

> 文字列範囲の置換、selection/caretの明示変更、確定値へのresetを利用でき、無効な文字境界を指定する更新を拒否すること。文字単位navigationを採用する場合はUnicode16.0のgrapheme境界に従うこと。

<a id="rem-rq-11"></a>
## REM-RQ-11 TextBox通知の観測順

- 現行要求: [RQ-07-006](../../11_requirements/RQ-07_binding.md#rq-07-006)、[RQ-09-038](../../11_requirements/RQ-09_ui-scope.md#rq-09-038)
- 関係案: [編集API・Unicode位置・IME回復](05-textbox.md)
- 承認境界: 既存の確定/未確定の意味を維持する。

> 同じ編集確定のBinding伝播とProperty通知の後に、本文・selection・composition・入力状態/理由が一致した編集snapshotを通知すること。Preeditでは確定値通知を発生させないこと。

<a id="rem-rq-12"></a>
## REM-RQ-12 座標・数値範囲・clip

- 現行要求: [RQ-09-008](../../11_requirements/RQ-09_ui-scope.md#rq-09-008)
- 関係案: [Layout・座標・Input route・Tree所属](06-geometry.md)
- 承認境界: DIP/f64/roundingの実現方式はDD。clipの見える結果は要求承認対象。

> 論理寸法とdevice scaleを区別でき、無効な数値や表現不能な配置結果を拒否して以前の公開配置を保つこと。初期Controlの描画・hit-testは祖先のclient範囲で切り取ること。

<a id="rem-rq-13"></a>
## REM-RQ-13 Input配送とpointer範囲

- 現行要求: [RQ-02-009](../../11_requirements/RQ-02_core-features.md#rq-02-009)、[RQ-09-010](../../11_requirements/RQ-09_ui-scope.md#rq-09-010)、[RQ-09-049](../../11_requirements/RQ-09_ui-scope.md#rq-09-049)
- 関係案: [Layout・座標・Input route・Tree所属](06-geometry.md)
- 承認境界: bubbleとpointer機能採用は別々の承認項目。キー割当を固定しない。

> 入力は対象からLogical祖先へ扱え、handled指定で以後の配送と既定動作を止められること。pointer move/down/up/wheelと単一pointer captureを追加候補として区別すること。

<a id="rem-rq-14"></a>
## REM-RQ-14 再所属失敗と再試行

- 現行要求: [RQ-09-043](../../11_requirements/RQ-09_ui-scope.md#rq-09-043)、[RQ-09-044](../../11_requirements/RQ-09_ui-scope.md#rq-09-044)
- 関係案: [Layout・座標・Input route・Tree所属](06-geometry.md)
- 承認境界: 既存の変更失敗非部分適用を具体化する。

> 切離し対象の表示資源再作成に失敗してもDetached構造・確定値・登録を保持し、明示再試行できること。所属だけを成功として公開しないこと。

<a id="rem-rq-15"></a>
## REM-RQ-15 Containerのfocusable範囲

- 現行要求: [RQ-09-059](../../11_requirements/RQ-09_ui-scope.md#rq-09-059)
- 関係案: [Layout・座標・Input route・Tree所属](06-geometry.md)
- 承認境界: 初期既定falseから新たな制限を追加するため独立承認。

> 初期版ではWindow/StackPanelの明示focusable=trueへの変更を受け付けないこと。Windowのactive状態は子のkeyboard focusと区別すること。

<a id="rem-rq-16"></a>
## REM-RQ-16 Window開閉とclose判断

- 現行要求: [RQ-09-001](../../11_requirements/RQ-09_ui-scope.md#rq-09-001)、[RQ-11-020](../../11_requirements/RQ-11_platform.md#rq-11-020)
- 関係案: [Control API・Window close・描画backend](07-platform.md)
- 承認境界: OS強制終了通知のveto不能と通常closeを区別する。

> Windowの表示開始・終了を明示でき、終了後もCore状態を保持して再表示できること。close要求をAccept/Decline/Deferで判断でき、既定Accept、未解決判断の破棄はDeclineとすること。Runtime終了は個別vetoの対象にしないこと。

<a id="rem-rq-17"></a>
## REM-RQ-17 描画backendとfallback

- 現行要求: [RQ-11-008](../../11_requirements/RQ-11_platform.md#rq-11-008)、[RQ-11-020](../../11_requirements/RQ-11_platform.md#rq-11-020)
- 関係案: [Control API・Window close・描画backend](07-platform.md)
- 承認境界: Direct2D/DRM等の手段はBD/DDへ配置。

> 初期profileの描画backendと依存条件を公表すること。device喪失後の復旧可否を識別でき、software rendererへの暗黙切替を行わないこと。

<a id="rem-rq-18"></a>
## REM-RQ-18 初期text表示の範囲

- 現行要求: [RQ-09-004](../../11_requirements/RQ-09_ui-scope.md#rq-09-004)、[RQ-09-053](../../11_requirements/RQ-09_ui-scope.md#rq-09-053)
- 関係案: [Control API・Window close・描画backend](07-platform.md)
- 承認境界: 表示機能の削減・範囲変更として別承認。

> 初期Textは単一paragraphの表示、client clipを提供し、soft-wrap/ellipsis/選択copyの追加を必須に含めないこと。Buttonの表示labelとAccessibility nameは独立すること。

<a id="rem-rq-19"></a>
## REM-RQ-19 font・IME・Accessibility profile

- 現行要求: [RQ-09-053](../../11_requirements/RQ-09_ui-scope.md#rq-09-053)、[RQ-10-009](../../11_requirements/RQ-10_accessibility.md#rq-10-009)、[RQ-11-021](../../11_requirements/RQ-11_platform.md#rq-11-021)
- 関係案: [Text engine・IME OS接続・Accessibility](08-access.md)
- 承認境界: 実装libraryの選択だけで実機確認済みと表示しない。

> 各profileのfont/文字対応、日本語IMEの導入条件、Accessibility bus/provider有無を識別できること。desktop LinuxのAT-SPI profileはbusを必要とし、DRM profileはIME brokerを明示すること。

<a id="rem-rq-20"></a>
## REM-RQ-20 OS同期actionの待機境界

- 現行要求: [RQ-10-007](../../11_requirements/RQ-10_accessibility.md#rq-10-007)、[RQ-10-011](../../11_requirements/RQ-10_accessibility.md#rq-10-011)
- 関係案: [Text engine・IME OS接続・Accessibility](08-access.md)
- 承認境界: OS規格の値ではない本製品の提案。Invokeの非同期受付と区別。

> OS同期編集actionの待機制限を設定・識別でき、初期既定候補を2秒とすること。未開始の取消と開始済み結果不明を区別し、timeoutをrollback成功として表示しないこと。

<a id="rem-rq-21"></a>
## REM-RQ-21 言語minimumとbuild基準

- 現行要求: [RQ-03-001](../../11_requirements/RQ-03_language-support.md#rq-03-001)、[RQ-03-002](../../11_requirements/RQ-03_language-support.md#rq-03-002)、[RQ-03-003](../../11_requirements/RQ-03_language-support.md#rq-03-003)、[RQ-03-004](../../11_requirements/RQ-03_language-support.md#rq-03-004)、[RQ-12-001](../../11_requirements/RQ-12_build-distribution.md#rq-12-001)
- 関係案: [C++公開型・handler・cleanup同期](09-cpp.md)、[C# token owner・interop・Dispose・load](10-csharp.md)、[Rust公開型・receiver・registry・panic](11-rust.md)、[Python API・通常終了handshake・特殊method](12-python.md)、[最低版・target matrix・toolchain lock・保守](16-build.md)
- 承認境界: 互換性の新しい利用条件。採用前に既存利用対象への影響を確認。

> 初期利用基準をC++17/CMake3.28、Rust1.85 edition2024、C#14/net10.0、CPython3.13とし、正式supportの処理系/profileを公表すること。free-threaded版とPyPy/subinterpreterのsupportを通常版から推定しないこと。

<a id="rem-rq-22"></a>
## REM-RQ-22 Python正常cleanupの協調条件

- 現行要求: [RQ-03-009](../../11_requirements/RQ-03_language-support.md#rq-03-009)、[RQ-03-010](../../11_requirements/RQ-03_language-support.md#rq-03-010)
- 関係案: [Python API・通常終了handshake・特殊method](12-python.md)
- 承認境界: 無条件な通常終了保証との限界差異。要求を暗黙に狭めず独立承認する。

> 管理entryの通常return・一般例外・SystemExitと管理host停止でcleanupを要求できること。hostはUI thread終了前に停止通知を接続し、callback/finalizer/handlerの利用者処理は有限に戻り、終了処理を同期待ちするmainとの循環待ちを作らないこと。未登録host・停止不能な利用者処理・即時process終了でcleanup完遂を保証しないこと。

<a id="rem-rq-23"></a>
## REM-RQ-23 Python基底初期化とoverride範囲

- 現行要求: [RQ-03-008](../../11_requirements/RQ-03_language-support.md#rq-03-008)、[RQ-03-011](../../11_requirements/RQ-03_language-support.md#rq-03-011)
- 関係案: [Python API・通常終了handshake・特殊method](12-python.md)、[callback識別・禁止検知・lint/MCP](13-analyzer.md)
- 承認境界: 動的変更を安全確認済みと表示せず、__del__確実実行を保証しない。

> 管理ownerの基底初期化未実行を検出し、資源取得の前後で明確に失敗できること。close/context/native entryの保護対象をoverride不可とし、任意演算子に一律base callを要求しないこと。

<a id="rem-rq-24"></a>
## REM-RQ-24 callback markerと検知能力

- 現行要求: [RQ-03-011](../../11_requirements/RQ-03_language-support.md#rq-03-011)、[RQ-03-012](../../11_requirements/RQ-03_language-support.md#rq-03-012)
- 関係案: [callback識別・禁止検知・lint/MCP](13-analyzer.md)
- 承認境界: 静的解析拡張の新規採用。JITは初期対象外。

> callback種別の明示annotationを各言語で利用でき、型・禁止効果・同期/非同期不一致を診断できること。禁止を証明できない経路はUnknownとし、警告とerrorを区別すること。

<a id="rem-rq-25"></a>
## REM-RQ-25 manifest資源上限

- 現行要求: [RQ-12-015](../../11_requirements/RQ-12_build-distribution.md#rq-12-015)
- 関係案: [署名envelope・schema・parser](14-manifest.md)
- 承認境界: 新しい拒否条件として要求側で承認する。

> manifestの資源制限を構成でき、初期候補値はbytes1 MiB/depth32/member4096/string64 KiB/signature8とすること。制限超過を明示拒否し、許容される新manifestへの変更とpolicy変更を追跡できること。

<a id="rem-rq-26"></a>
## REM-RQ-26 offline期限・rollback/失効

- 現行要求: [RQ-12-013](../../11_requirements/RQ-12_build-distribution.md#rq-12-013)、[RQ-12-014](../../11_requirements/RQ-12_build-distribution.md#rq-12-014)
- 関係案: [鍵管理・失効・offline・staging](15-trust.md)
- 承認境界: offline availabilityへの影響が大きい選択。長期airgap代替も比較する。

> 明示trust bundleでも期限切れ・rollback・検証不能を拒否すること。候補期間をroot365日/manifest30日/revocation7日とし、長期offlineは別profileの承認済み期間を用いること。取得していない最新失効を即時検知すると保証しないこと。

<a id="rem-rq-27"></a>
## REM-RQ-27 初期support profile

- 現行要求: [RQ-11-001](../../11_requirements/RQ-11_platform.md#rq-11-001)、[RQ-11-002](../../11_requirements/RQ-11_platform.md#rq-11-002)、[RQ-11-021](../../11_requirements/RQ-11_platform.md#rq-11-021)、[RQ-12-010](../../11_requirements/RQ-12_build-distribution.md#rq-12-010)
- 関係案: [最低版・target matrix・toolchain lock・保守](16-build.md)
- 承認境界: ARMv7の正式製品対象を削除せず実施stageを区別。代表board採否は独立。

> 初期profile候補をWindows11 x64、Debian12 x64、ARMv7/AArch64 cross-buildと代表RPi4 AArch64とすること。初期ARMv7はbuild検証表示に限定し、実機未確認を明示すること。

<a id="rem-rq-28"></a>
## REM-RQ-28 初回stableとwrapper互換

- 現行要求: [RQ-04-028](../../11_requirements/RQ-04_abi-boundary.md#rq-04-028)
- 関係案: [最低版・target matrix・toolchain lock・保守](16-build.md)
- 承認境界: 初回公開前の番号選定でありrelease実行を指示しない。

> 初回stableをRuntime1.0.0/ABI1の候補とし、pre-releaseをrc.Nで識別できること。wrapper versionが独立でも必要Runtime/ABI/featureを判断できること。

<a id="rem-rq-29"></a>
## REM-RQ-29 保守・証拠の保存

- 現行要求: [RQ-12-017](../../11_requirements/RQ-12_build-distribution.md#rq-12-017)、[RQ-12-018](../../11_requirements/RQ-12_build-distribution.md#rq-12-018)、[RQ-12-019](../../11_requirements/RQ-12_build-distribution.md#rq-12-019)、[RQ-12-020](../../11_requirements/RQ-12_build-distribution.md#rq-12-020)
- 関係案: [最低版・target matrix・toolchain lock・保守](16-build.md)
- 承認境界: 内部logの個人情報・権限・例外はSD/運用へ。新しい公開SLAを追加しない。

> EOL後も最終artifact・metadata・support告知を参照可能にすること。内部test logはEOL+24か月、security監査logは2年という候補期間で運用責任を定めること。

<a id="rem-rq-30"></a>
## REM-RQ-30 初期link形式

- 現行要求: [RQ-12-003](../../11_requirements/RQ-12_build-distribution.md#rq-12-003)、[RQ-12-004](../../11_requirements/RQ-12_build-distribution.md#rq-12-004)、[RQ-12-005](../../11_requirements/RQ-12_build-distribution.md#rq-12-005)
- 関係案: [package名・loader・artifact取得・catalog](17-package.md)
- 承認境界: 静的variantの採用はmulti-language状態共有条件を別に審査する。

> 初期配布は共通native dynamic runtimeを利用し、各言語へ独立Coreを静的同梱するvariantを提供対象に含めないこと。

<a id="rem-rq-31"></a>
## REM-RQ-31 次段階Style/Theme/Resource/Template

- 現行要求: [RQ-09-023](../../11_requirements/RQ-09_ui-scope.md#rq-09-023)、[RQ-09-024](../../11_requirements/RQ-09_ui-scope.md#rq-09-024)、[RQ-09-025](../../11_requirements/RQ-09_ui-scope.md#rq-09-025)、[RQ-09-026](../../11_requirements/RQ-09_ui-scope.md#rq-09-026)
- 関係案: [次段階UI・将来候補の範囲と拡張点](19-scope.md)
- 承認境界: 既存の次段階範囲を具体化する。初期Controlへの追加とは分ける。

> 次段階でtyped表示設定の再利用、共有外観切替、local→Logical祖先→Runtime themeのResource参照、Logical identityを保つ内部表示差替えを提供すること。未定義/型不一致/cycleを明示失敗とし、任意script実行を必須に含めないこと。


## 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 31要求変更候補の作成 | 利用者 | 一括生成を指示。採用・正本反映・実装は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 未設計事項の一括レビュー案を作成 | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.2 / 全31要求の採用・RQ本文適用 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.2 | 2026-10-17 | 全31要求の採用・RQ本文適用を正本へ適用し、旧判断との対応を保持 | Codex |
