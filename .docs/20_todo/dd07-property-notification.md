# DD-07 Propertyの保存と通知 作業計画

- 状態: 完了（DD-PROP-01～10承認済み、2026-10-08）
- 作成日: 2026-10-07
- 対象: Core管理Propertyの初期公開値型、setter/getter意味論、等価判定、購読通知、終了時処理
- 詳細設計: [DD-07 Propertyの保存と通知](../13_detail_design/07-property-notification.md)
- 全体計画: [詳細設計フェーズ計画](detail-design-phases.md)

## 目的

承認済みP1～P4、BD-01～03、DD-01～06を維持し、5種類の初期公開値型を持つCore管理Propertyの生成・読み書き・変更通知・購読の境界を、4言語で共通にレビューできるようにする。

## 作業手順

1. ADR-0001、Accepted要件、`abi-spec.md` A5/A8/A9/A17/A18、`mvvm-semantics.md` P1～P4/V1～V8、BD-01～03、承認済みDD-01～06を照合し、承認済み意味論と後続へ送られた事項を分ける。
2. BOOL/I64/UTF8/F64/DECIMALのProperty kind、生成・型別getter/setter・Validator callback signatureの設計用symbol、C ABI値表現を提案する。F64とDECIMALの意味をBD-01以上に拡張せず、カスタム値・null・OBJECTを初版へ加えない。
3. 各型の等価比較、no-op setter、成功時の即時確定、validation/copy/allocation失敗時の状態不変、output初期化をP2・A8・DD-05に整合させる。
4. 処理区間、変更Propertyのcoalescing、通知順、購読者順、自己解除・追加購読、callback生成変更の次回配信、外側処理終了時の配信を定義する。P3/P4とBD-02/03を維持する。
5. 有限作業枠の単位・budget案・yield boundaryを定め、DD-03のdispatcher/control priorityおよびBD-03の「完了した配信回の境界でyield」と整合させる。Binding graphに依存する非収束判定・診断の因果範囲はDD-08との責務境界を示す。
6. worker thread、shutdown前後、保留通知の破棄、subscription/callback destroyについて、DD-01～04・BD-36との状態遷移を示す。
7. 判断ID付きの未承認レビュー案を `.docs/13_detail_design/07-property-notification.md` に保存し、正常系・失敗系・順序・終了時の単体確認ケース、後続担当、未決境界を記す。`git diff --check`と参照先・状態の整合を確認する。

## 対象範囲

- 初期公開5値型のProperty kindと型別ABIの設計案
- F64/DECIMALの型別Validator callback signature（登録・評価規則はDD-09）
- 値の保存、取得、型一致、等価比較、setterの原子性
- Property変更通知の処理区間、coalescing、キュー順、callback順
- Subscription登録・解除・自己解除・shutdown処理
- P3/BD-03の有限作業枠、通知保留、UI loopへ譲る境界
- 4言語で共通の振る舞いを確認するケース

## 対象外・後続へ送る範囲

- カスタム値、一般null許容、OBJECT、Collectionを初期Property kindへ加える判断（BD-01で後続）
- Binding初期同期・方向・graph・循環検出と、Binding固有error意味論（DD-08）
- Property Validatorの登録・評価順・View Input/Validation結果の詳細（DD-09）。本書は承認済みsetter失敗との接続だけを扱う。
- library-managedな型変換APIとparse/format規則。文字列↔整数・DECIMAL等の基本変換は将来の設計候補として記録する。本フェーズでは変換APIを定義せず、Property setterに暗黙変換を追加しない。
- Async Commandの状態Propertyや進捗（DD-10）
- C++/C#/Rust/Python固有の型・例外・delegate管理（DD-18～21）
- UI loop adapter実装とplatform固有のwake挙動（DD-16）
- 実装header、完全な配布API、性能目標・計測結果

## 承認記録

利用者は2026-10-08にDD-07を承認した。DD-PROP-01～10を同日承認済みとして詳細設計へ反映した。将来の内蔵値変換API・parse/format規則は後続設計に残す。
