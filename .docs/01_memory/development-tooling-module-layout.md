<!-- scope: Development CLI and unit catalog modularization -->
# freemvvm: 開発CLI・unit台帳の現状

- カテゴリ: 開発環境 / `tools/dev.py` / unit catalog
- 対象: [開発CLI・unit台帳の分割計画](../20_todo/development-tooling-modularization.md)

## 現状

- `tools/dev.py`は互換entrypointとして全61 commandを維持する。共通environment/process/report helper、ABI/platform、bootstrap、Rust/C++/C#/Python wrapper command handlerは`tools/dev_support/`へ抽出済み。
- `tools/units.yaml`はschema 3の小さなJSON互換manifest。170 unitを12個のowner/category shardへ移行済み。
- `.docs/32_unit_test/implementation-test-inventory.md`と複数計画・DEVELOPMENT文書がroot catalogを説明・参照するため、split時は全consumerを検索する。
- `python tools/dev.py ...`はDEVELOPMENTとGitHub Actionsから直接呼ばれる。分割後も互換entrypointと全command名を維持する必要がある。

## 分割時の原則

- `tools/dev.py`と同名packageの衝突を避け、helper packageには`tools/dev_support/`のような別名を使う。
- 共通process/environment/report helperと各大区分・言語固有handlerを分け、異なる言語branchが同じdispatcherを編集しない形にする。
- Unit recordsはunit単位まで細分化せず、Coreの責務カテゴリ、ABI、Platform、build/package、wrapper言語のowner単位でshard化する。
- Root manifestは小さく保ち、shardを決定的に発見して重複ID・未解決依存を検査する。提案中のrecord構造は`unit_name`（現行unit ID）・`path`・`cases`を中心にする。これは構成例であり、現行のJSON互換YAML形式を維持してparser依存を追加しない。
- `path`は現行のsource/config/test-design等を役割別に保持する。`cases.release`は旧logical TC配列、`cases.unit`はUT ID・commandを持つ。RQ/DD対応は旧形式と同じunit-levelに残し、caseごとの関係を推測しない。
- 計画0.3は2026-11-06に承認された。実装中のloaderはschema 2を互換読込し、schema 3 shardではunit_name重複・依存欠落・path/command形を検査する。schema 2の`UNIT-WRAP-CS-BIND-001`からのdangling dependencyは、scopeが一致する`UNIT-WRAP-CPP-007`へschema 3移行時に修正して明記する。
- `tools/dev.py`のbaselineは2026-11-06時点で2,757行・61 commands。通常reportは`.local/verification/<command>.json`へ出し、Linux Docker provisional commandは独立schema/pathを使う。command一覧・report field・status意味を分割前後で比較する。
- schema 3移行では170 unitを12 owner/category shardに分けた。`cases.release`は旧TC配列、`cases.unit`はUT ID/commandを保持し、requirements/designはunit-levelのままにする。config pathへroot manifestと所属shardを追加し、旧recordを機械的に再構成して比較する。
- phase 4ではbootstrap environment構築、MSVC/child process実行、JSON report書込を独立helperへ抽出する。`tools/dev.py`からは互換名でimportし、CLIのreport field・command/result記録を変えない。`test_dev_support.py`はhost environmentの非変更、PATH順、process result field、JSON encoding/newlineを検証する。
- CI failure diagnosis: `tools/dev.py --help` can run without the pinned venv, but full bootstrap checks require a checkout-local `.venv`. If this managed worktree has no Ruff, run the Ruff binary from the primary checkout against this worktree by passing changed paths; do not edit or stage the primary checkout.
- Command handler extraction begins with `dev_support/commands/abi.py`. Its ordered `COMMANDS` tuple feeds argparse choices, and its handler retains the existing CMake targets and gate names. Handler tests should stub the shared `run` function and assert exact argv/report labels without invoking compilers.
- Rust wrapper handler extraction keeps `COMMANDS` in the original CLI order and reproduces the native C ABI build plus OS-specific library search environment before dispatch. Fake-run tests verify each consumer/test command and preserve the host mapping without compiling wrappers.
- Platform Docker handler tests stub git and docker subprocesses against a temporary root, and assert both the provisional report contract and blocked error reporting without running Docker.
- C++ wrapper command extraction keeps all 12 existing command names and report gate strings in `dev_support.commands.cpp`. Fake `run` tests assert CMake targets, CTest filters, and the relocatable package consumer sequence without invoking the toolchain.
- C# wrapper command extraction keeps runtime unit tests and native C ABI consumer commands grouped by gate label. Fake `run` tests create a temporary native library file and assert platform-specific path selection, consumer args, and existing report labels.
- Python wrapper command extraction keeps individual pytest gates, C ABI consumers, custom `CARGO_TARGET_DIR`, and the Linux-only window peer smoke in one module. Fake `run` tests cover command argv, native artifact path, `PYTHONPATH`, and host rejection behavior.
- Bootstrap command extraction keeps `doctor`, `check`, `format`, and `minimum` in the original CLI order. The aggregate `check` command sequence is copied as a single handler and covered by an ordered gate-name contract test.
- Command report scopes are stored beside their command handler and combined in `dev_support.command_scopes`; unmapped commands retain `development-bootstrap` as their fallback scope. This removes command-specific metadata from the CLI entrypoint without changing report values.
