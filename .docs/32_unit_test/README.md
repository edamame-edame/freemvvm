# テスト設計の入口

- 文書版: 0.1
- 状態: 0.1全文承認済み（利用者、2026-10-17）。製品testは未実装・未実行。接続gapは別管理
- 更新日: 2026-10-17

要求の280RQ・92論理caseは[要求対応表](../30_release_test/requirement-test-matrix.md)と[要求別assertion](../30_release_test/requirements-test-assertions.md)を正本とする。ここでは承認済みDD1.1を操作・観測可能な試験設計へ具体化する。文書作成だけで全RQの詳細試験設計が完了したことにはしない。

## 文書構成

| 文書 | 検証対象 |
| --- | --- |
| [Core・Runtime](core-runtime.md) | handle / 値 / ABI / post / 終了 |
| [イベント](events.md) | callback / Property / Binding / Command |
| [Collection](collections.md) | batch / snapshot / Selection。次段階対象 |
| [UI](ui.md) | Tree / Layout / Control / Input / Focus / TextBox |
| [WebView](webview.md) | DOM Arena / Markdown / HTML / CSS / WebLayout / Scene / Script VM / MVVM Bridge |
| [言語wrapper](wrappers.md) | 4言語owner / cleanup / 解析器 |
| [Platform・build](platform-build.md) | snapshot / action / artifact / packageの局所契約 |
| [release縦断scenario](../30_release_test/system-scenarios.md) | 実Core・4wrapper・OS/profileでの用途適合 |
| [version回帰計画](../31_version_test/regression-plan.md) | 変更単位からの影響選択 / 新旧互換 |

上位規約は[unit-test方針](unit-test-strategy.md)、[実装台帳](implementation-test-inventory.md)、[共通manifest](../30_release_test/requirements-test-manifest.md)。テスト識別子は実testの予定識別子であり、設計項目IDではない。各表のTCリンクは既存要求対応表からの関連付けであり、一行でそのTC全体を検証したことにはしない。特にTC-TRACE等の監査caseは動作結果だけで充足したと扱わない。UNIT ID、実path/symbol/command、test実装revisionは実装境界決定時に登録する。既存台帳のplanned例をreadyへ昇格したものではない。

## 共通fixtureと後始末

各表の一行を独立testとして扱い、節のfixtureを前提とする。成功/失敗・複数言語・profileのparameterごとにassertion結果を分ける。allocator failpointは操作のprepare中だけへ限定し、fixture構築/後始末まで無差別に失敗させない。状態・通知数・token消費・callbackログを操作前後で比較し、全heap構造の完全一致は要求しない。

fake host/clock/OS/FSは確定的に進行し、並行testはbarrierで開始/受付/commit/終了を制御する。sleepの長さを正しさのoracleにしない。内部counter・pin・queue hookはtest用観測点であり、公開APIへ追加しない。fixtureによるfakeの成功と実backendの成功を別に記録する。

後始末は、workerへの停止通知と終了確認→明示cancel/unbind/close→実行中callback完了→Runtime終了→残owner/snapshot解放を、そのAPIで許されたthread・順序で行う。終了自体を試すcaseは重複終了の有無も観測する。登録userdataはnative destroy後だけ回収し、failed release tokenとcleanup ticketの残数を記録する。failed cleanupをpassへ読み替えず、一時cache・workspace・OS handleも回収する。

## 実装・実行記録

| 項目 | 登録する値 |
| --- | --- |
| 予定test / 論理TC / RQ | 各表の識別子。共有TCでもRQごとのassertion結果を保持 |
| UNIT / source / test path・symbol | 独立検証可能な実装範囲。現在は未登録 |
| 根拠 | DD文書パス・1.1・対象節、SD版、RQ版。設計ID配列は作らない |
| command・toolchain・target/profile | 実在targetから決める。Rust cargo test、C++ CTest、C# dotnet test、Python pytestの採用は既存台帳に従う |
| revision | 実装とtestの同一checkout SHA。PR merge SHA使用時はhead/baseも保存 |
| 結果 | 現在はnot-run。将来pass/fail/blockedと理由・各assertionログを記録 |
| artifact | package/platform試験はidentity・digest、OS/driver/font/IME環境も保存 |

未実装・未実行と、oracleが不整合で実装へ進めないblockedを区別する。本索引と今回の試験設計8文書は0.1全文承認済み。製品testの実装・実行状態は承認と別管理し、実command・test symbol・coverage閾値を仮定で埋めない。開発基盤のみ、[bootstrap検証](development-bootstrap.md)と[開発手順](../../DEVELOPMENT.md)を追加した。

## 設計との接続gap

| 対象 | 根拠と差異 | 試験側の扱い・変更対象 |
| --- | --- | --- |
| Command進捗channel | [RQ-06-013](../11_requirements/RQ-06_threading-dispatch.md#rq-06-013)は単一channel・未定/0～1を要求。[Command](../13_detail_design/events/commands.md)「ProgressとProperties」は専用API/値型を設けないと記載 | UT-COMMAND-005はoracle接続待ち。RQを変えずDD/APIの整合を別改訂で確認 |
| 編集通知順 | [RQ-09-066](../11_requirements/RQ-09_ui-scope.md#rq-09-066)はBinding・Property通知後の編集snapshot通知。[TextBox](../13_detail_design/ui/text-editing.md)「所有権・通知・失敗」に順序保留記述が残る | UT-TEXT-005は要求の順序を保持。DDの保留文とqueue接続の整合が必要 |
| 無効Focus要求の状態図 | [Input/Focus](../13_detail_design/ui/input-focus.md)の状態図にinvalid/no state changeと記しながらActiveFocusedからActiveNoFocusへの矢印がある。本文は旧Focus維持 | UT-FOCUS-001は本文とRQに従う。図の矢印修正をDD変更として記録 |

これらの設計は今回承認された1.1に残る差異であり、承認記録を取り消したり要求を暗黙変更したりしない。test作成中に製品設計本文は修正しない。該当試験を含むunitのready化前に差異を解決し、その改訂とtestを同じ変更単位へ反映する。

## 残る詳細化

今回の6分野は最初の具体化であり、既存92TCの完全な詳細分解ではない。全要求coverageの状態は既存対応表を維持する。性能/メモリ測定条件、Unicode全文字corpus、OS/device別IME・Accessibility、support/EOL監査、次段階Control、manifest暗号互換vectorの網羅表は対応TCを入口に詳細化する。未設計範囲を今回のpassへ含めない。profileや未承認の数値閾値をテストの都合で確定しない。

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |
| 2026-10-17 | 0.1全文 | 利用者 | 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存試験案を承認し環境整備を開始。製品test実装・合格を意味しない |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
