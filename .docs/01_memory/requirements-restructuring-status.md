# 要求仕様 / RQ文書再編・承認管理・test対応の状態

- 確認日: 2026-10-12
- スコープ: `.docs/11_requirements/` の要求参照、変更承認、要求test・設計へのtraceability

## 現行入口と承認境界

- 要求入口は利用者指定の綴り `reqirements-specification.md`。RQ-01～16の16章に分割し、要求IDは `RQ-<2桁章番号>-<3桁連番>` とした。
- 生産順は RQ → BD（design = basic-design）→ DD（API具体化を含む）/ SD → software + 対応unit-test。要求testはRQから導出する検証成果物として作成し、BDと検証条件の整合を確認する。参照リンクはこの依存を逆にたどり、下流文書から上流の根拠へ向ける。要求仕様から下流成果物へのリンクは置かない。RQ章への目次ナビゲーションは例外。
- 旧本文にすでに記載された承認済み要求事項を継承・要求側に集約した。今回の章分割・管理方針は利用者指示に基づく。再編集版全体と受入確認案はレビュー待ち。将来候補の採用や未決事項の採用は承認していない。
- 旧Accepted ADRの技術的制約は維持する。不整合は差異・要求変更/ADR変更/設計変更の対象を記録して承認し、下流設計を理由に要求を暗黙に変えない。

## 移行と検証

- 旧パス `mvvm-ui-library-requirements.md` は旧章番号からの移行入口。旧本文は `archive/mvvm-ui-library-requirements-20261012.md` に保存した。
- `requirements-id-migration.md` が暫定REQ ID196行と正式RQ ID195件の対応を保持する。旧REQ-6.2-16とREQ-6.3-11は同一の共通モデル先行要求としてRQ-09-037に統合した。
- `.docs/30_release_test/requirement-test-matrix.md` は正式RQへのリンクと個別caseの逆引きへ更新した。論理case88件にRQリンクを持たせたが、実test・実行証跡は未登録。test matrixからDD-23への下流リンクは除いた。
- secure-designの入口は `.docs/14_secure_design/00-security-design-map.md`、段階案は `.docs/20_todo/secure-design-phases.md`。内容・脅威範囲・対策は未着手・未承認。
- 詳細設計各章の本文・承認履歴は今回編集していない。旧要求リンクは互換入口を経由して現行章へ辿れる。
- 作業計画は `.docs/20_todo/requirements-restructuring.md`。

## 次回参照時

総目次→対象RQ章で要求本文・対象stage・承認履歴・未決の所管を確認し、下流の要求test・設計文書から必要なRQを参照する。再編集版のレビュー待ちと、継承要求が承認済みであることを混同しない。
