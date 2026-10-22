# 開発基盤の検証

- 文書版: 0.1
- 状態: 環境整備指示に基づく実施記録。製品unit-testと区別
- 作成日: 2026-10-17
- 実装単位: UNIT-BUILD-001（開発コマンドとbootstrap probe）
- 根拠: [RQ-03-014](../11_requirements/RQ-03_language-support.md#rq-03-014)、[RQ-12-009](../11_requirements/RQ-12_build-distribution.md#rq-12-009)、[RQ-11-021](../11_requirements/RQ-11_platform.md#rq-11-021)、[build DD](../13_detail_design/build/toolchain-packages.md) 1.1、[実装unit契約](implementation-test-inventory.md)

## 検証単位と入力

対象sourceは `tools/dev.py`、`tools/check_docs.py`、`tools/probes/`、設定はCargo/CMake/.NET/Python manifestとtoolchain/依存pin。crateとwrapperの空の骨格をcompileする。製品のUNIT-CORE-001やUT-HANDLE-001を実装済みへ昇格しない。

前提は[開発手順](../../DEVELOPMENT.md)に従う標準CPython3.13のvenv、Rustの二版、C/C++ compiler、固定.NET SDK。代表入力は同一worktreeのsource/configとtoolchainである。各commandは固定版不足・compiler不在・不正リンク・compile/link失敗を非zeroで返し、reportにも失敗理由を残す。

| 検証ID | 操作・期待結果 | 実行入口 |
| --- | --- | --- |
| UT-BUILD-ENV-001 | 固定toolとvenvを照合し、全言語の骨格をcompile/build。C/C++ probeで固定幅値・C++17・float特性とatomic利用を確認し、C++ Runtime unitとrelocated package consumerを実行 | venv Pythonで `tools/dev.py check` |
| UT-BUILD-MIN-001 | Rust1.85/edition2024で全workspaceをcompile check。minimum未確認を開発版成功で代替しない | venv Pythonで `tools/dev.py minimum` |

後始末はcompiler/installerの子process終了を確認し、生成物をvenv/target/out/obj/bin/egg-infoへ分離する。checkで開発wheelをrepositoryのvenvへ再導入するが、OS既定環境へinstallしない。自動再帰削除は行わず、再実行時はbuild toolの増分更新を使う。

## 証跡と限界

Windows x64で両入口を実行する。結果の正本は `.local/verification/check.json` と `minimum.json`。SHAとdirty状態、全入力hash、tool版と各gate終了codeを保持し、未commit変更をHEADに含まれる実装として報告しない。GitHub上は同じcheckout source/testを実行するworkflowを作成したが、リモートrunは未実施。

この単位はビルド環境の確認であり、要求test92TCや製品単体試験70項目の合格を意味しない。Linux/ARM/実GPU/IME/provider/native loader・署名済みartifactは未検証。正式supportのprofile lockは別に作成する。

## 承認履歴

| 日付 | 対象 | 承認者 | 結論・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 開発環境整備の実施 | 利用者 | 「環境整備から始めてください」に基づき実施。製品APIやrelease合格の承認を含めない |

## 改訂履歴

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | bootstrap source/config、検証入口と証拠範囲を記録 | Codex |
