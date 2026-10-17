# 詳細設計 / 分野別正本・旧設計ID追跡廃止

- 更新日: 2026-10-17
- スコープ: `.docs/13_detail_design/` の構成、文書管理、後続編集の入口
- 計画・結果: [詳細設計の実再編集](../20_todo/detail-design-category-restructure.md)
- 正本入口: [詳細設計](../13_detail_design/README.md)

後続の利用者「全部承認します」により、現行19本文と入口の各1.0版全文を2026-10-17承認済みとして各文書へ記録した。その後[基本設計の同様の再編集](basic-design-category-restructure.md)を実施し、DDの上位BD参照を新しい分野別本文へ更新した。本文の動作・コードブロックを承認記録・根拠参照更新で変更しない。

現在は[DDクラス設計補足](dd-class-design-supplement.md)により19本文と入口が1.1改訂案。既承認1.0本文は保持し、クラス補足は確認待ちと区別している。適用中の承認済み内容と追加した型名/メソッド分担を混同しない。

利用者は計画だけの変更ではなく、旧root文書のアーカイブ移動と新しい分野別文書の実作成を明示した。旧42文書を `archive/2026-10-17/` へ移動し、6分野・19本文＋READMEを作成した。指定5分野にビルド・配布を追加。約20冊の責務単位で読み切れる範囲にまとめ、旧ファイル単位を維持しない。

新設計は要求からソフトウェアへ反映するDDの正本。設計独自ID・項目承認表・旧ID追跡表・互換リンクは使わない。RQと上位BDへの根拠参照、文書版全体での承認を維持する。機能上のID、実装unit ID、test/case IDは別であり廃止しない。AGENTS.mdも今回指示へ合わせて更新した。

archiveはローカル保存専用で、設計配下の.gitignoreによりGit除外。旧設計と以前の旧ID棚卸し・構成レビュー・計画はここに退避した。`.docs/design/` 配置案は採用していない。過去の計画・reviewのリンクを現行設計の入口にしない。

言語wrapperは4言語別と解析器、UIはTree/Layout・Input/Focus・TextBox/Validation/IME・Control、Platformは抽象化とOS内部レイヤーに分冊した。Controlの公開操作とWindow close requestはUI、Scene/peer/renderer/OS providerはPlatformに置いた。共通ABIはランタイム/オブジェクト分野にある。

現役BD/SD/testのリンクを張り直し、unit-test台帳の設計参照を文書path/version/sectionのschema 2へ更新した。新しい台帳validatorや製品コードを実装したわけではない。旧設計IDに戻る参照を再導入しない。

検査では新20文書の880ローカルリンク/anchor、165表、162 fenceにエラーなし。全81コードブロックを保持し、旧42文書はhash一致。製品build/test、Mermaid描画は未実施。再編集前からあった他の作業を保全し、commit/pushは行っていない。
