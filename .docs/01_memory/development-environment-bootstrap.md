# 開発環境 / Windows bootstrap・toolchain・4言語build

- 更新日: 2026-10-17
- スコープ: root Cargo/CMake/global.json/pyproject、crates、wrappers、tools、bootstrap workflow
- 手順: [DEVELOPMENT.md](../../DEVELOPMENT.md)
- 計画・結果: [環境整備](../20_todo/development-environment-bootstrap.md)
- 試験記録: [bootstrap検証](../32_unit_test/development-bootstrap.md)

利用者の実装開始/環境整備指示で、既存試験9文書の0.1全文を承認済みへ更新した。Rust Core→C ABIのcrateと、Coreを直接取り込まないRust sys/safe wrapper、C++ INTERFACE target、C#net10.0 project、Python src-layoutを追加。製品API/配布用header/native loaderは未実装で、version0.0.0の生成物は開発骨格である。

repositoryのRust pinは1.90.0、minimum1.85.0/edition2024。標準CPython3.13.12の `.venv` を作成し、CMake3.28.4/Ninja package1.11.1.4/pytest8.3.5/Ruff0.11.13とbuild依存をexact版で固定した。`.NET SDK10.0.401` をglobal.jsonで選択。default Python3.15/CMake4.3RCを利用しない。OS既定PATHやrustup defaultを変更していない。rustupはtoolchain導入時に自身を1.29.1へ自動更新した。

Windowsではvswhereの最新C++ BuildToolsを検出し、VsDevCmdを子processに限って実行する。cmdのset出力は `/u` + UTF-16LEで受け、キーをuppercaseへ統一。env一覧を表示しない。Windows Popenの実行ファイル探索はchild env PATHだけでは解決しないため、shutil.whichでvenv CMake/Ninja/Ruffを絶対pathへ解決する。既存developer shellのINCLUDE/LIBがあれば利用する。

実行compilerはBuildTools14.51.36231、SDK10.0.26100.0、x64。初期PATHのCommunity MSVC14.44ではない。check16gateとCTest環境probe2件、minimum Rust1.85のcompile checkが成功。製品testを0件passとして数えていない。結果とdirty HEAD/input hashは `.local/verification/` に保存しGit除外。bootstrap実装単位はUNIT-BUILD-001、台帳は `tools/units.yaml`（schema2/JSON互換YAML）。

active文書76本5805ローカルリンク、lint/format、manifest parse、git diff --checkと生成物除外を確認し、対象外文書/AGENTS329ファイルを保持した。`.venv`、target/out、Python build/egg-info、C# bin/objはGit対象外。root LICENSEは既承認Apache-2.0方針に従う標準本文。

GitHub ActionsはWindows2022/Ubuntu24.04の全bootstrap検証をpush/PRで実行する設定のみ作成。full SHA pin/read-only権限。リモートrunは未実施で、Ubuntu runnerをDebian12正式profile検証とはしない。ARM/cross/sysroot/実機と製品unit選択器は未実装。次はHandleRegistryの製品unit/testを同じ変更で登録・実装する。既存Command進捗/TextBox通知/Focus図の接続gapは該当単位前に別改訂で整合する。
