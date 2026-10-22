# DD-10 作業計画: 非同期Command

- 状態: 完了（DD-CMD-01～12内容承認済み、2026-10-11）
- 対象文書: `.docs/13_detail_design/10-async-command.md`
- 前提: DD-01～09、C1～C4、A21～A24、BD-26、DD-DSP-01～08、DD-CB-01～09、DD-ABI-01～10
- 承認境界: 利用者は2026-10-11にDD-10の内容を承認した。本計画はDD-10の承認反映までを扱い、DD-11以降の着手承認を含まない。

## 作業項目

1. ADR-0001、要件、非同期Commandの基本設計・ABI本文と承認済み判断を確認する。
2. Command生成、実行可否、operation ID、状態遷移表と図、start callback失敗時の戻り値・観測を具体化する。
3. cancel tokenのスレッド安全な照会、キャンセル要求とworker実結果の区別、完了報告のUI thread/`runtime_post`境界を定める。
4. 古いID・重複完了・完了後progress・post受理後shutdownの競合時に、状態とclosure所有権がどうなるか整理する。
5. failure snapshot、状態/can-execute Property、Binding/通知との接続をDD-07/08・DD-ABI-06/07と整合させる。
6. close、画面終了、shutdown時のworker非待機とwrapper側参照寿命を区別し、DD-02等へ委ねる所有範囲を明示する。
7. 進捗専用APIがないこと、BD-26の集約方針、worker管理・screen ownership等の未決境界を明記する。
8. 判断ID付きレビュー案と確認ケースを保存し、関連計画・進捗記録を更新する。
9. `git diff --check` と参照先を確認し、利用者承認を受けてフェーズ計画・進捗記録を完了状態へ更新した。

## 終了条件

- DD-10の判断IDと承認結果が保存され、承認済み契約と未決事項を区別している。
- 利用者の承認を同じ詳細設計文書へ反映し、フェーズ計画・進捗記録を更新した。
- DD-11は未着手のままにする。
