# 単体試験：4言語wrapper・cleanup・callback解析

- 文書版: 0.1
- 状態: 0.1全文承認済み（利用者、2026-10-17）。製品testは未実装・未実行。接続gapは別管理
- 更新日: 2026-10-17

候補単位は各言語owner/borrow/lease/ticket、callback root、Python終了集約、解析器IR/診断。native ABI shimはtoken消費・未消費を明示して応答し、実Coreとのparityは上位試験へ分ける。

上位根拠: [単体試験方針](unit-test-strategy.md)、[実装unit台帳契約](implementation-test-inventory.md)、[共通fixture・実行記録](README.md)。

設計根拠: [wrappers/cpp.md](../13_detail_design/wrappers/cpp.md) 1.1「所有型 / cleanup同期」、[wrappers/csharp.md](../13_detail_design/wrappers/csharp.md) 1.1「CallLease / Dispose」、[wrappers/rust.md](../13_detail_design/wrappers/rust.md) 1.1「所有型 / Drop / UI context」、[wrappers/python.md](../13_detail_design/wrappers/python.md) 1.1「owner / callback root / 終了」、[wrappers/analyzer.md](../13_detail_design/wrappers/analyzer.md) 1.1「IR / generation / 診断」。

## ownerと例外境界

native呼出し回数、token消費、lock取得、destroy、root保持を記録するshimを使う。元例外の伝播中にもcleanupを実行し、callbackからの再入をbarrierで再現する。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-wrap-cpp-001"></a>UT-WRAP-CPP-001 | Ownedをmove、明示close失敗、destructor中release失敗、handler交換を試す | move元は非所有。明示失敗はowner保持、自動後始末失敗はPendingNodeへ一回移転。lock外native/user call | [RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-04-016](../11_requirements/RQ-04_abi-boundary.md#rq-04-016)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-WRAP-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01)、[TC-WRAP-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-ABI-EXPOSE-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-expose-01) |
| <a id="ut-wrap-cs-001"></a>UT-WRAP-CS-001 | CallLease中にDisposeし、最後のlease終了時releaseを失敗させる | 新leaseを止め既存callのtokenを保持。失敗tokenをCleanupTicketへ保全、delegate rootをnative destroy前に回収しない | [RQ-05-007](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-007)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)<br/>[TC-CALLBACK-01](../30_release_test/requirement-test-matrix.md#case-tc-callback-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-WRAP-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01)、[TC-WRAP-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01) |
| <a id="ut-wrap-rs-001"></a>UT-WRAP-RS-001 | OwnedのUI型をworkerへ移すcompile-fail fixture、borrow超過、Drop失敗、callback panicを試す | UI ownerのSend/Sync制約とborrow境界をcompileで検査。失敗ticketをregistry保持、panicはFFIを越えない | [RQ-06-003](../11_requirements/RQ-06_threading-dispatch.md#rq-06-003)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-04-015](../11_requirements/RQ-04_abi-boundary.md#rq-04-015)<br/>[TC-THREAD-01](../30_release_test/requirement-test-matrix.md#case-tc-thread-01)、[TC-WRAP-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-WRAP-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01)、[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-ABI-EXPOSE-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-expose-01) |
| <a id="ut-wrap-py-001"></a>UT-WRAP-PY-001 | 初期化失敗、context内例外、close失敗、callback中root回収、host停止を再現 | owner状態を二重消費せず、元例外をcleanupで隠さない。C destroy stackから戻るまでrootを保持、待機でGIL/必要lockを解放 | [RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-04-017](../11_requirements/RQ-04_abi-boundary.md#rq-04-017)、[RQ-03-008](../11_requirements/RQ-03_language-support.md#rq-03-008)、[RQ-03-009](../11_requirements/RQ-03_language-support.md#rq-03-009)、[RQ-03-015](../11_requirements/RQ-03_language-support.md#rq-03-015)<br/>[TC-LIFE-01](../30_release_test/requirement-test-matrix.md#case-tc-life-01)、[TC-WRAP-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)、[TC-WRAP-CS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cs-01)、[TC-WRAP-RS-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-rs-01)、[TC-WRAP-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01)、[TC-ABI-EXPOSE-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-expose-01) |
| <a id="ut-wrap-parity-001"></a>UT-WRAP-PARITY-001 | 各shimへ同じraw status/unknown code、UTF8/NUL、F64/DECIMALを返す | 4言語で値と機械識別を保存し、型不一致を暗黙切捨てしない。実Core parityの代替とはしない | [RQ-04-020](../11_requirements/RQ-04_abi-boundary.md#rq-04-020)、[RQ-04-022](../11_requirements/RQ-04_abi-boundary.md#rq-04-022)、[RQ-04-027](../11_requirements/RQ-04_abi-boundary.md#rq-04-027)<br/>[TC-ABI-REP-01](../30_release_test/requirement-test-matrix.md#case-tc-abi-rep-01)、[TC-OWN-01](../30_release_test/requirement-test-matrix.md#case-tc-own-01)、[TC-ERROR-01](../30_release_test/requirement-test-matrix.md#case-tc-error-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01) |
## 解析器

解析対象は一時workspaceの固定ソース。generation更新と解析完了を制御し、ソースに実行時副作用を仕込んでも解析時には実行しない。

各行を独立fixtureで実施する。後始末は共通規則に従い、この節で保持したowner・登録・session・workerも回収する。

| 予定test ID | 前提からの入力・操作 | 観測・期待結果 | 要求根拠 / 論理case |
| --- | --- | --- | --- |
| <a id="ut-analyzer-001"></a>UT-ANALYZER-001 | 既知禁止呼出し、未解決alias、循環warning、UI blocking候補を含むcallback IRを解析 | 証明できる禁止だけerror、Unknownをerrorへ捏造せずwarningと区別。source spanを保存 | [RQ-03-011](../11_requirements/RQ-03_language-support.md#rq-03-011)、[RQ-03-012](../11_requirements/RQ-03_language-support.md#rq-03-012)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-WRAP-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01) |
| <a id="ut-analyzer-002"></a>UT-ANALYZER-002 | 旧generation解析中に編集し、新generationを公開してから旧解析を完了させる | 旧診断を最新結果としてpublishしない。cache一致だけを現行ソースの正しさの根拠にしない | [RQ-03-011](../11_requirements/RQ-03_language-support.md#rq-03-011)<br/>[TC-BIND-01](../30_release_test/requirement-test-matrix.md#case-tc-bind-01)、[TC-TRACE-01](../30_release_test/requirement-test-matrix.md#case-tc-trace-01)、[TC-WRAP-PY-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-py-01) |

## 承認履歴

| 日付 | 対象版 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.1全文 | — | レビュー案・未承認。作成指示を試験設計の承認や試験合格とは扱わない |
| 2026-10-17 | 0.1全文 | 利用者 | 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」により既存試験案を承認し環境整備を開始。製品test実装・合格を意味しない |

## 改訂履歴

| 版 | 日付 | 内容・理由 | 作成者 | 承認との対応 |
| --- | --- | --- | --- | --- |
| 0.1 | 2026-10-17 | テスト文書作成指示に基づき前提・操作・観測・期待値・後始末を具体化 | Codex | 本版全文のレビュー待ち。未実装・未実行 |
