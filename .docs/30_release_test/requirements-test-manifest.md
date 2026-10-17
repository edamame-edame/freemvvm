# Requirements Test Manifest — 要求テストのルール・契約

- 文書版: 0.2
- 状態: 利用者承認済み（文書版0.2、2026-10-14）
- 作成日: 2026-10-14
- 対象: 要求testのcoverage、traceability、case状態、実行証跡、監査規則
- 上位根拠: [要求仕様索引](../11_requirements/reqirements-specification.md)、[RQ-16 文書管理](../11_requirements/RQ-16_document-governance.md)
- 作業計画: [requirements-test-manifest 作業計画](../20_todo/requirements-test-manifest.md)

本書は、要求testの記録と対応表を作るための共通ルールを定める。製品の要求・設計判断、個々の要求case一覧、release/version固有の合否条件、CIの実装方式を定める文書ではない。RQの承認状態は各RQ章を正本とし、本書やtest caseの記載で変更しない。

本書の文書版0.2と共通ルールは利用者承認済みである。各test成果物、RQ、case対応範囲、test実装・実行状態は、それぞれの文書と記録の承認状態に従う。本書の承認は、それら個別成果物の承認やtest合格を意味しない。

## 1. 成果物の役割

| 成果物 | 管理する内容 | 本書との関係 |
|---|---|---|
| RQ章 | 利用者から見える機能、条件、保証、例外、対象stage | 要求testの判定根拠。承認状態はRQ章の履歴に従う |
| 要求テスト対応表 | RQ IDごとの論理case ID、観測内容、coverage状態、実装・実行証跡欄 | 要求testのcase catalogとtraceability台帳 |
| Release test規約 | releaseごとのsystem用途適合確認とrelease可否 | 毎回releaseで適用する専用gateを定める |
| Version test規約 | 変更要求・影響範囲に応じた回帰確認 | version updateごとに適用する専用gateを定める |
| Unit-test方針 | 実装単位とunit-testの対応、変更revisionでの検証 | 要求test caseを実装単位のunit-testで置き換えない |
| BD/DD/SD | 要求を実現する責務・方式・境界 | 要求caseの具体的な期待値が設計済み範囲にあるか確認する根拠 |

本書は要求test対応表のcase一覧を複製しない。Release/Version test規約にある固有gateも再定義しない。ここでは複数成果物が共通に守る記録・状態・監査契約を定める。

## 2. 用語

| 用語 | 意味 |
|---|---|
| 要求test | RQに書かれた利用者向け結果・条件を観測して満足を確認するtest |
| 論理case | 前提、操作、観測、期待値、後始末、根拠が定まった検証単位。frameworkのtest functionやCI jobとは別 |
| case ID | 論理caseを一意に示す識別子。要求テスト対応表の既存`TC-*`はこのIDとして扱う |
| 実test | 論理caseを実装し、特定revision・target/profileで実行するtest |
| test ID | 実testをtest台帳・実行証跡から追跡する識別子。主な検証境界に対応したprefixを付け、case IDとは別に管理する |
| 設計oracle | 承認済みRQと適用可能な承認済みBD/DD/SDから導いた期待結果 |
| 実行証跡 | どのcaseを、どのrevision/version/targetで実行し、何が観測されたかを示す記録 |
| coverage状態 | 要求やcaseが設計・実装・実行のどこまで進んでいるかを示す状態 |

一つのlogical caseは複数言語・targetで複数の実test実行を持てる。case定義、test実装、個別実行結果を一つの状態へまとめない。

## 3. 共通ルール

対象となる各RQ IDを要求テスト対応表で追跡し、caseの有無、対象stage、詳細待ち、非実施理由のいずれかを明示する。caseを設計できていない要求も台帳から除かない。

論理caseは安定したcase IDを持ち、前提、入力・操作、観測結果、期待値の根拠、後始末を判定可能な粒度で定義する。各要求test caseは対象RQを参照し、複数RQを一caseで検査する場合はRQごとのassertionと合否根拠を識別する。RQにない製品動作を期待値へ加えない。

caseの期待値は承認済みRQと、適用可能な承認済み設計から導く。未承認設計、未定義API、未決target/profileの期待値をtest都合で確定しない。要求から具体値が決まらないデータがcaseの実施に必要な場合に限り、test計画文書へ代表使用値を記載する。その際は、値の用途、代表値とした理由、対応するcase/RQまたは設計根拠、採用に伴う仮定・適用範囲を記録し、値を選んだ経緯を追跡可能にする。代表使用値はtest入力を具体化するものであり、製品要求や合否閾値を追加する根拠にはしない。利用者に保証する挙動や合否閾値自体がRQから決められない場合は、test計画で補完せずRQの決定へ戻す。

要求test caseの定義、実testの実装、実行、実行合格を別々に記録する。論理caseが割り当て済みでも実装・実行の証明とはならず、未設計・未実装・未実行・blocked・結果不明をpassまたは成功skipへ換算しない。対象外とする場合は根拠、適用stage、再評価条件を記録する。

実testの結果は、test IDとcase ID、対象revision、runtime/ABI/wrapper/schema version、target/profile、実行日時、pass/fail/blocked等の結果、ログまたはreport参照へ結び付ける。該当するpackage/artifact検証ではidentityとdigestも記録する。

要求変更時は、追加・変更・削除されたRQと受入条件を既存caseへ照合し、caseの追加・改訂・統合・廃止と影響する回帰範囲を記録する。過去case/evidenceは上書きせず、変更前の結果を追跡できるようにする。

Release testは各releaseでのsystem用途適合、Version testは変更された要求と影響範囲の回帰をそれぞれの規約に従い判定する。要求test対応表だけでrelease可否やversion変更完了を判定しない。要求、設計根拠、case、実test、evidenceの参照は依存関係に沿って管理し、RQ章から下流成果物への逆向きリンクを作らない。性能caseは対応するRQの測定対象・条件・環境・revision・結果を記録し、RQで承認されていない数値閾値を合否条件に加えない。

## 4. Coverage・case監査

要求テスト対応表は、case設計・実test inventory・実行証跡を取得した段階を区別して監査する。

| 監査観点 | 要確認として示す条件 | 記録する内容 |
|---|---|---|
| 要求の未対応 | 対象RQにcaseも、stage/未設計等の状態と理由もない | RQ ID、不足内容、担当する設計/test作業 |
| 孤立case | caseがRQにも承認済み設計判断にも対応しない | case ID、必要性、削除/統合/根拠追加の判断 |
| 重複候補 | 前提・操作・期待値・境界が同じで、別のリスクを検査していない | 重複case ID、統合案、統合後も維持するassertion |
| 共有caseの不足 | 複数RQを一caseで扱い、各RQ固有の期待値・結果を区別できない | 失われるassertion、分割または観測追加案 |
| 実行証跡不足 | 結果があるがrevision、version、target/profile、結果参照等を特定できない | 欠けた項目、再実行または証跡補完の条件 |
| 未解決coverage | 設計gap、後続stage、未実行などをrelease対象から誤ってpass集計している | 対象ID、誤集計の箇所、適用gateへの影響 |

重複候補の統合はcase件数を減らすためだけに行わない。期待値や検証境界が異なる場合は個別assertionを残す。

## 5. 状態の記録

要求test対応表にあるcoverage状態と、実testの実行結果を別の列または別レコードで記録する。文書版0.5の対応表には、以下のcoverage語彙が使われている。

| Coverage状態 | 意味 |
|---|---|
| `設計case割当済` | 論理caseと判定対象を定義した。実装・実行は未証明 |
| `部分/詳細待ち` | case群の一部はあるが、oracle/API/profile等に未設計項目がある |
| `設計gap` | 対象要求のcase詳細設計が不足または未計画 |
| `後続stage` | 次段階で扱う要求。現在のrelease passに数えない |
| `将来候補` | 採用承認前の候補。現行releaseの必須testにしない |
| `process audit` | runtime動作ではなく、計画・承認・記録を監査する |
| `実行記録待ち` | case定義済みだが、対象revision等の実行証跡がない |

coverage状態の語彙を変更する場合は、対応表、集計、release/version gateで使われる意味を一緒に見直す。case結果は別に`pass`、`fail`、`blocked`、`not-run`等で記録し、実際の実行結果と理由を保持する。

## 6. 検証境界の区別

| 検証種別 | 確認対象 | それだけで証明しないもの |
|---|---|---|
| unit-test | 実装単位の局所契約。実装変更と同じrevisionで対応testを実行する | RQ全体、wrapper/platformをまたぐ結果 |
| requirements test | RQの利用者向け観測条件と論理case | 実装完了、release pass |
| ABI/wrapper/integration/system test | 複数層を通る契約、言語間の共通結果、境界固有の失敗 | 未試験target/profileや全release構成 |
| platform/package test | target上の実動作、package consumer、artifact適合 | cross-buildだけによるhardware/runtime support |
| release test | 用途に応じた対象release全体の動作 | 別revision・別profileでの合格 |
| version test | 変更要求と影響範囲、関連回帰の結果 | release全体の用途適合 |

実装・実行するtest IDは、主に検証する境界が識別できるprefixから始める。論理case IDとは別の識別子とし、test IDからcase ID、対象RQまたは承認済み設計根拠、実行証跡を追跡できるようにする。

| 検証境界 | test ID prefix | 使い分け |
|---|---|---|
| unit-test | `UT-` | 一つの実装単位の局所契約を検証する |
| requirements test | `REQT-` | RQの利用者向け条件を主な判定対象とする |
| ABI test | `ABI-` | ABI表現、互換性、境界の拒否・受渡しを検証する |
| wrapper test | `WRAP-` | 特定言語wrapperの型・例外・所有・thread境界を検証する |
| integration test | `INT-` | 複数層・複数部品の接続と連携を検証する |
| system / vertical test | `SYS-` | 製品構成を通したシナリオを検証する |
| platform test | `PLAT-` | OS、target、hardware、backendの実行特性を検証する |
| package test | `PKG-` | packageの構成、選択、導入・consumer利用を検証する |
| release test | `REL-` | release対象の用途適合gateを検証する |
| version test | `VER-` | 変更影響と回帰範囲を検証する |

test IDは`<prefix>-<対象>-<連番>`を基本形とする。複数境界に関わるtestは主たる判定境界のprefixを一つ選び、その他の境界をtest台帳の属性・traceabilityに記録する。テストframework、function名、job名の命名方法は本規則では固定しない。要求テスト対応表の既存`TC-*`は実行test IDではなく論理case IDとして維持し、この変更だけで改番しない。新たに実装するtest IDとの対応を台帳へ記録する。

具体的なscenario、case ID、ABI互換組合せ、targetの合格suiteは要求test対応表、適用設計、Release/Version test規約の担当範囲で管理し、本書で再掲しない。

## 7. 未決範囲

| 未決事項 | 決定・実装先 |
|---|---|
| case IDの追加・改訂・廃止と表の更新手順 | 要求test対応表の管理手順 |
| 各要求のcase詳細、言語/target別実test inventory、重複監査 | 要求test対応表とtest実装計画 |
| Framework、CI provider、runner/job、test command、結果format、retention | 後続のtest/CI実装計画 |
| release support profileと必須gateの頻度・承認担当 | 上位要求・support lifecycle・release運用 |
| RQ-13のmeasurement workload、集計法、数値目標 | RQ変更承認または根拠を伴う後続計画 |
| Test evidenceの機密情報除外・保存先・権限・保管期間 | 適用するsecure-design/release運用。ここでは未承認のsecurity要件を追加しない |

## 承認履歴

| 日付 | 対象版・ID / 範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-10-14 | Requirements-test manifestの作成とDD-23からの再分類 | 利用者 | 作成指示。文書版の承認とは区別 |
| 2026-10-14 | 共通ルールのID省略、代表使用値の記録、検証境界別test ID prefix | 利用者 | 追記指示 |
| 2026-10-14 | 文書版0.2・共通ルール全体 | 利用者 | 承認。承認範囲は版0.2の本文・test ID prefix体系・代表使用値の記録規則 |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 0.1 | 2026-10-14 | DD-23案の検証ルール・case/evidence契約をテスト文書へ再編。詳細scenarioや個別gateは既存case台帳・Release/Version testへ分離 | Codex | 利用者の文書再分類指示。全判断は未承認 |
| 0.2 | 2026-10-14 | 共通ルールの個別IDを除去し、未規定データの代表使用値の記録・traceabilityと、検証境界別のtest ID prefix規則を追加 | Codex | 利用者が2026-10-14に承認 |
