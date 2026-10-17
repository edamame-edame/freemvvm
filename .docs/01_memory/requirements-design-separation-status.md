# 要求・設計整理 / 全BD・DD走査と後続の境界

- 最新状態: [2026-10-17 全案適用](requirements-design-separation-application-status.md)

以下はそれ以前の進行記録。現在の承認・適用状態は上記を参照する。


- 更新日: 2026-10-12
- スコープ: 要求・BD・DDの混在整理、ID移行計画
- 計画正本: [責務分離とID整理計画](../20_todo/requirements-design-separation.md)
- 調査台帳: [全66文書の調査台帳](../20_todo/requirements-design-separation-inventory.md)

## 現在の状態

利用者は全BD/DDを走査し、要求・設計の分離対象をリストアップしてto-doにするよう指示した。12_designの42件（BD00～36と基礎5文書）、13_detail_designの24件を確認し、全件台帳と横断課題10件、SEP-01～07の段階計画を作成した。今回は調査と計画のみ。仕様本文の移管・ID再採番・新しい要求採用は未実施であり、次段階は着手指示を受けて開始する。

## 後続が守る事項

- 現行RQ01～07/09～16の0.3版は承認済み。候補・未決の採用は別。RQ08廃止とID非再利用を維持する。要求索引は0.5へ更新し、0.3版全体承認と今回のロール指定を別記した。
- 外部保証と実現方式を文単位で分ける。技術制約や観測できる設計既定値を一律に要求へ移さない。既にRQへ集約された製品方針は二重定義を減らす。
- A11/A14/A19とDD-LIFE-04/05・DD-PROP-08にrelease/解除の差がある。新しい文書を自動優先せず、要求保証と承認範囲を比較して解決する。
- BD36-01/02/04の方針確認と36-03の未確認提案を区別。Python通常終了の保証とhook/coordinatorの未決方式、GIL非依存と任意thread化を混同しない。
- DD22はJCS/Ed25519/SHA-256の提案範囲を承認済みと§11に記録。候補本文だけで未承認に戻さない。exact parameters等は未決。旧DD23案は2026-10-14に詳細設計対象外へ再分類し、Requirements Test Manifestへ移管。同manifest文書版0.2と共通ルール案は2026-10-14利用者承認済み。
- designとbasic-designは同一のBD。要求testはRQに基づく検証成果物であり、BDの前に別のdesign工程を設けない。関係図の参照方向はBD→RQ、要求test→RQとする。
- 最新の利用者指定: 利用者から見える要求上の例外・保証・結果はRQ。BDは層の責務、主要data/ownership flow・stateなどRQを満たす上位手段、DDはclass/interface・method/signature/call flowと詳細なdata/ownership/lifecycle/stateを記す。APIのクラス/interface図・メソッド名・signature・呼出し手順はDDに統合し、利用者に保証する動作・例外・結果はRQに置く。独立API工程を前提にしない。計画済みDD完了後の全項目整理という作業順は維持し、未決APIを今回確定しない。SD/要求test/unit-testの既存計画を参照する。
- 要求の承認はRQで完結し、設計承認を読み替えない。参照は下流から上流。旧新ID対応は計画側の台帳、過去承認は追記履歴で保存する。

## 役割反映の結果

2026-10-12の追加指示で、要求索引・RQ-16、整理計画/台帳、DD/SD計画・SD入口、AGENTSの参照手順を更新した。APIを独立成果物とする前提をDD内の具体化へ変更した。66文書の個別移管・ID改番、DD-23等の未承認設計の採用は未実施。過去の別章方針・承認記録は履歴として保存する。

## 要求と設計の記述境界

利用者から見える例外・除外・保証・結果は要求に属する。BD/DDではその要求を満たす責務分担、層間のデータと所有の流れ、処理手順、状態遷移、interfaceと呼出し関係を抽象度に応じて記す。設計上の都合だけで外部動作や保証を追加・例外化しない。

## SEP-01現在の作業

2026-10-17追記: 利用者の一括作成指示により、SEP-01～07の全案件レビュー案を作成した。最新の作成状態・件数・承認境界は[一括レビュー状態](requirements-design-separation-batch-review-status.md)、案の入口は[全案件索引](../20_todo/requirements-design-separation-review/index.md)。以下は一括作成前のX01進行記録として保持する。

- X01（公開handleのreleaseとregistrationの明示解除）は、[承認されたX01案に基づきRQ-05/07、ABI、DD-02/07/08/09の改訂案](../20_todo/requirements-design-separation-x01-release.md)を更新して再整理。承認済みABI A11/A14/A19とDD-LIFE-04/05の差異を確認し、RQ-05-002/006・RQ-07-011で契約が明示されていない点を記録した。
- RQ-05 v0.8は2026-10-13利用者承認済み。全IDを利用者の能力・外部保証中心に記述し、受入確認をrequirements-testへ分離した。requirements-test v0.5のRQ-05-001～009とTC-LIFE-01/TC-OWN-01/TC-CALLBACK-01の対応範囲も同日承認済み。対応表の他範囲は未承認である。DD-02 v0.3のDD-LIFE-03～05は2026-10-13承認済みで、公開handle token releaseとregistration解除の内部sequence、owner参照の更新、in-flight完了後のcleanupを規定した。次段階としてDD-07/08/09 v0.3でsubscription・Binding・Validatorのowner、解除・teardown・shutdown cleanupを整合させたレビュー案を作成した。各X01改訂判断は利用者レビュー待ち。ABI改訂案と対応表の残りを確認し、X01全体の対応確認を完了してからX02へ進む。
