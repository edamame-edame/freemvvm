# 要求ID移行対応表

- 文書版: 0.2
- 状態: 再編に伴う移行記録・レビュー待ち
- 更新日: 2026-10-12
- 入口: [要求仕様の総目次](reqirements-specification.md)

暫定REQ IDは旧要求test対応表の識別子であり、旧要求本文には存在しなかった。正式IDの定義先は各RQ章とする。本表は旧文書の箇条書き・確認事項を遡るための記録であり、旧IDを新規要求やtestに使用しない。旧196項目を現行の要求へ対応付け、共通モデル先行要求の二重表現を統合した。旧縦断シナリオのREQ-5項目は既存要求へ統合し、独立要求章は設けない。

| 旧要求test ID | 旧項目 | 正式RQ ID・定義先 | 移行内容 |
|---|---|---|---|
| REQ-3-01 | 共通MVVM UIをC++/C#/Rust/Python 3で利用 | [RQ-01-001](RQ-01_purpose-architecture.md#rq-01-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-3-02 | Rust Core＋versioned C ABI＋thin wrapper | [RQ-01-002](RQ-01_purpose-architecture.md#rq-01-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-01 | Object | [RQ-02-001](RQ-02_core-features.md#rq-02-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-02 | Property | [RQ-02-002](RQ-02_core-features.md#rq-02-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-03 | Binding | [RQ-02-003](RQ-02_core-features.md#rq-02-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-04 | Command | [RQ-02-004](RQ-02_core-features.md#rq-02-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-05 | Collection | [RQ-02-005](RQ-02_core-features.md#rq-02-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-06 | Selection | [RQ-02-006](RQ-02_core-features.md#rq-02-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-07 | Visual Tree | [RQ-02-007](RQ-02_core-features.md#rq-02-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-08 | Layout | [RQ-02-008](RQ-02_core-features.md#rq-02-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-09 | Input | [RQ-02-009](RQ-02_core-features.md#rq-02-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-10 | Focus | [RQ-02-010](RQ-02_core-features.md#rq-02-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-11 | IME | [RQ-02-011](RQ-02_core-features.md#rq-02-011) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-12 | Error handling | [RQ-02-012](RQ-02_core-features.md#rq-02-012) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-13 | Lifetime management | [RQ-02-013](RQ-02_core-features.md#rq-02-013) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.1-14 | UI thread management | [RQ-02-014](RQ-02_core-features.md#rq-02-014) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.2-01 | C++ | [RQ-03-001](RQ-03_language-support.md#rq-03-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.2-02 | C# | [RQ-03-002](RQ-03_language-support.md#rq-03-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.2-03 | Rust | [RQ-03-003](RQ-03_language-support.md#rq-03-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.2-04 | Python 3 | [RQ-03-004](RQ-03_language-support.md#rq-03-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P01 | fixed-width integers | [RQ-04-001](RQ-04_abi-boundary.md#rq-04-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P02 | float/double | [RQ-04-002](RQ-04_abi-boundary.md#rq-04-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P03 | UTF-8 pointer+length | [RQ-04-003](RQ-04_abi-boundary.md#rq-04-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P04 | opaque handle | [RQ-04-004](RQ-04_abi-boundary.md#rq-04-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P05 | explicit retain/release | [RQ-04-005](RQ-04_abi-boundary.md#rq-04-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P06 | ui_status | [RQ-04-006](RQ-04_abi-boundary.md#rq-04-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P07 | error object | [RQ-04-007](RQ-04_abi-boundary.md#rq-04-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P08 | callback+userdata+destroy | [RQ-04-008](RQ-04_abi-boundary.md#rq-04-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P09 | ABI version | [RQ-04-009](RQ-04_abi-boundary.md#rq-04-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-P10 | feature query | [RQ-04-010](RQ-04_abi-boundary.md#rq-04-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-N01 | no Rust types/trait/generic/borrow in ABI | [RQ-04-011](RQ-04_abi-boundary.md#rq-04-011) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-N02 | no C++ class/STL/RTTI/exception in ABI | [RQ-04-012](RQ-04_abi-boundary.md#rq-04-012) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-N03 | no Python object in ABI | [RQ-04-013](RQ-04_abi-boundary.md#rq-04-013) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-N04 | no C# object/delegate in ABI | [RQ-04-014](RQ-04_abi-boundary.md#rq-04-014) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-N05 | no Rust panic in ABI | [RQ-04-015](RQ-04_abi-boundary.md#rq-04-015) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-N06 | no C++ exception in ABI | [RQ-04-016](RQ-04_abi-boundary.md#rq-04-016) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.3-N07 | no Python exception in ABI | [RQ-04-017](RQ-04_abi-boundary.md#rq-04-017) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-01 | handle owner specified | [RQ-05-001](RQ-05_ownership-callbacks.md#rq-05-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-02 | retain/release specified | [RQ-05-002](RQ-05_ownership-callbacks.md#rq-05-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-03 | destruction order specified | [RQ-05-003](RQ-05_ownership-callbacks.md#rq-05-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-04 | string/array ownership specified | [RQ-05-004](RQ-05_ownership-callbacks.md#rq-05-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-05 | pointer borrow duration specified | [RQ-05-005](RQ-05_ownership-callbacks.md#rq-05-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-06 | callback retain/unsubscribe specified | [RQ-05-006](RQ-05_ownership-callbacks.md#rq-05-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-07 | no callback after unsubscribe | [RQ-05-007](RQ-05_ownership-callbacks.md#rq-05-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-08 | userdata destroy responsibility | [RQ-05-008](RQ-05_ownership-callbacks.md#rq-05-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.4-09 | callback re-entrancy declared | [RQ-05-009](RQ-05_ownership-callbacks.md#rq-05-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.5-01 | UI operations limited to UI thread | [RQ-06-001](RQ-06_threading-dispatch.md#rq-06-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.5-02 | worker UI work uses dispatcher | [RQ-06-002](RQ-06_threading-dispatch.md#rq-06-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.5-03 | cross-thread data races prevented | [RQ-06-003](RQ-06_threading-dispatch.md#rq-06-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.5-04 | async Command completion/error/cancel defined | [RQ-06-004](RQ-06_threading-dispatch.md#rq-06-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-01 | OneTime | [RQ-07-001](RQ-07_binding.md#rq-07-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-02 | OneWay | [RQ-07-002](RQ-07_binding.md#rq-07-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-03 | TwoWay | [RQ-07-003](RQ-07_binding.md#rq-07-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-04 | update direction | [RQ-07-004](RQ-07_binding.md#rq-07-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-05 | update timing | [RQ-07-005](RQ-07_binding.md#rq-07-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-06 | update order | [RQ-07-006](RQ-07_binding.md#rq-07-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-07 | binding cycle | [RQ-07-007](RQ-07_binding.md#rq-07-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-08 | re-entrancy | [RQ-07-008](RQ-07_binding.md#rq-07-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-09 | transaction policy | [RQ-07-009](RQ-07_binding.md#rq-07-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-10 | validation error propagation | [RQ-07-010](RQ-07_binding.md#rq-07-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-4.6-11 | unbind behavior | [RQ-07-011](RQ-07_binding.md#rq-07-011) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-5-01 | TextBox input reaches ViewModel | [RQ-07-003](RQ-07_binding.md#rq-07-003) | TwoWayの意味へ統合。旧再編集ID RQ-08-001は重複により廃止 |
| REQ-5-02 | ViewModel change reaches Text | [RQ-07-002](RQ-07_binding.md#rq-07-002) | OneWayの意味へ統合。旧再編集ID RQ-08-002は重複により廃止 |
| REQ-5-03 | Button executes Command | [RQ-02-004](RQ-02_core-features.md#rq-02-004) | Command要求へ統合。旧再編集ID RQ-08-003は重複により廃止 |
| REQ-5-04 | Command availability | [RQ-02-004](RQ-02_core-features.md#rq-02-004) | Commandの実行可否へ統合。旧再編集ID RQ-08-004は重複により廃止 |
| REQ-5-05 | focus moves | [RQ-09-009](RQ-09_ui-scope.md#rq-09-009) | 初期基盤のFocus要求へ統合。旧再編集ID RQ-08-005は重複により廃止 |
| REQ-5-06 | keyboard operation | [RQ-09-010](RQ-09_ui-scope.md#rq-09-010) | 初期基盤のkeyboard input要求へ統合。旧再編集ID RQ-08-006は重複により廃止 |
| REQ-5-07 | accessibility name | [RQ-10-003](RQ-10_accessibility.md#rq-10-003) | Accessibility name要求へ統合。旧再編集ID RQ-08-007は重複により廃止 |
| REQ-5-08 | same test in four languages | [RQ-01-001](RQ-01_purpose-architecture.md#rq-01-001) | 4言語共通利用要求へ統合。旧再編集ID RQ-08-008は重複により廃止 |
| REQ-6.1-01 | Window | [RQ-09-001](RQ-09_ui-scope.md#rq-09-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-02 | Widget | [RQ-09-002](RQ-09_ui-scope.md#rq-09-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-03 | Container | [RQ-09-003](RQ-09_ui-scope.md#rq-09-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-04 | Text | [RQ-09-004](RQ-09_ui-scope.md#rq-09-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-05 | TextBox | [RQ-09-005](RQ-09_ui-scope.md#rq-09-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-06 | Button | [RQ-09-006](RQ-09_ui-scope.md#rq-09-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-07 | StackPanel | [RQ-09-007](RQ-09_ui-scope.md#rq-09-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-08 | basic layout | [RQ-09-008](RQ-09_ui-scope.md#rq-09-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-09 | focus | [RQ-09-009](RQ-09_ui-scope.md#rq-09-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-10 | keyboard input | [RQ-09-010](RQ-09_ui-scope.md#rq-09-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.1-11 | minimal accessibility name | [RQ-09-011](RQ-09_ui-scope.md#rq-09-011) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-01 | CheckBox | [RQ-09-012](RQ-09_ui-scope.md#rq-09-012) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-02 | ToggleButton | [RQ-09-013](RQ-09_ui-scope.md#rq-09-013) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-03 | ScrollViewer | [RQ-09-014](RQ-09_ui-scope.md#rq-09-014) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-04 | Image | [RQ-09-015](RQ-09_ui-scope.md#rq-09-015) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-05 | Border | [RQ-09-016](RQ-09_ui-scope.md#rq-09-016) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-06 | Separator | [RQ-09-017](RQ-09_ui-scope.md#rq-09-017) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-07 | ProgressBar | [RQ-09-018](RQ-09_ui-scope.md#rq-09-018) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-08 | CollectionModel | [RQ-09-019](RQ-09_ui-scope.md#rq-09-019) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-09 | SelectionModel | [RQ-09-020](RQ-09_ui-scope.md#rq-09-020) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-10 | ListView | [RQ-09-021](RQ-09_ui-scope.md#rq-09-021) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-11 | virtualized list | [RQ-09-022](RQ-09_ui-scope.md#rq-09-022) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-12 | Style | [RQ-09-023](RQ-09_ui-scope.md#rq-09-023) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-13 | Theme | [RQ-09-024](RQ-09_ui-scope.md#rq-09-024) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-14 | Resource | [RQ-09-025](RQ-09_ui-scope.md#rq-09-025) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-15 | Template | [RQ-09-026](RQ-09_ui-scope.md#rq-09-026) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.2-16 | shared Collection/model contract precedes list Controls | [RQ-09-037](RQ-09_ui-scope.md#rq-09-037) | 同一要求へ統合。旧§6.3末尾の共通モデル先行条件 |
| REQ-6.3-01 | ComboBox | [RQ-09-027](RQ-09_ui-scope.md#rq-09-027) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-02 | TreeView | [RQ-09-028](RQ-09_ui-scope.md#rq-09-028) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-03 | TableView | [RQ-09-029](RQ-09_ui-scope.md#rq-09-029) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-04 | Dialog | [RQ-09-030](RQ-09_ui-scope.md#rq-09-030) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-05 | Popup | [RQ-09-031](RQ-09_ui-scope.md#rq-09-031) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-06 | Menu | [RQ-09-032](RQ-09_ui-scope.md#rq-09-032) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-07 | ToolBar | [RQ-09-033](RQ-09_ui-scope.md#rq-09-033) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-08 | TabView | [RQ-09-034](RQ-09_ui-scope.md#rq-09-034) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-09 | binding diagnostics | [RQ-09-035](RQ-09_ui-scope.md#rq-09-035) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-10 | UI test support | [RQ-09-036](RQ-09_ui-scope.md#rq-09-036) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-6.3-11 | shared model/selection/template/diff/virtualization before individual controls | [RQ-09-037](RQ-09_ui-scope.md#rq-09-037) | 同一要求へ統合。旧§6.3末尾の共通モデル先行条件 |
| REQ-7-01 | Accessibility is foundational | [RQ-10-001](RQ-10_accessibility.md#rq-10-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-02 | role | [RQ-10-002](RQ-10_accessibility.md#rq-10-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-03 | name | [RQ-10-003](RQ-10_accessibility.md#rq-10-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-04 | state | [RQ-10-004](RQ-10_accessibility.md#rq-10-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-05 | value | [RQ-10-005](RQ-10_accessibility.md#rq-10-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-06 | focus state | [RQ-10-006](RQ-10_accessibility.md#rq-10-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-07 | keyboard operation | [RQ-10-007](RQ-10_accessibility.md#rq-10-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-08 | enabled/disabled | [RQ-10-008](RQ-10_accessibility.md#rq-10-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-09 | platform accessibility API mapping | [RQ-10-009](RQ-10_accessibility.md#rq-10-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-7-10 | ARIA APG as reference only; desktop/embedded adaptation is project-defined | [RQ-10-010](RQ-10_accessibility.md#rq-10-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.1-01 | Linux ARMv7 | [RQ-11-001](RQ-11_platform.md#rq-11-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.1-02 | Linux AArch64 | [RQ-11-002](RQ-11_platform.md#rq-11-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.1-03 | Linux x64 | [RQ-11-003](RQ-11_platform.md#rq-11-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.1-04 | Windows x64 | [RQ-11-004](RQ-11_platform.md#rq-11-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.1-05 | Windows ARM64 | [RQ-11-005](RQ-11_platform.md#rq-11-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.1-06 | macOS x64/ARM64 | [RQ-11-006](RQ-11_platform.md#rq-11-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.1-07 | bare metal/MCU separate profile | [RQ-11-007](RQ-11_platform.md#rq-11-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-01 | separate renderer | [RQ-11-008](RQ-11_platform.md#rq-11-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-02 | separate platform backend | [RQ-11-009](RQ-11_platform.md#rq-11-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-03 | separate text engine | [RQ-11-010](RQ-11_platform.md#rq-11-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-04 | separate image codec | [RQ-11-011](RQ-11_platform.md#rq-11-011) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-05 | separate input backend | [RQ-11-012](RQ-11_platform.md#rq-11-012) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-06 | separate accessibility backend | [RQ-11-013](RQ-11_platform.md#rq-11-013) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-07 | embedded backend order DRM/KMS+EGL/GLES | [RQ-11-014](RQ-11_platform.md#rq-11-014) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-08 | embedded backend order Wayland | [RQ-11-015](RQ-11_platform.md#rq-11-015) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-09 | embedded backend order X11 | [RQ-11-016](RQ-11_platform.md#rq-11-016) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-10 | embedded backend order framebuffer | [RQ-11-017](RQ-11_platform.md#rq-11-017) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-11 | embedded backend order software renderer | [RQ-11-018](RQ-11_platform.md#rq-11-018) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-8.2-12 | isolate GPU/input/IME/rotation/power/window dependencies in adapter | [RQ-11-019](RQ-11_platform.md#rq-11-019) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-01 | artifact per target triple | [RQ-12-001](RQ-12_build-distribution.md#rq-12-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-02 | per OS/architecture library | [RQ-12-002](RQ-12_build-distribution.md#rq-12-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-03 | same target C ABI artifact shared by four languages | [RQ-12-003](RQ-12_build-distribution.md#rq-12-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-04 | C++ CMake install/export | [RQ-12-004](RQ-12_build-distribution.md#rq-12-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-05 | Rust `-sys`/safe crate, separate runtime artifact, explicit prebuilt import | [RQ-12-005](RQ-12_build-distribution.md#rq-12-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-06 | C# RID native asset | [RQ-12-006](RQ-12_build-distribution.md#rq-12-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-07 | Python `ctypes`, compiled extension not required | [RQ-12-007](RQ-12_build-distribution.md#rq-12-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-08 | Python wrapper/native runtime distribution separation and initial wheel targets | [RQ-12-008](RQ-12_build-distribution.md#rq-12-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-09 | Yocto SDK/sysroot/linker/C library fixed in CI | [RQ-12-009](RQ-12_build-distribution.md#rq-12-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-9-10 | Windows→Linux x64→ARM cross-build/hardware gate order | [RQ-12-010](RQ-12_build-distribution.md#rq-12-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-01 | startup time | [RQ-13-001](RQ-13_nonfunctional-measurement.md#rq-13-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-02 | binary size | [RQ-13-002](RQ-13_nonfunctional-measurement.md#rq-13-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-03 | memory use | [RQ-13-003](RQ-13_nonfunctional-measurement.md#rq-13-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-04 | render FPS | [RQ-13-004](RQ-13_nonfunctional-measurement.md#rq-13-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-05 | layout time | [RQ-13-005](RQ-13_nonfunctional-measurement.md#rq-13-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-06 | Property update cost | [RQ-13-006](RQ-13_nonfunctional-measurement.md#rq-13-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-07 | Collection diff cost | [RQ-13-007](RQ-13_nonfunctional-measurement.md#rq-13-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-08 | callback/FFI crossing cost | [RQ-13-008](RQ-13_nonfunctional-measurement.md#rq-13-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-09 | maximum UI-thread work time | [RQ-13-009](RQ-13_nonfunctional-measurement.md#rq-13-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-10 | device stability | [RQ-13-010](RQ-13_nonfunctional-measurement.md#rq-13-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-10-11 | same feature/profile and separate rendering/layout/text/image/GPU sync/FFI profiling | [RQ-13-011](RQ-13_nonfunctional-measurement.md#rq-13-011) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-01 | C ABI specification before implementation | [RQ-14-001](RQ-14_delivery-sequence.md#rq-14-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-02 | ABI version/error/ownership/thread rules before implementation | [RQ-14-002](RQ-14_delivery-sequence.md#rq-14-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-03 | implement Rust Object/Property/Binding/Command | [RQ-14-003](RQ-14_delivery-sequence.md#rq-14-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-04 | create four wrappers | [RQ-14-004](RQ-14_delivery-sequence.md#rq-14-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-05 | run MVVM smoke in four languages | [RQ-14-005](RQ-14_delivery-sequence.md#rq-14-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-06 | add Window/StackPanel/TextBox/Text/Button | [RQ-14-006](RQ-14_delivery-sequence.md#rq-14-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-07 | add focus/keyboard/IME basics | [RQ-14-007](RQ-14_delivery-sequence.md#rq-14-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-08 | add accessibility name/role | [RQ-14-008](RQ-14_delivery-sequence.md#rq-14-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-09 | add Collection/Selection/Template | [RQ-14-009](RQ-14_delivery-sequence.md#rq-14-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-10 | implement renderer/platform adapter | [RQ-14-010](RQ-14_delivery-sequence.md#rq-14-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-11 | add ABI compatibility test to CI | [RQ-14-011](RQ-14_delivery-sequence.md#rq-14-011) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-12 | measure real-device performance | [RQ-14-012](RQ-14_delivery-sequence.md#rq-14-012) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-11-13 | add batch API/optimization only where needed | [RQ-14-013](RQ-14_delivery-sequence.md#rq-14-013) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-01 | OS priority and target matrix | [RQ-15-001](RQ-15_requirement-decisions.md#rq-15-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-02 | native vs custom drawing | [RQ-15-002](RQ-15_requirement-decisions.md#rq-15-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-03 | Renderer API/model | [RQ-15-003](RQ-15_requirement-decisions.md#rq-15-003) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-04 | initial control abstraction | [RQ-15-004](RQ-15_requirement-decisions.md#rq-15-004) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-05 | text shaping/Unicode/RTL | [RQ-15-005](RQ-15_requirement-decisions.md#rq-15-005) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-06 | IME composition | [RQ-15-006](RQ-15_requirement-decisions.md#rq-15-006) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-07 | binding cycles | [RQ-15-007](RQ-15_requirement-decisions.md#rq-15-007) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-08 | validation model | [RQ-15-008](RQ-15_requirement-decisions.md#rq-15-008) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-09 | async Command cancel/progress | [RQ-15-009](RQ-15_requirement-decisions.md#rq-15-009) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-10 | UI transaction | [RQ-15-010](RQ-15_requirement-decisions.md#rq-15-010) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-11 | error object structure | [RQ-15-011](RQ-15_requirement-decisions.md#rq-15-011) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-12 | ABI compatibility policy | [RQ-15-012](RQ-15_requirement-decisions.md#rq-15-012) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-13 | Python wrapper/tooling | [RQ-15-013](RQ-15_requirement-decisions.md#rq-15-013) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-14 | license | [RQ-15-014](RQ-15_requirement-decisions.md#rq-15-014) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-15 | support period | [RQ-15-015](RQ-15_requirement-decisions.md#rq-15-015) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-16 | distribution form | [RQ-15-016](RQ-15_requirement-decisions.md#rq-15-016) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-17 | CI compiler/target matrix | [RQ-15-017](RQ-15_requirement-decisions.md#rq-15-017) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-18 | accessibility scope | [RQ-15-018](RQ-15_requirement-decisions.md#rq-15-018) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-P1 | ownership/callback/thread priority | [RQ-15-019](RQ-15_requirement-decisions.md#rq-15-019) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-P2 | Binding update semantics priority | [RQ-15-020](RQ-15_requirement-decisions.md#rq-15-020) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-P3 | OS/embedded rendering priority | [RQ-15-021](RQ-15_requirement-decisions.md#rq-15-021) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-12-P4 | Text/IME/Accessibility priority | [RQ-15-022](RQ-15_requirement-decisions.md#rq-15-022) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-13-01 | ADR records architecture decisions | [RQ-16-001](RQ-16_document-governance.md#rq-16-001) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-13-02 | research records external basis/options/open issues | [RQ-16-002](RQ-16_document-governance.md#rq-16-002) | 要求内容を継承。要求test状態・下流未決は要求承認とは別管理 |
| REQ-13-03 | accepted requirements change reviews compatibility | [RQ-16-003](RQ-16_document-governance.md#rq-16-003) | 今回指示により要求承認と下流承認の関係を更新 |
| REQ-13-04 | ADR wins conflict with requirements; update ADR as needed | [RQ-16-004](RQ-16_document-governance.md#rq-16-004) | 今回指示により要求承認と下流承認の関係を更新 |

## 承認履歴

| 日付 | 対象版・ID / 範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-09-26 | 旧要求仕様の該当範囲 | 利用者 | Acceptedを継承。後日追記の承認日・未決範囲は旧版で確認 |
| 2026-10-12 | 要求承認管理・章分割・RQ ID化 | 利用者 | 今回の指示による管理方針。旧本文に記載済みの承認事項を要求側で管理する |
| — | 移行ID・同一要求の統合・参照リンク / 再編集版0.2 | — | 継承要求の再編集版はレビュー待ち |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 旧版 | 2026-09-25 | 旧要求仕様を作成。2026-09-26にAccepted | 旧文書参照 | 旧要求仕様の承認記録 |
| 0.1 | 2026-10-12 | 章分割、RQ ID、受入確認案・設計/testリンク、要求承認履歴の集約 | Codex | 管理方針の指示に基づく再編集。版全体のレビューは未完了 |
| 0.2 | 2026-10-12 | 重複するRQ-08再編集IDを廃止し、旧REQ-5項目を既存要求へ統合 | Codex | 要求移行記録を更新。レビュー待ち |
