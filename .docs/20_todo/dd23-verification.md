# DD-23 縦断検証設計 作業計画（再分類済み）

- 状態: 詳細設計計画として終了。検証契約を独立test文書へ移管
- 着手日: 2026-10-12
- 更新日: 2026-10-14
- 対象: 要求・承認済み詳細判断のtraceability、4言語共通縦断シナリオ、境界/ABI/target/package verification、release/version gate
- 上位出典: Accepted ADR-0001、各RQ章の承認履歴、承認済みA1～A32/P/B/C/D/V/L、BD-01～35
- 先行詳細設計: 承認済みDD-01～22
- 対応する詳細設計: なし（旧レビュー案は2026-10-14に再分類後、利用者指示で削除）

> この計画は2026-10-14に再分類され、以降の作業計画としては使用しない。共通テストルール・契約は[requirements-test-manifest作業計画](requirements-test-manifest.md)と`.docs/30_release_test/requirements-test-manifest.md`へ移管した。DD-23は詳細設計の承認・完了対象ではない。以下は再分類前の計画記録。

## 目的

Rust Core、versioned C ABI、C++/C#/Rust/Python wrapper、target artifactを通す初期縦断シナリオと、契約・互換性・失敗系を検証する責務を整理する。Release testは各releaseでのシステム用途適合を、version testは変更要求と影響範囲に応じた回帰を確認するという既存規約を維持し、通過gateとtraceabilityを提案する。テストframework、CI provider、具体runner/job、未設計APIの振る舞い、数値閾値は根拠なく確定しない。

## 作業手順

1. ADR、各RQ章の承認状態、設計骨格、DD-00、DD-01～22の判断ID、BD-16/22のverification state、release/version test規約を照合する。承認済み要求と改訂案を同一状態として扱わない。
2. DD判断ID・verification item・case/evidenceの追跡方法を提案する。RQと論理case IDの対応は独立した要求test対応表を正本とし、DD-23に複製しない。初期縦断シナリオ外であることを理由にCollection/SelectionやNFRの要求を対象台帳から落とさない。
3. Core/unit、C ABI contract/compatibility、wrapper contract（4言語）、platform/runtime/hardware、artifact/package consumerのverification levelと責務を提案する。テストframeworkやCI製品は指定しない。
4. RQ-03-001～004とRQ-09-001～011のWindow/StackPanel/TextBox/Text/Buttonシナリオをlanguage parity matrixにし、setup、操作、observable result、cleanup、shared fixtureの境界を定義する。Accessibility、focus/input/IMEを各DDの承認済み範囲内で対応付ける。
5. DD-01～22から主要なnormal/failure/lifecycle/thread/callback/async/validation/rendering/accessibility/package/trustケースを集め、同一契約のcross-language coverageとCore固有caseを分ける。
6. ABI compatibility/version matrixをA3・DD-ABI-03～10・BD-28およびDD-22へ対応付ける。旧client/new same-major runtime、unknown status/feature、symbol/version mismatch、manifest schemaとartifact digest/trust failureを含める。
7. BD-16/22およびDD-22のbuild/runtime/hardware verification stateをtest gateへ対応付ける。Windows x64/Linux x64/ARM cross-build/representative ARM boardの段階を保ち、cross-build成功をruntime/hardware supportへ昇格しない。
8. release testとversion testの役割、release可否gate、変更影響分析からregression selectionへの流れを提案する。要求変更でtraceabilityを再点検し、毎releaseで用途適合を確認する。
9. RQ-13-001～011のperformance/resource measurementを対応付け、起動時間・binary size・memory・FPS・layout・Property update・Collection diff等を記録対象として扱う。閾値、計測環境、未設計Collection細則は未決または後続へ送る。
10. `DD-VERIFY-*`判断ID付き未承認レビュー案を詳細設計文書として一時保存した。decision map上の対応不足、未計画のCollection詳細設計、DD-24以降との境界を点検し、利用者確認前は承認済み仕様にしない。その後テストルール・契約へ再分類し、原案ファイルは2026-10-14に削除した。

## 2026-10-14 再照合・更新記録

- RQ再編後の正本を確認し、共通scenarioの根拠をRQ-03-001～004/RQ-09-001～011、platformをRQ-11、実測項目をRQ-13-001～011へ更新した。要求IDの承認状態は各RQ章本文と個別承認履歴に従う。
- 要求test対応表がRQ→logical case IDと受入観測を管理し、unit-test方針が実装単位を管理するため、DD-23はこれらを重複作成せず、横断verification level/gate/DD根拠を管理する境界を追記した。要求test対応表の全体状態は未承認であり、承認済み範囲を区別する。
- 文書参照を現行RQ索引・章へ更新し、RQから下流へのリンクを追加していない。BD-16のWindows/Linux native検証とARM cross-build/hardware gateを照合した。
- DD-VERIFY-01～12はすべて未承認。DD-23フェーズの完了チェックは利用者レビュー結果を反映するまで付けない。

## 対象外と停止条件

- ADR、要求仕様、A/P/B/C/D/V/L、BD、承認済みDDの意味をこの文書だけで変更すること。
- 未設計・未承認APIの具体動作をverification expectationとして先取りすること。
- CI provider、runner image、test framework、job graph、test sharding/parallelism、具体commandやrepository layoutを選定すること。
- 根拠がない数値performance threshold、flake retry count、coverage percentage、release candidate countを設定すること。
- Collection/Selectionの詳細API・通知順・競合など未設計事項をテスト設計だけで確定すること。必要なら計画されたDDフェーズ追加を別途提案する。
- 実装コードやテストsuiteを作成すること。承認済み設計を基にしたimplementation/test planは後続作業とする。

## レビューで確認する点

1. 同じ縦断シナリオの振る舞いと期待結果がC++/C#/Rust/Pythonで比較できる。
2. C ABI contractと各wrapper固有テストを区別し、同一性が必要な振る舞いを言語別の偶然にしない。
3. 初期縦断シナリオ、全要求、Collection/NFR、失敗・終了境界のtraceabilityに漏れがない。
4. ABI major/minor/patch、未知値、artifact identity/signature/manifest、target profileを承認済みDD-06/22の境界で検証する。
5. build/runtime/hardware verification stateとrelease support matrixが対応し、ARM cross-buildだけでhardware supportを主張しない。
6. release testの毎release用途適合gateとversion testの変更影響・回帰selectionが混ざらない。
7. 未設計事項・測定閾値・CI/test toolingを暗黙に決めず、次の担当が分かる。

## 承認記録

未承認。レビュー案作成後、DD-VERIFY判断ごとに利用者確認を受け、結果を同じ詳細設計文書と本計画へ反映する。
