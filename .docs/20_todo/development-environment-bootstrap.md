# 実装開始 / 開発環境とビルド基盤

- 作成日: 2026-10-17
- 状態: Windows x64の開発基盤作成・検証完了。CIリモート/Linux/ARMは未検証
- 指示: 「OK そろそろスクラッチを開始しましょうか。環境整備から始めてください。」

## 範囲と進め方

既存テスト9文書の0.1全文承認を記録し、環境整備を開始する。製品機能を先行実装せず、Rust CoreとC ABIのcrate、Coreを直接取り込まない各wrapperのソース配置・言語manifest、共通開発コマンドを用意する。試験設計と実装・実行の状態は別管理とする。既存の設計接続gapは環境構築で確定しない。

要求minimumはRust1.85/edition2024、C++17/CMake3.28、C#14/net10.0、CPython3.13。開発用RustはDD候補の1.90.0へ固定し、minimum1.85.0も別gateで確認する。.NETは既存SDK10.0.401を選択する。Python3.13のrepository内venvへ開発依存を固定し、CMakeの既存RC版やdefault Python3.15に依存しない。OS全体の既定toolchain/PATHは変更しない。

Windows x64でcompile/link/buildを実行し、Linux nativeとminimum buildはCI構成を用意する。ARMのsysroot/SDK・代表board・実OS機能試験は別phase。CIはpush/PRの同一checkoutで動き、bootstrap時点は全gateを実行する。変更選択・unit台帳の高度化と製品unit-testは機能実装時に進める。現段階のprobeを70件の製品単体試験合格に数えない。

## to-do

- [x] AGENTS・言語規則・Git規則・RQ/ADR/BD/DD・既存test契約と実環境を確認する。
- [x] test9文書の承認と、環境整備の実装単位・検証項目を記録する。
- [x] toolchain/venvと依存のexact版を用意する。
- [x] Rust workspace、C/C++ build probe、C#/Python/Rust wrapper配置を作成する。
- [x] 開発コマンド・設定・ignore・導入手順とbootstrap用CIを作成する。
- [x] Windows上で全bootstrap gateとRust minimumを実行し、実測環境と結果を記録する。
- [x] 既存変更の保持・生成物除外・文書リンクを確認し、作業メモを保存する。

## 完了条件

クリーンcheckoutから必要toolchainを導入する手順と、format/check/buildを再実行する入口がある。Core→C ABI→thin wrapperの依存方向を保ち、公開APIの空実装を成功として配布しない。local結果と未実行CI/Linux/ARMを区別する。commit/push・公開・実機support宣言は含めない。

## 実施結果

[開発手順](../../DEVELOPMENT.md)、[実装unit台帳](../../tools/units.yaml)、[bootstrap検証](../32_unit_test/development-bootstrap.md)を作成。既存test9文書は0.1全文承認済みへ更新した。Rust4crate、C/C++環境probe、C#net10.0 project、Python src-layout/wheel build、各language manifestとlockを用意した。製品公開APIやnative loaderは未実装。

Rust1.90.0/rustfmt/clippyとminimum1.85.0を導入し、既存標準CPython3.13.12のvenvへ固定CMake3.28.4/Ninja/pytest/Ruff/build依存を導入した。既存.NET10.0.401を選択。MSVC環境はvswhereから検出したBuildTools14.51.36231、Windows SDK10.0.26100.0、x64を使用した。初期PATHにあったCommunity版とは異なるため実行時の採取値を証拠とする。release用profile/compiler lockの確定とは区別する。

Windowsの `tools/dev.py check` は16gate成功、C/C++のCTest2probe成功、`minimum` はRust1.85.0の全workspace compile check成功。`.local/verification/check.json` / `minimum.json` / `preservation.json`へdirty状態・HEAD・入力hash・実toolchain・commandと結果を保存した。文書76本の5805ローカルリンク検査、lint/format、manifest parse、生成物除外、git diff --check成功。既存の対象外文書とAGENTS計329ファイルのhash一致。

CIはWindows2022/Ubuntu24.04で全bootstrap gateを実行する設定を作成し、Actions4本をfull SHAへpinした。リモート実行は未実施。製品unitのpath mapping/依存逆閉包選択、製品単体試験70項目、release9/version6、Linux正式profile/ARMcross/board実行は後続。既存3接続gapも未解消を維持する。次の実装単位はhandle registryを入口とし、製品unit台帳と対応testを同じ変更で作成する。

2026-10-18、VS Code同梱rust-analyzerと固定Rust 1.90.0の互換警告を受け、対応する`rust-analyzer` componentを`rust-toolchain.toml`と導入手順・bootstrap CIへ追加した。固定toolchain/compiler版とrustup defaultは変更していない。詳細は[Rust Analyzer component対応](rust-analyzer-pinned-component.md)を参照。
