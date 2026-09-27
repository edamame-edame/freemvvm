# MVVM UIライブラリ Python lint/type check とMCP連携に関する上位判断案

- 状態: BD-15承認済み（2026-09-26）
- 対応: library compilerのPython向け静的解析、View言語との型整合、MCP server公開
- 根拠: 承認済みBD-03、BD-14、L1～L4、A3、要求仕様第4・12節。DD-00は未承認

## BD-15 承認された判断

**Python向けlint/type checkをlibrary compilerの開発ツール群へ組み込み、同じ解析基盤をMCP server経由で利用できるようにする。Python解析は`ctypes` wrapper APIの型stubとView言語／ViewModelの宣言情報を使って静的に検査する。MCP serverはLint成果物と診断snapshotをTemp等の一時領域へ展開・保持し、都度workspace全体を再生成せず変更ファイルと依存箇所だけを再解析する。Build/CIも同じ診断エンジンを実行できるようにし、MCP serverだけを唯一の実行経路にはしない。解析で対象Control・Propertyを特定できるPython event callbackでは、BD-03で他の静的解析可能言語に求めた同一Control Propertyアクセスの警告を出す案とする。動的に解決されるPythonコードでは完全な検出を保証せず、承認済みRuntime保護は常に維持する。**

## 提案する境界

| 要素 | 推奨する責務 |
|---|---|
| Library compiler / diagnostics core | Language analyzerから診断を集約し、rule ID、severity、message、source span、related locationを共通形式で返す。同じ入力と設定からCLI/CIとMCPで同じ結果を生成する。 |
| Python analyzer | `ctypes`を隠すPython wrapperの公開型stub、Python type annotation、ViewModel宣言を参照し、API引数・戻り値・callback・Binding接続の静的型整合を検査する。 |
| View analyzer | View言語のBinding path、Property型、event callback接続を解析し、Python側宣言と照合可能な型情報を共有する。View言語固有構文・規則はView analyzerに置く。 |
| Other language analyzers | C++／C#／Rustの既存toolchain診断を共通形式へ取り込める拡張点を用意する。各言語をlibrary compilerが独自に再実装することまでは要求しない。 |
| MCP server | Workspace/fileを指定したlint・type-check requestを受け、共通diagnosticsを返す。開発エージェントがMCP経由で同じcompiler解析を呼び出せるようにする。 |
| Build/CI entry | MCPと同じcompiler/checkerをCLIまたはbuild integrationから直接実行する。MCP daemonの起動状態をbuild再現性やCI成功条件にしない。 |

## Incremental解析と一時キャッシュ

- MCP serverはworkspace単位のLint snapshot、生成型情報、再利用可能な解析結果をOSのTemp directory等へ保持する。これらは再生成可能なcacheであり、repository内のsourceや正本のspecを置き換えない。
- 通常requestでは変更されたsourceと、その型・Binding依存先だけを再解析する。未変更nodeの結果は再利用し、診断も差分更新する。初回、cache破損、全体設定変更時は全解析を許す。
- cache keyには少なくともworkspace identity、source/dependency content hash、compiler・analyzer version、rule/config、生成されたwrapper type stubとView schemaのversion/hashを含める。いずれかが変われば該当範囲を無効化し、古い結果を成功扱いで返さない。
- 解析中にsourceが更新された場合、旧generationの結果が新sourceに対応するように見せない。diagnostic snapshotをgeneration/hash付きで扱い、更新はatomicに公開する。具体formatとfile watcher方式は詳細設計へ送る。
- 一時cacheはworkspaceごとに分離し、server/session終了、TTL、容量上限等で削除できるようにする。削除後は再解析で復旧できる。

## 静的解析の範囲とBD-03との整合

- Python API利用とViewModel接続では、注釈または生成型情報がある範囲を強く型検査する。未注釈の値やdynamic dispatchに依存する式を、型安全とみなして黙って通さない。`unknown`等のdiagnosticまたは明示的なdynamic境界として扱う。
- event callback内の同一Control Propertyアクセスは、型情報とControl identityを解析で結び付けられる場合に警告する。読み書きを分け、同一Property書き戻しは強い警告とする。動的にしか決まらないaliasやreflectionまで捕捉すると約束しない。
- これはBD-03の「Pythonにはcompile-time warningを要求しない」という承認済み例外を、**静的に証明できるPython箇所に限ってlint警告を追加する提案**である。承認前はBD-03の本文・Runtime契約を変更しない。
- Runtimeの有限通知処理、再入・非収束連鎖の保護、diagnosticsはPythonでも既承認どおり残す。Static lintはRuntime safetyの代替ではない。
- analyzerが解決できなかったdynamic codeは、別の検査またはRuntime診断が必要になり得る。解析不能を無警告の安全判定として扱わない方針を推奨する。

## MCPを採用する理由と位置づけ

- MCPを診断エンジン本体ではなく、library compilerの解析機能を開発toolへ公開するprotocol adapterとして置く。これにより、CI/buildとエージェント利用の結果が分岐しない。
- MCPは通常のbuild compilerやIDE内蔵type checkerを置き換えない。build/reproducibilityはcompiler CLI、agent向け対話的解析はMCP、IDE統合は必要に応じて別adapterから同じ診断基盤を利用できる。
- 一時cacheで再解析を局所化し、解析全体の再実行を減らす。再利用の正しさはworkspace/source/config/checker dependencyのfingerprintで担保し、遅延した古いdiagnosticを最新と誤認させない。
- Python linterのanalyzer製品選択、MCP tool/schema、file access/daemon lifecycle、診断fix提案は詳細設計へ送る。

## この判断では決めないこと

- mypy、Pyright等の既製checker採用、独自Python AST/type analyzerとの分担、version/pinning。
- Python type strictness設定、未注釈コードへの適用段階、既存project向けsuppression形式。
- C++／C#／Rust各analyzerの具体製品・command、View compilerの実装方式。
- Python wrapperの`.pyi`生成方法、ViewModel schema共有形式、cross-language symbol resolver。
- MCP tool名・request/response schema、IDE/LSP連携、auto-fix、cache、増分解析の詳細。
- Temp rootの選定、cache directory naming、TTL/容量既定値、file watcherとhashの具体実装。
- BD-03全体のRuntime上限・diagnostics変更。静的警告範囲の拡張以外は本判断の対象外。

## 承認記録

2026-09-26に利用者承認。library compilerのPython lint/type checkをMCP serverから利用できるようにし、Build/CIとは同じdiagnostics engineを使う。Lint成果物・診断snapshot等はTemp directoryへ保持し、workspace/source/config/analyzer fingerprintに基づいて変更ファイルと依存箇所のみ増分再解析する。古い結果を最新と誤認させず、cacheは再生成可能とする。Pythonのevent callback内で同一Control Propertyへアクセスする警告は、type/control identityを静的に解決できる場合に出す。これによりBD-03のPython例外をその解析可能範囲で縮小するが、dynamic codeの全件検出は保証しない。Runtime保護・diagnosticsは従来どおりPythonにも適用する。checker製品、MCP schema、cache TTL等は後続設計とする。DD-00の承認やDD-01以降へ進む判断ではない。
