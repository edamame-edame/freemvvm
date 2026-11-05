# 開発CLI・unit台帳のモジュール分割計画

- 計画版: 0.3
- 状態: 計画承認済み（利用者、2026-11-06）。実装着手前
- 対象version: `0.0.1`
- 対象branch: `change/dev-unit-module-layout`（`origin/version/0.0.1`起点）
- 関連: [開発環境bootstrap・toolchain](development-environment-bootstrap.md)、[実装unit台帳](../32_unit_test/implementation-test-inventory.md)、[単体試験方針](../32_unit_test/unit-test-strategy.md)、[Git規則](../02_git/rule.md)

## 背景

現状の`tools/dev.py`は約2,740行で、CLI parser、共通環境準備、report生成、全bootstrap gate、ABI/platform処理、4言語wrapperのテストcommand dispatchが集中している。新しい言語・機能を別branchで追加するたび同じdispatcherが競合しやすい。

`tools/units.yaml`はschema 2のJSON互換形式で約10,773行・170 unitを1ファイルに持つ。unit追加や状態更新が同じ巨大ファイルへの変更になり、merge conflictとGit上でのdiff閲覧を難しくしている。

## 目的

- `python tools/dev.py <既存command>`とCIの呼び出し方を維持したまま、CLIを共通入口と責務別moduleへ分ける。
- テスト・ツールの実行処理を大区分と対象言語ごとのmoduleに分け、Python/Rust/C++/C#の独立作業で共通dispatcherの編集を避ける。
- unit定義を責務カテゴリ別のshardへ移し、言語別wrapperのunit更新をそれぞれのshard内で完結させる。
- unit ID、要求/設計/試験対応、test command、実行結果report、CI gateの意味を維持する。

## 構成案

### CLIと実行module

`tools/dev.py`は互換性を保つ薄いentrypointとして残す。`dev.py`と同名のpackage directoryを作るとimport解決が曖昧になるため、package名は`tools/dev_support/`とする。

```text
tools/dev.py                         # 既存のCLI entrypoint
tools/dev_support/
  __init__.py
  cli.py                             # command registry、argparse、dispatch
  environment.py                     # venv/toolchain/environment準備
  process.py                         # subprocess実行・MSVC environment
  reports.py                         # verification report共通処理
  commands/
    __init__.py
    bootstrap.py                     # doctor / check / format / minimum
    abi.py                           # C/C++ ABI smokeとheader probes
    platform.py                      # Linux Platform Docker等
    wrappers/
      __init__.py
      rust.py
      cpp.py
      csharp.py
      python.py
```

共有責務は`environment.py`・`process.py`・`reports.py`へ置き、言語module間の相互importや重複実装を作らない。各moduleが担当commandを明示登録し、`cli.py`は既存command名とhandlerを結びつける。

### Unit catalog

`tools/units.yaml`は小さいroot manifestとして残し、JSON互換YAML記法でschemaとshard globだけを記録する。unit records本体はowner/category別の`.yaml` shardsに置く。データ形式は現行どおりJSON互換を維持し、以下は項目構造の例として示す。各recordの`unit_name`には現行unit IDを保持し、移行時に識別子を変更しない。

```yaml
schema: 3
unit_shards: "units/**/*.yaml"
```

shard内では、`unit_name`・`path`・`cases`を中心に読みやすくまとめる。`path`は現行のsource/config/test-design等のパスを役割別に保持し、`cases`は試験ケースごとに既存のUT ID、実行command、要求・設計・リリース試験との対応を保持する。状態や依存関係などケースに属さないunit metadataは別fieldとして残す。

```yaml
units:
  - unit_name: UNIT-CORE-085
    state: complete
    scope: resolve-control-text-once-and-share-revision-stamped-snapshot-with-layout-and-scene
    path:
      source:
        - crates/core-runtime/src/lib.rs
        - crates/core-runtime/src/runtime.rs
      config:
        - Cargo.toml
        - Cargo.lock
      test_design: .docs/32_unit_test/ui.md
    depends_on:
      - UNIT-CORE-084
      - UNIT-PLATFORM-003
    cases:
      - case_name: platform-default-text-snapshot
        unit_test_id: UT-TEXT-PLATFORM-DEFAULT-001
        command:
          - cargo
          - test
          - -p
          - freemvvm-core
          - platform_default_text_snapshot_is_shared_by_layout_and_visual_scene
          - --locked
        requirements:
          - RQ-09-004
          - RQ-09-006
        design:
          - path: .docs/13_detail_design/ui/controls.md
            version: 1.4 approved
            section: Text-family immutable display snapshot
        release_cases:
          - TC-LAYOUT-01
```

```text
tools/units.yaml                     # schema + unit shard pattern
tools/units/
  core-runtime.yaml
  core-ui.yaml
  core-scene-text.yaml
  abi.yaml
  platform-linux.yaml
  platform-windows.yaml
  build-package.yaml
  wrappers-rust.yaml
  wrappers-cpp.yaml
  wrappers-csharp.yaml
  wrappers-python.yaml
```

分類は実装所有境界に合わせ、unitごとにファイルを作らない。core領域は変更頻度と責務を見て数個のshardに分け、wrapperは言語単位で独立させる。shardの自動検出により、新しい言語shard追加のたびに共通unit file一覧を編集しない。

### Unit loaderと検査

loaderはroot manifestのglobを読み、全shardのschema、`unit_name`一意性、`depends_on`解決、path/command形式を検査する。`unit_name`は既存のunit IDをそのまま保持する。`cases`の各項目は試験を一意に識別し、既存の`cases`（release test case IDs）と`tests`（UT ID・command）およびunit-levelの要求/設計対応を欠落なく移行できる構造とする。既存データにケース単位でない要求/設計対応がある場合は、移行前に対応するcaseへ割り当てるかunit metadataとして残し、意味を推測して変更しない。

この項目構造はJSON互換YAMLでも表現できるため、parser依存は追加しない。現行のJSON読込契約を維持し、shardもJSONとして読み込めるJSON互換YAMLにする。

既存unit recordの意味・fieldを変えずに移行し、unit ID集合と各unitの内容が移行前後で一致することを機械的に比較する。文書とツールが`tools/units.yaml`を読む箇所を全件検索し、同じloader契約へ更新する。

## 作業フェーズ

| 順 | 作業 | 完了条件 |
| --- | --- | --- |
| 1 | command/manifest baseline固定 | 現行CLI command名・report schema・170 unitのID/recordをsnapshotし、既存参照者を一覧化 |
| 2 | unit catalog loader | root manifestとshard読込、決定的結合、unit_name重複・依存欠落・不正path検査を実装し、loaderの単体試験を追加。既存JSON読込依存を維持 |
| 3 | manifest shard移行 | recordsをCore/ABI/Platform/build-package/言語別wrapper shardへ移し、`unit_name`集合・依存・case/command・要求/設計対応がbaselineと同じことを確認 |
| 4 | CLI共通基盤の分離 | environment/process/report機能を`dev_support`へ移し、既存entrypointのCLI挙動を保つ |
| 5 | command module分割 | bootstrap、ABI、platform、Rust/C++/C#/Python wrapperのcommand handlerを責務別moduleへ移し、各module境界のtestを実施 |
| 6 | 文書・CI・差分確認 | DEVELOPMENT、unit-test文書、CI/他consumerの参照を更新。docs link check、manifest validation、全言語focused test、`check`、`minimum`を実行 |
| 7 | 集約レビュー | 変更対象一覧、unit移行結果、既存CLI互換、同一revision CI、残課題を照合し、まとめてversion向けPRを作成 |

## 完了条件

- `tools/dev.py`から現行すべてのcommandが同じ名前・引数・exit status・report semanticsで使える。
- `tools/dev.py`は互換entrypointとbootstrapだけを担当し、言語別handlerは専用moduleに閉じる。
- unit shard移行前後で`unit_name`（現行unit ID）が同一であり、全170 recordの意味・依存・path・RQ/設計/TC・UT ID/commandを失わない。
- `tools/units.yaml`は小さいmanifestとなり、wrapper言語ごとのunit追加が別shardで行える。
- shard追加/読込時にduplicate IDや未解決dependencyを検出できる。
- docs link check、manifest validation、対象unitのfocused test、`tools/dev.py check`、`tools/dev.py minimum`、Windows/Ubuntu CIが成功する。
- 新規コード・unitを別責務の共通fileへ集約しない。現行の未追跡Windows input bridge draftをこの作業へ混ぜない。

## 境界と注意

- この計画は開発用CLI/テスト台帳の構成変更に限る。製品Core、C ABI、wrapper public API、unit ID、要求/設計承認範囲を変更しない。
- CI workflowから呼ばれる`python tools/dev.py ...`は互換entrypointとして残す。
- 既存CLIのcommand aliasやreport形を整理目的で改名・統合しない。
- 巨大な一括書換えにせず、loader導入、shard移行、command module抽出を独立検証可能なcommit単位にする。Gitのrename-aware diffを確認し、閲覧不能な巨大差分にしない。
- 各phaseは対応する実装単位・unit-testと同じrevisionで検証する。計画承認済みだが、実装は独立した検証単位で進める。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-06 | 計画0.1 | superseded | `dev.py`とunit catalogの共有編集競合を減らす初期構成案 |
| 2026-11-06 | 計画0.2 | superseded | unit recordsを`unit_name`・`path`・`cases`中心に再構成する案を追加 |
| 2026-11-06 | 計画0.3 | 承認済み | 構成例として扱い、JSON互換形式を維持。parser依存追加を除外。利用者承認 |
