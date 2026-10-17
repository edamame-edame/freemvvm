# DD-08 Binding詳細設計 / 承認と未決候補

- カテゴリ: 詳細設計・Binding
- 更新日: 2026-10-14
- 対象: `.docs/13_detail_design/08-binding-graph.md`

## 作業時に確認したこと

- DD-BIND-01～09の基礎判断は2026-10-09承認済み。DD-BIND-04のX01本文改訂とDD-BIND-10～15の追加具体化候補は別途レビュー待ちとして扱う。
- DD-BIND-10～15は、因果循環証拠、長期保留診断、診断/回復API、128件slice計数単位、配信中graph世代、Binding error codeを具体化するレビュー候補である。承認前に確定仕様へ昇格しない。
- 保留事項を残す場合、上位設計だけでは決まらない理由、候補、推奨案、影響をDD上で提示する。
- 型変換は現行の同kind Binding要件から外れるため、詳細設計の未決ではなく範囲外。追加するなら要求とBDを先に変更する。
- Binding error domainはDD-06の共通ABI catalogと整合させる。以前のDD-22参照は誤記。
- 通知sliceはcallbackの個数ではない。DD-PROP-09では最大128件の完了Property delivery itemを扱い、各Property itemはsubscriber snapshot内のcallback全員が完了する単位。DD-BIND-08ではBinding propagation itemも同じ完了作業枠へ含める。
- 2026-10-14にDD-BIND-10候補(b)、DD-BIND-12/14/15推奨案が承認された。DD-BIND-11は利用者設定WatchDog・無効化方針・初期値5秒まで承認済み。DD-BIND-13の計数item定義も承認済み。
