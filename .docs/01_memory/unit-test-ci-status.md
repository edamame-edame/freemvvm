# Unit-test / 実装単位・GitHub Actions検証方針の状態

- 確認日: 2026-10-12
- スコープ: 実装単位とunit-testの対応、変更ファイル起点のCI検証

## 現行案

- 要求仕様の総目次はsoftwareとunit-testを同段階の対応成果物として示し、GitHub Actionsがpush/PR更新後に変更実装単位のunit-testと影響回帰を起動する方針を記載している。
- unit-testのreview draftは `.docs/32_unit_test/unit-test-strategy.md`、作業範囲と未決の境界は `.docs/20_todo/unit-test-strategy.md` を参照する。
- 実装単位はファイルと同義にせず、unit ID、source paths、適用RQ・BD/DD、test design/case ID、test path/command、影響test、commit SHA/resultを対応づける案。
- GitHub Actionsの変更選択はpathだけでなく影響依存も考慮し、対応情報のない変更を黙ってskipしない。ローカルの未push編集はGitHub Actionsで検証できない。
- 2026-10-12時点でrepository rootに `.github/workflows`、実装source tree、language manifestは確認できていない。実行job/framework/runner/mappingは未決で、workflow YAMLは未作成。

## 参照時の注意

unit-test案は未承認であり、TDD手法、unit ID形式、framework、coverage閾値、GitHub Actions構成を確定事項として扱わない。要求case・release/version testとの役割分担を維持する。
