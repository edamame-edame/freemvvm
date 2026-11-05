# C# Input event wrapper

- 対象unit: `UNIT-WRAP-CS-INPUT-001`
- 状態: 実装・focused test・Windows/Linux CI完了。commit `7da59da`をpush済み、同revision CI run `42028447142` success。Input/focus TODO batch全体の集約PRは後続
- 依存: `UNIT-ABI-022`、`UNIT-WRAP-CS-COMMAND-001`、`UNIT-WRAP-CS-CONTROLS-001`、`UNIT-WRAP-CS-WINDOW-001`
- 承認済み設計: `.docs/13_detail_design/wrappers/csharp.md` 1.6「Input event wrapper API」および `.docs/13_detail_design/runtime-objects/abi.md` 1.6「Input event subscription ABI」
- 試験設計: `.docs/32_unit_test/wrappers.md` 0.6 `UT-WRAP-CS-INPUT-001` / `UT-CONSUMER-CS-INPUT-001`

## 目的

承認済みInput ABIをC#の各`UiWidget`へ公開し、borrowed event viewのcallback寿命、delegate root、subscription cancel/release、delegate例外のABI境界を実装・検証する。

## 作業項目

1. [x] Core Input branchを親にした`impl-csharp/input-focus` branchを作成する。
2. [x] 依存する完了済みC# UI Control/Window wrapper branchを統合し、設計・test文書の競合を解消する。
3. [x] 文書競合を解消し、unit依存関係をControls/Window unitsまで明記する。
4. [x] C ABI入力イベントAPIと対応する型・enum・statusを確認し、C# `INativeApi`/PInvokeを追加する。
5. [x] callback-only `InputEventView`、Widget登録API、delegate root transfer、cancel/close寿命、例外処理を実装する。
6. [x] fake ABI focused testsで5種類のevent/payload、wrong-kind、登録失敗、borrow失効、self-cancel、callback実行中Dispose、例外handler経路を確認する。
7. [x] 別process C# consumerでpublic APIをcompileし、実ABI登録・cancel・closeを確認する。
8. [x] C# focused/consumer unit、unit manifest JSON parse、document link validationを実行し結果を記録する。
9. [x] 実装・試験を同じrevisionへpushし、日本語commit `7da59da`で`impl-csharp/input-focus`を更新。Windows/Linux CI run `42028447142`がsuccess。
10. [ ] 完了後、Input/focus関連TODO batch全体が揃った段階でPRを作成する。

## 検証記録

- `dotnet build wrappers/csharp/tests/RuntimeTests/RuntimeTests.csproj --no-restore`: 成功、0 warning / 0 error。
- `dotnet run --project wrappers/csharp/tests/RuntimeTests/RuntimeTests.csproj --no-build --no-restore`: 成功。5種類のevent、payload/wrong-kind、registration failure、callback後view失効、self-cancel、callback中Dispose、delegate例外を確認。
- `cargo build --locked -p freemvvm-c-abi`: 成功。
- `dotnet run --project wrappers/csharp/tests/Consumer/Consumer.csproj --configuration Debug --no-restore -- --input-only <absolute-native-path>`: 成功。実ABIで登録・cancel・close。
- `python -m py_compile tools/dev.py`, `python tools/check_docs.py`, `tools/units.yaml` JSON parse: 成功。
- Python `ruff`はこのworktree環境に存在せず未実行。CIでbootstrap後に実行する。
- C# Input fake ABI unitはRuntimeTests suite内で実行。CI run `42028447142`はWindows/Linux両方でsuccess。peer起動は環境依存のため、実C ABI consumerのCI経路からは除き、fake ABIでopen status転送を確認する。

## 対象外

- Core/C ABIの入力配送、Focus遷移、OS event発生の挙動変更。
- C++/Rust/Python wrapper実装。
- UI描画、実機入力、Platform adapterの合否判断。

## 完了条件

- 5種類のInput eventとpayload accessorをC# public APIから使える。
- borrowed `InputEventView`はcallback後の再利用を拒否し、textはmanaged `string`へcopyされる。
- 登録失敗ではmanaged rootを保持せず、登録成功後はnative destroyまでrootを維持する。
- cancelとsubscription handle closeを区別し、destroyを一度だけ実行する。
- delegate例外とfailure handler例外がC ABIを越えず、delegate例外時はeventをhandledにする。
- `UT-WRAP-CS-INPUT-001`と`UT-CONSUMER-CS-INPUT-001`が最終実装revisionで成功する。
