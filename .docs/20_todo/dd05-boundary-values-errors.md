# DD-05 値の受渡しと失敗契約 作業計画

- 状態: 完了（判断承認済み、2026-10-07）
- 作成日: 2026-10-07
- 対象: C ABI境界における値・buffer・error handle・失敗時出力の共通契約
- 詳細設計: [DD-05 値の受渡しと失敗契約](../13_detail_design/05-boundary-values-errors.md)
- 全体計画: [詳細設計フェーズ計画](detail-design-phases.md)

## 目的

4言語のwrapperが同じnative ABIを安全に使えるよう、既承認の値受渡し・error契約を整理し、未定義の境界規則に限定したレビュー案を作る。

## 作業手順

1. ADR-0001、Accepted要件・設計骨格、ABI A2/A5～A8/A15/A18、BD-01/27、DD-01～04の状態と判断本文を確認する。
2. UTF-8入力とcaller buffer出力のpointer/length、借用期間、copy、容量不足、出力初期化を一つの契約としてまとめる。
3. `ui_status`と任意`out_error`、不変error snapshot、errorを作れない場合、API失敗時の副作用・部分出力の扱いを提案する。
4. 判断ごとに上位出典、検証ケース、未決境界、担当後続フェーズを記し、DD-05文書をレビュー案として保存する。
5. `git diff --check`とリンク先・判断状態の整合を確認し、利用者承認後にDD-05文書・全体計画・承認状況メモを更新する。

## 対象範囲

- 値・byte列の境界での借用とcopyの時点
- UTF-8、pointer/length、caller buffer、buffer不足時の共通規則
- 成功・失敗時におけるrequired outputとoptional error output
- statusとerror object snapshotの関係、error生成失敗時の扱い
- 検証・copy・確保失敗時に状態や通知を部分変更しないための共通境界

## 対象外・後続へ送る範囲

- F64/DECIMAL Propertyの関数署名、DECIMALのwire encoding、型別生成/read/write/Validator ABI（DD-07およびwrapper設計）
- Collectionやその他の配列・snapshot ABI（Collection専用設計と上位判断）
- error domain/codeの全一覧、数値割当、cause/diagnostic取得API（DD-06/各機能設計）
- 各言語の例外型・Result型、文字列変換APIの正式な公開形（DD-18～21）
- Property/Binding/Validation/Command個別の成功・拒否意味論

## 承認境界

利用者の2026-10-07の承認を受け、DD-VAL-01～07をDD-05文書に反映した。承認範囲は本書記載の判断に限り、後続設計の判断や実装を含まない。
