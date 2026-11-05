<!-- scope: Development CLI and unit catalog modularization -->
# freemvvm: 開発CLI・unit台帳の現状

- カテゴリ: 開発環境 / `tools/dev.py` / unit catalog
- 対象: [開発CLI・unit台帳の分割計画](../20_todo/development-tooling-modularization.md)

## 現状

- `tools/dev.py`は単一entrypoint内にargparse command一覧、共通環境・report処理、ABI/platform gate、Rust/C++/C#/Python wrapper test dispatchを持つ約2,740行のPython file。
- `tools/units.yaml`はJSON互換YAML schema 2の単一catalogで、2026-11-06のversion branchで170 unit・約10,773行。unit records内の`config`がroot catalog pathを繰り返し参照する。
- `.docs/32_unit_test/implementation-test-inventory.md`と複数計画・DEVELOPMENT文書がroot catalogを説明・参照するため、split時は全consumerを検索する。
- `python tools/dev.py ...`はDEVELOPMENTとGitHub Actionsから直接呼ばれる。分割後も互換entrypointと全command名を維持する必要がある。

## 分割時の原則

- `tools/dev.py`と同名packageの衝突を避け、helper packageには`tools/dev_support/`のような別名を使う。
- 共通process/environment/report helperと各大区分・言語固有handlerを分け、異なる言語branchが同じdispatcherを編集しない形にする。
- Unit recordsはunit単位まで細分化せず、Coreの責務カテゴリ、ABI、Platform、build/package、wrapper言語のowner単位でshard化する。
- Root manifestは小さく保ち、shardを決定的に発見して重複ID・未解決依存を検査する。提案中のrecord構造は`unit_name`（現行unit ID）・`path`・`cases`を中心にする。これは構成例であり、現行のJSON互換YAML形式を維持してparser依存を追加しない。
- `path`は現行のsource/config/test-design等を役割別に保持する。`cases`にはUT ID・commandと要求/設計/release test caseの対応を保持し、状態や依存関係はunit-level metadataに置く。schema2からの移行はunit ID・record・依存・traceabilityの全件比較で検証し、unit-test文書とCIの参照先も更新する。
- この新しいshapeは計画0.3の提案であり、文書単位の承認前に確定仕様や実装済みschemaとして扱わない。
