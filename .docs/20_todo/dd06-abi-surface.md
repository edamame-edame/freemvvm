# DD-06 ABIシンボルと互換性 作業計画

- 状態: 完了（DD-ABI-01～10承認済み、2026-10-07）
- 作成日: 2026-10-07
- 対象: 公開C ABIのsymbol/version/feature面、共通status/error catalog、UI loop接続、shutdown要求結果の表現
- 詳細設計: [DD-06 ABIシンボルと互換性](../13_detail_design/06-abi-surface.md)
- 全体計画: [詳細設計フェーズ計画](detail-design-phases.md)

## 目的

A1～A32およびBD-27/28に沿ってC ABIの公開面を識別可能にし、追加・修正・破壊的変更の互換性境界と、DD-03から送られた未決の境界hook/statusをレビュー可能にする。

## 作業手順

1. ADR-0001、Accepted要件/design overview、ABI判断とC宣言、BD-27/28、DD-01～05、DD-03 §4/7を読む。
2. symbol major、3要素runtime version、feature query、status/error domainの安定性と未知値処理を提案する。
3. 利用者定義domain/codeの自動採番と既存code安定性の要件を整理する。自動採番はsource/build-timeの補助に限定し、機能固有のerror codeと意味論はDD-07～10等の担当設計に残す。
4. `UiLoopBridge`接続点とshutdown要求が受理された場合の返却・完了観測を、DD-03の承認済み動作と整合する形で提案する。
5. symbol追加/旧runtime組合せと旧header/new runtimeの互換検証表を作る。既存C宣言はレビュー資料として識別する。
6. DD-ABI判断IDを付けたレビュー案を保存し、参照先・上位契約・`git diff --check`を点検して利用者確認を受ける。

## 対象範囲

- ABI symbol majorとnative runtime MAJOR.MINOR.PATCHの対応
- major/minor/patchごとの互換性規則
- symbol追加とfeature IDの識別・未知値処理
- 共通status値とerror domainの初版catalogおよび拡張規則
- 利用者定義error codeをstatusへ追加せずdomain/codeで運ぶ方針と、自動採番の安定性要件
- UI loop bridgeをruntimeへ接続する公開境界
- shutdown呼出しの同期・非同期要求と受付済み／完了済みの観測
- ABI compatibility/version testの最低条件

## 対象外・後続へ送る範囲

- 各機能の完全なAPI関数一覧・実装済みheader
- Property/Binding/Validation/Command固有のerror code・結果意味論（DD-07～10等）
- cause/diagnostic取得APIやmessage localization
- wrapper packageのversion、loader検索順、runtime fetch/cache/provenance（DD-18～23）
- OSごとのhost event loop接続実装（DD-16）

## 承認記録

利用者は2026-10-07にDD-06を承認した。DD-ABI-01～10を承認済みとして詳細設計へ反映した。C宣言は設計資料であり、配布用headerまたは実装済みAPIとして扱わない。未決境界と後続フェーズは全体計画に従って継続する。
