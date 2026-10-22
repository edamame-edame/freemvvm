# 詳細設計フェーズ計画（`.docs/13_detail_design/`）

- 状態: 運用中。詳細設計の個別判断は各フェーズで確認する
- 作成日: 2026-09-26
- 対象: Rust Core、versioned C ABI、C++ / C# / Rust / Python 3 wrapper、初期縦断シナリオ
- この計画の目的: 小さな設計範囲ごとにレビューし、承認された判断だけを詳細設計として積み上げる

## 2026-10-17 具体化分冊の適用状況

[詳細設計索引](../13_detail_design/detailed-design-specification.md)のDD-24～40、[SD索引](../14_secure_design/00-security-design-map.md)のSD-06/07、[実装unit台帳](../32_unit_test/implementation-test-inventory.md)を承認・適用した。全200件の対応は[適用結果](remaining-design-application-result.md)へ記録した。以下の以前の作業・承認記録は各記録日時点であり、未決方式の現状はこの分冊を参照する。

## 進め方

1. 各フェーズの着手時に、ADR、要件、該当する `.docs/12_design/` の本文と判断表を再確認する。文書の見出しや状態欄だけで全文の承認を推定しない。
2. 一度に一つの小さな設計範囲を扱う。`.docs/13_detail_design/` にレビュー案を作り、前提、責務、状態遷移、所有権、失敗時、スレッド、依存関係、検証観点のうち該当するものを記す。
3. 新しい設計判断には詳細設計固有のID（例: `DD-HND-01`）を付ける。上位文書の承認済みIDを出典として併記し、未承認の判断と混ぜない。承認済み判断を変更する必要があれば、詳細設計だけで上書きせず上位文書のレビューへ戻す。
4. 各レビュー案について利用者の確認を受け、判断ごとの承認・修正・保留を**同じ文書**へ反映する。承認前の案を確定仕様や実装指示として扱わない。
5. フェーズ完了時に要件と判断IDへの対応、隣接文書との整合、未決事項、単体で確認できる振る舞いを点検する。実装計画とテスト計画は、承認済みの詳細設計から別途作る。

## 前提と現在の承認境界

- `.docs/04_approval/adr-0001-rust-core.md` は `Accepted`。Rust Core、versioned C ABI、4言語の薄いwrapper、組み込みLinux対応を前提とする。
- `.docs/11_requirements/mvvm-ui-library-requirements.md` と `.docs/12_design/design-overview.md` は `Accepted`。設計範囲と初期縦断シナリオの参照元として扱う。要件内の未決事項は別途判断する。
- `.docs/12_design/runtime-contract.md` のR1～R4は承認済み。後続仕様として残された事項は別途判断する。
- `.docs/12_design/abi-spec.md` はA1～A32、`.docs/12_design/mvvm-semantics.md` はP1～P4・B1～B4・C1～C4・D1～D4・V1～V8、`.docs/12_design/language-bindings.md` はL1～L4を承認済みと記す。各文書の未定義事項は承認済みとはみなさない。
- `abi-spec.md` のC宣言は設計レビュー用であり、配布用ヘッダや実装済みAPIではない。

## フェーズ一覧

チェックは「レビュー案作成、利用者確認、同一文書への結果反映、整合確認」が終わった時点で付ける。番号は推奨順であり、前提が未承認のフェーズは進めない。

### 基盤と初期縦断シナリオ

- [x] **DD-00 出典と判断の対応表** — `.docs/13_detail_design/00-decision-map.md`。上位文書の状態、承認済みID、未決事項、初期縦断シナリオへの対応を整理（2026-09-27承認）。
- [x] **DD-01 Handle登録と検証** — `01-handle-registry.md`。トークン発行、種類・Runtime照合、retain/release、無効・二重releaseの扱い、検証観点（DD-HND-01～06承認済み、2026-09-27）。前提: R1/A1の承認済み契約。
- [x] **DD-02 所有権と終了** — `02-runtime-lifetime.md`。公開参照と内部参照、Subscription・Bindingの参照方向、shutdown、worker release後の破棄順（DD-LIFE-01～08承認済み、2026-09-29）。後日確認されたcallback内shutdownの優先dispatch方針はBD-36に記録し、API status/完了観測の詳細は未決として管理。
- [x] **DD-03 UI threadと投稿キュー** — `03-dispatcher.md`（DD-DSP-01～08承認済み、2026-10-05）。UI thread判定、FIFO post queue、dispatch/通知の公平性、wake-up、`UiLoopBridge`境界、shutdown優先control taskを決定。status/完了観測とUI loop停止時の同期worker動作はDD-06/16・wrapper設計へ送る。
- [x] **DD-04 Callbackと再入** — `04-callback-lifecycle.md`（DD-CB-01～09承認済み、2026-10-07）。closure所有権の遷移、解除とin-flight callback、自己解除、通知の非再帰、callback種別別再入、wrapper例外とdestroy、イベントメソッドの事前検知候補と識別marker案を整理。詳細な検知ルールと各言語toolingは追加候補DD-24へ切り出す。前提: DD-02～03、R3・R4の承認済み判断。
- [x] **DD-05 値の受渡しと失敗契約** — `05-boundary-values-errors.md`（DD-VAL-01～07承認済み、2026-10-07）。UTF-8、buffer、error handle、出力初期化、失敗時の原子性を整理。F64/DECIMAL個別ABIやerror catalog等は後続設計へ送る。前提: DD-01～04と該当するABI判断。
- [x] **DD-06 ABIシンボルと互換性** — `06-abi-surface.md`（DD-ABI-01～10承認済み、2026-10-07）。シンボル命名、version/feature query、status/error共通面、UI loop接続hook、shutdown受付と完了の表現、互換規則を決定。未決境界はDD-16、DD-18～23へ送る。
- [x] **DD-07 Propertyの保存と通知** — `07-property-notification.md`（DD-PROP-01～10承認済み、2026-10-08）。5値型のProperty ABI、等価判定、setter、通知queue、購読・解除、shutdown時の振る舞いを決定。将来の内蔵値変換APIとparse/format規則は後続設計へ送る。作業計画: `dd07-property-notification.md`。
- [x] **DD-08 Bindingの伝播とグラフ** — `08-binding-graph.md`（DD-BIND-01～09承認済み、2026-10-09）。初期同期、OneTime/OneWay/TwoWay、更新順、循環制約、解除、Bindingエラーと因果連鎖診断境界を決定。前提: DD-07、B1～B4・D1～D4、A13～A16、BD-03。作業計画: `dd08-binding-graph.md`。
- [x] **DD-09 ValidationとView入力** — `09-validation-input.md`（DD-VALID-01～10承認済み、2026-10-11）。Property Validator、View入力拒否、TextBoxの確定値と表示値、理由の保持・解放を決定。IME/Controlの未定義部分は保留。前提: DD-07～08、V1～V8、A17/A18・A25～A28・A30～A32。作業計画: `dd09-validation-input.md`。
- [x] **DD-10 非同期Command** — `10-async-command.md`（DD-CMD-01～12承認済み、2026-10-11）。実行可否、状態遷移図、完了報告、協調的キャンセル、終了・失敗時の後始末を決定。前提: DD-03～09、C1～C4、A21～A24、BD-26。作業計画: `dd10-async-command.md`。

### UIとplatform（対応する基本設計の判断後）

各フェーズの着手時に、該当する基本設計の判断が承認済みかを確認する。未決の上位事項は詳細設計側だけで確定しない。DD-11の前提となるBD-04/05/11は承認済みであり、その未決境界は各DD文書で管理する。

- [x] **DD-11 UI treeと要素寿命** — `11-ui-tree-lifetime.md`。親子・所有、取り外し、Detached subtree、Binding/Subscriptionの寿命境界を整理（DD-TREE-01～08承認済み、2026-10-11）。前提: DD-02、BD-04/05/11。作業計画: `dd11-ui-tree-lifetime.md`。
- [x] **DD-12 Layout** — `12-layout.md`。Measure/Arrange、サイズ制約、変更時の再計算、StackPanelの最小動作を定義（DD-LAYOUT-01～08承認済み、2026-10-12）。前提: 承認済みDD-11、BD-06。作業計画: `dd12-layout.md`。
- [x] **DD-13 InputとFocus** — `13-input-focus.md`。正規化keyboard/text/pointer入力、active Window/Focus、Logical event routingを扱う（DD-INPUT-01～10承認済み、2026-10-12）。key bindingとFocus traversal/UI actionの対応付けは対象外（付加価値候補: `keyboard-actions-value-memo.md`）。前提: 承認済みDD-11/12、BD-07～10。作業計画: `dd13-input-focus.md`（2026-10-12完了）。
- [x] **DD-14 TextBoxとIME** — `14-textbox-ime.md`。編集候補、確定、composition、Validationとの接続、破棄時の整合（DD-TEXT-01～12承認済み、2026-10-12）。Unicode内部位置単位、TextBox自身のProperty Validator拒否、IME candidate退避失敗などは未決事項として記録。前提: 承認済みDD-09/13、BD-09/10/23/24。作業計画: `dd14-textbox-ime.md`（2026-10-12完了）。
- [x] **DD-15 初期Control** — `15-controls-v1.md`。Window、StackPanel、TextBox、Text、Buttonの最小契約とCommand/Property接続（DD-CONTROL-01～12承認済み、2026-10-12）。§7の未決事項は後続へ維持。key/action bindingは対象外。前提: 承認済みDD-08～14、BD-11～13。作業計画: `dd15-initial-controls.md`（2026-10-12完了）。
- [x] **DD-16 描画とPlatform adapter** — `16-platform-rendering.md`。共通Renderer/Platform Adapter境界、Window/surface lifecycle、frame scheduling、DD-03 event loop接続、raw input/IME接続（DD-PLAT-01～10承認済み、2026-10-12）。Window close request後の判断・Core Window寿命、寸法/scale、concrete backend/ABI等は未決として後続へ維持。作業計画: `dd16-platform-rendering.md`（2026-10-12完了）。
- [x] **DD-17 Accessibility** — `17-accessibility.md`。Core Accessibility snapshotとLogical TreeのOS provider mapping、role/name/state/value/action/focus、通知とAT actionの接続（DD-ACC-01～10承認済み、2026-10-12）。§8の未決事項は維持。前提: 承認済みDD-13～16、BD-13。作業計画: `dd17-accessibility.md`（2026-10-12完了）。

### 言語境界と検証

- [x] **DD-18 C++ wrapper** — `18-wrapper-cpp.md`。RAII、callback保持、明示解除、例外/error変換、thread-affine handleの破棄（DD-CPP-01～15承認済み、2026-10-12）。例外無効buildのnon-throw経路、利用者設定callback exception handler、C++内pending-cleanup registryを記録。registry詳細と公開API型は後続API設計へ保留。前提: 承認済みDD-01～17、L1～L4。作業計画: `dd18-wrapper-cpp.md`（2026-10-12完了）。
- [x] **DD-19 C# wrapper** — `19-wrapper-csharp.md`。P/Invoke、delegate保持、Dispose、GCHandle callback state、thread affinityと例外/error変換（DD-CS-01～14承認済み、2026-10-12）。BD-18のRID package方針を維持し、loader/TFM/cleanup implementationは後続へ送る。前提: 承認済みDD-01～18、L1～L4。作業計画: `dd19-wrapper-csharp.md`（2026-10-12完了）。
- [x] **DD-20 Rust safe API** — `20-wrapper-rust.md`。typed owned/borrowed handle、thread marker、dispatcher、Dropと明示終了、callback panic containment（DD-RS-01～15承認済み、2026-10-12）。cleanup registry/API詳細とCargo/link/loadは後続へ保留。前提: 承認済みDD-01～19、L1～L4。作業計画: `dd20-wrapper-rust.md`（2026-10-12完了）。
- [x] **DD-21 Python 3 wrapper** — `21-wrapper-python.md`。承認済みBD-14のctypes、callable保持、明示close/context manager、owner finalizerからのcleanup handoff、Python main/host thread終了時のRuntime shutdown signal、GIL/thread、例外変換（DD-PY-01～21承認済み、2026-10-12）。通常終了hook、未捕捉例外、`os._exit()`等の保証境界を含む。前提: 承認済みDD-01～20、BD-14/15/17、L1～L4。作業計画: `dd21-wrapper-python.md`（2026-10-12完了）。
- [x] **DD-22 Buildと配布** — `22-build-distribution.md`。target別native artifact build/profile、4言語package接続、canonical artifact catalog、manifest/signature/trust、target verification/support境界（DD-BUILD-01～11承認済み、2026-10-12）。exact toolchain、package/catalog運用、key custody、正式support matrix等は未決として維持。前提: 承認済みDD-06・DD-16・DD-18～21、BD-16～22/28～32。作業計画: `dd22-build-distribution.md`（2026-10-12完了）。
- **DD-23 再分類（詳細設計フェーズから除外）** — 旧縦断検証案の内容は製品詳細設計ではなく、要求テストのルール・契約として `.docs/30_release_test/requirements-test-manifest.md` へ再編。旧案は未承認のためDD判断・DD完了として数えず、原案ファイルは利用者指示で削除。記録: `.docs/20_todo/requirements-test-manifest.md`。
- [ ] **DD-24 イベントメソッド制約と事前検知（追加候補）** — `24-event-method-analysis.md`。callback種別ごとのerror/warning/infoルールと共通の検知分類、C++/C#/Rust/Pythonで異なるmarker・解析・計測方法、間接呼出しとdynamic callableの限界、診断とCI統合を設計する。前提: DD-04承認、DD-07/09/10/14のcallback固有契約、DD-18～21の各wrapper、BD-03。DD-04で追加候補として確認済み。着手・範囲は別途計画レビューで決める。

## 各フェーズの完了条件

現在の利用者方針（2026-10-12）: DDはBDを具体化し、クラス設計、データフロー、所有と参照、ライフサイクル、言語特有の仕様をまとめる。クラス図・メソッド名・signature・公開契約のAPI具体化もDDに含め、独立したAPI設計工程は設けない。必要ならDD内の章・分冊にする。計画済みDD-00～22の完了後に未決を含むAPI具体化項目を整理する作業順は維持する。旧DD-23の検証案は詳細設計対象外へ再分類した。既存DDの「後続API設計へ」という留保はDD内の具体化待ちとして引き継ぎ、個別API名等の未決を今回の構成変更で確定しない。過去のAPI別章方針は進行履歴として保存する。追加候補DD-24は別途着手・範囲を判断する。

- 対象範囲と対象外、上位判断ID、追加判断ID、レビュー結果が追跡できる。
- 依存先との所有権・スレッド・エラー・終了時の契約に矛盾がない。
- 単体で確認できる正常系と主要な失敗系が記され、未決事項に次の担当フェーズが付いている。
- 判断が未承認ならチェックを付けず、次フェーズの確定仕様として引用しない。

### DD-04 レビュー案の作業計画

1. R3/R4、ABIのA9～A20・A28、Property通知P3/P4、L4、承認済みDD-01～03からcallbackの所有権・thread・再入契約を抽出する。
2. 登録成功/失敗、解除、callback実行中の解除、所有者破棄、shutdownをclosure状態遷移とdestroy順で提案する。
3. callback種別ごとの再入制限と例外の扱いを照合し、共通のABI境界規則と後続フェーズへ委ねる固有意味論を分ける。各言語でcallback対象メソッドを識別する属性・attribute・proc-macro・decoratorの案について、静的解析と実行時計測の用途を分けて検討する。
4. 自己解除、実行待ちcallbackの無効化、destroyの一回性、例外後の状態、shutdownとの競合を検証ケースにする。
5. 事前検知の候補を「禁止違反」「警告」「調査候補」に分類し、DD-04で共通の責務まで定められるか、個別ルールと解析器設計を追加フェーズへ送るかを整理する。
6. 判断ID付きで `.docs/13_detail_design/04-callback-lifecycle.md` に未承認のレビュー案を作成し、未決事項とDD-07/09/10/14・wrapper/tooling設計等への依存を明記する。承認後にのみ本計画を完了にする。

対象外: 新しい公開ABI関数・callback別の完成したAPI設計、値型・Validation結果・Commandの固有意味論変更。上位承認済みのR3/R4、A/P/B/C/V/L判断とDD-01～03を上書きしない。

### DD-05 値の受渡しと失敗契約の作業計画

1. Accepted ADR・要件・基本設計に加え、A2/A5～A8/A15/A18、BD-01/27、承認済みDD-01～04を確認し、既存の境界契約を出典付きで整理する。
2. 入力pointer/length、UTF-8、caller buffer、値型のscalar受渡しについて、借用期間・コピー時点・null/zero-length・buffer不足時の出力規則を提案する。
3. `ui_status`と任意`out_error`、error snapshotの不変性、必要な全出力の初期化、error生成失敗時の扱いを一貫した失敗契約として提案する。
4. 入力検証・コピー・確保に失敗したときの状態不変と出力初期化を提案し、操作別の原子性（A8、Property操作、Collection batch等）の範囲を越える一律rollbackは導入しない。
5. 対象外と後続先を明記する。F64/DECIMALのProperty個別ABI・DECIMALの符号化、配列/Collection表現、error code catalogの数値・全code、各wrapperの例外/Result型は後続設計へ送る。
6. DD-VAL判断ID付きで `.docs/13_detail_design/05-boundary-values-errors.md` にレビュー案を作り、4言語から同じ意味で検証できる正常系・失敗系を記す。承認を受けた後にだけ本計画と記憶用状況を更新する。

対象外: A5やBD-01の値型意味論変更、Property/Binding/Validation/Command固有の結果意味論、公開C ABI宣言の実装済み扱い。

### DD-06 ABIシンボルと互換性の作業計画

1. Accepted ADR/要件/design overview、A1～A32、BD-27/28、承認済みDD-01～05を照合し、確定契約・現行C宣言・明示的未決を分ける。
2. ABI major付きsymbol名前、runtime MAJOR.MINOR.PATCH query、feature IDの安定性、minor追加・patch fix・breaking majorの規則を提案する。
3. 共通`ui_status`とerror domainの初版catalog、未知値を扱うwrapper規則を提案する。機能固有のerror codeと意味論はDD-07～10等の担当設計に残す。
4. DD-03/BD-36から送られたshutdownの同期/非同期要求と「受付済み／完了済み」の表現、UI loop bridgeのABI登録点・callback契約を小範囲で提案する。DD-16へ残すplatform固有接続・event loop実装を区別する。
5. 既存symbolとstatusの互換表、必要featureの利用条件、旧header/new runtimeおよび旧runtime/新wrapperの確認ケースをまとめる。
6. DD-ABI判断ID付きで `.docs/13_detail_design/06-abi-surface.md` にレビュー案を保存し、`git diff --check`とリンク・判断状態を確認して利用者レビューへ提示する。承認後にのみ全体計画・状況メモを完了更新する。

対象外: 各APIの完全なC宣言一覧を実装済みheaderとして発行すること、Property/Binding/Validation/Commandの機能固有結果意味論、package manager/loaderの探索順・artifact配布実装、platform別event loop adapterの内部実装。

## 作業履歴と現在位置

DD-00の対応表は2026-09-27に承認済み。DD-01は利用者の着手指示後にレビューし、DD-HND-01～06の承認を受けて同日に完了した。DD-02は2026-09-29に利用者承認済み。DD-03は利用者承認を受け、DD-DSP-01～08を2026-10-05に承認済みとして反映した。未決のstatus/完了観測と同期workerのUI loop停止時動作はDD-06/16・wrapper設計へ送る。DD-04はイベントメソッドの事前検知候補をerror/warning/infoに分類し、DD-CB-08/09とDD-24追加候補へ記録した。DD-CB-01～09は利用者承認を受け、2026-10-07に承認済みとして反映した。DD-24は追加候補のままであり、未着手。DD-05はDD-VAL-01～07の承認を受け、2026-10-07に承認済みとして反映した。DD-06は2026-10-07に着手指示と内容承認を受け、DD-ABI-01～10を同日承認済みとして反映した。利用者定義error codeはdomain/codeで表し、auto相当採番の安定性を求める。DD-07は2026-10-07に着手指示を受け、DD-PROP-01～10の未承認レビュー案を作成した。利用者は将来、文字列↔整数/DECIMAL等の基礎的な相互変換をライブラリ内蔵する意向を示した。変換API・規則はDD-07の未承認範囲外として後続検討に記録した。

2026-10-08: DD-07のDD-PROP-01～10は利用者承認を受け、同日承認済みとして反映した。レビュー中に示された将来の文字列↔整数/DECIMAL等の基本変換をライブラリ内蔵する意向は記録し、API・parse/format規則を後続設計へ送った。

2026-10-09～11: DD-08は2026-10-09にDD-BIND-01～09の承認を受け、Binding伝播・グラフ・因果診断の判断を同日反映した。DD-09は2026-10-09に着手し、2026-10-11にDD-VALID-01～10の承認を受けて同日反映した。TextBox自身のProperty Validator拒否時の編集状態、IME/Undo/Redo詳細、subscription間の厳密な通知順などは未決として各文書に残した。DD-10は2026-10-11にDD-CMD-01～12と状態遷移図を利用者承認済みとして反映した。operation ID枯渇、error catalog、worker wrapper所有などを未決として残した。DD-11の着手指示はこの作業開始前まで保留中だった。

2026-10-11: DD-11に着手し、同日DD-TREE-01～08の内容承認を受けた。Logical Treeのみが所属所有参照を持つこと、tree変更制約、Detached subtreeの維持、Binding/Subscriptionの非自動解除、remove後の再所属方針を反映した。ABI symbol、Detached時のControl API一覧、Visual/Platform再生成、focus/IME、template owner等は後続設計に残す。DD-12の着手は別途指示を待つ。

2026-10-11～12: DD-12に着手し、DD-LAYOUT-01～08の内容承認を2026-10-12に受けた。Core Measure/Arrange、制約、StackPanelのVertical/Horizontal配置、Align、Layout invalidation、失敗時のcomplete layout snapshot保持を反映した。寸法単位、scale/rounding、ABI表現、Control別Sizer、Renderer接続は未決として残す。DD-13の着手は別途指示を待つ。

2026-10-12: DD-13に着手。承認済みBD-07～10、DD-11/12を基に正規化Input、active Window/Focus、Focus遷移、keyboard/text配送、Logical event routingの未承認レビュー案を作成した。利用者指摘を受け、Tab/Shift+Tab等のkeyとFocus移動の対応付けはDD-13の契約から除外し、Ctrl+Z等のshortcut/action bindingも含む付加価値候補を `keyboard-actions-value-memo.md` に分離した。状態フロー図をMermaid state diagramへ変更した。利用者はDD-INPUT-01～10を承認し、判断記録を `.docs/13_detail_design/13-input-focus.md`、計画を `dd13-input-focus.md` に反映してDD-13を完了した。公開event ABI・TextBox編集・pointer hit-test等は後続へ分離し、DD-14の着手は別途指示待ち。

2026-10-12: DD-14に着手。承認済みBD-09/10/23/24、DD-09、DD-13を基に、TextBox編集状態、caret/selection、IME session遷移、Validator接続、失敗時保持のDD-TEXT-01～12未承認レビュー案を作成した。Unicode内部位置単位、TextBox自身のProperty Validator拒否、IME commit退避失敗などは未決として提示する。キー割当とUndo/Redoは対象外。DD-15には進まない。

2026-10-12: 利用者はDD-TEXT-01～12を承認した。DD-TEXT-09～12で列挙したProperty Validator拒否時の入力状態、IME candidate退避失敗、Unicode内部位置単位、通知細部は未決として維持することを含め、判断記録を `.docs/13_detail_design/14-textbox-ime.md`、計画を `dd14-textbox-ime.md` に反映してDD-14を完了した。Undo/Redo・key binding・公開API/ABI・OS固有IME実装は対象外。DD-15の着手は別途指示待ち。

2026-10-12: DD-15に着手。承認済みBD-11～13およびDD-08～14を基に、Widget共通意味論、Window/StackPanel/Text/TextBox/Button、Property/Binding/Command・Accessibility接続のDD-CONTROL-01～12未承認レビュー案を作成した。ancestor enabled semantics、Focusable既定値、visibility初期対応、Button Command association/owner等を利用者確認待ちとする。key/action bindingは対象外。DD-16には進まない。

2026-10-12: 利用者はDD-CONTROL-01～12を記載どおり承認した。承認状態を `.docs/13_detail_design/15-controls-v1.md`、作業計画、フェーズ一覧へ反映し、DD-15を完了した。§7の未決事項は確定せず後続へ維持する。key/action bindingは承認対象外。DD-16への着手は別途指示待ち。

2026-10-12: 利用者の着手指示を受けてDD-16に着手。BD-12共通Renderer/Platform Adapter方針、DD-03 `UiLoopBridge`、DD-12 Layout結果、DD-13 Input/Focus、DD-14 IMEとの接続を対象とする。寸法単位/scale、具体backend/library、公開ABI、OS Accessibility、Window close policy等の未決事項を上位判断なしに確定しない。レビュー案は未承認であり、DD-17には進まない。

2026-10-12: 利用者はDD-PLAT-01～10を記載どおり承認した。承認記録を `.docs/13_detail_design/16-platform-rendering.md`、作業計画、フェーズ一覧へ反映してDD-16を完了。close request後のaccept/decline/defer、Core Window寿命、寸法/scale、具体backend/ABI等は未決のまま維持した。続けて利用者の指示を受けDD-17に着手し、Accessibility mappingの計画と未承認レビュー案を作成する。

2026-10-12: 利用者はDD-ACC-01～10を承認した。承認記録を `.docs/13_detail_design/17-accessibility.md`、作業計画、フェーズ一覧へ反映しDD-17を完了した。DD-15由来の意味論、OS固有profile/bus依存、AT実機・適合範囲は未決のまま維持。利用者の次フェーズ着手指示を受け、DD-18の計画 `.docs/20_todo/dd18-wrapper-cpp.md` と未承認レビュー案 `.docs/13_detail_design/18-wrapper-cpp.md` を作成した。DD-02 worker release enqueue failureとC++ destructorの整合をDD-18内の未決として明示。DD-19へは進まない。

2026-10-12: 利用者はDD-CPP-01～15を承認した。例外無効buildでのnon-throw result経路、例外有効buildにおける利用者設定callback exception handler、release failure時のC++内pending-cleanup registryを追加しDD-18を完了。追加API設計章はDD-00～23の計画済み詳細設計完了後に作成・整理する方針で保留。利用者方針に沿ってDD-19 C# wrapperの計画と未承認レビュー案を作成する。

2026-10-12: 利用者はDD-CS-01～14を承認した。C# fixed-width P/Invoke、managed callback root/destroy、Dispose/cleanup manager、status/error/thread/shutdownの境界を反映してDD-19を完了。SafeHandle最終採用、TFM、interop generator、loader、registry実装、公開API名は後続へ維持。追加API設計章は計画済みDD完了後に組み立てる方針を維持し、利用者の次フェーズ指示を受けDD-20 Rust safe APIを開始。
2026-10-12: Accepted ADR/要件、承認済みL1～L4・A1～A32・BD-20およびDD-01～19を照合し、`.docs/20_todo/dd20-wrapper-rust.md` と未承認レビュー案 `.docs/13_detail_design/20-wrapper-rust.md` を作成。Rust owned/borrowed handle、`Send`/`Sync`境界、dispatcher closure、Drop cleanup ticket、FFI callback panic containment、error/shutdown境界をDD-RS-01～15として提案した。crate/API名、MSRV/edition、registry実装、panic時の機能別挙動、Cargo/link/load詳細は未決。承認待ちのためDD-20は未完了。
2026-10-12: 利用者はDD-RS-01～15を承認した。承認状態と記録を `.docs/13_detail_design/20-wrapper-rust.md` へ反映し、作業計画・フェーズ一覧・状況メモでDD-20を完了にした。cleanup registry実装、crate/API名、MSRV/edition、callback固有panic挙動、Cargo/link/load詳細は未決のまま維持。続けて承認済みBD-14/15/17とDD-01～20を確認し、Python `ctypes` wrapperの計画 `.docs/20_todo/dd21-wrapper-python.md` を先に作成した。Python callback lifetime、GIL/thread、finalizer cleanup、exception/error境界をレビュー対象とし、DD-21設計案の作成に進む。
2026-10-12: BD-14/15/17と承認済みDD-01～20を確認し、Python `ctypes` wrapperの作業計画 `.docs/20_todo/dd21-wrapper-python.md` と未承認レビュー案 `.docs/13_detail_design/21-wrapper-python.md` を作成した。DD-PY-01～16でfixed-width ctypes prototype、owned token/explicit close、finalizer cleanup、callback rooting/destroy、GIL/UI thread、例外handler/error、BD-17 distribution境界を提案。Python version/free-threaded support、finalizer/registry実装、event-loop safe-point integration、handler API、package/loaderは未決であり、DD-21は承認待ち。
2026-10-12: 利用者の追加方針を受けDD-21レビュー案と作業計画を改訂。Python special method overrideを一律に扱わず、method別base chain-up metadataとruntime guard/BD-15 analyzerの候補を追記し、`__del__`をcleanup正本・base-call enforcementにしない案を追加。GIL有効/無効に依存しない明示同期、free-threaded buildでの検証候補も追加した。DD-PY判断は全て未承認のまま。
2026-10-12: 利用者はDD-PY-01～21を承認。通常終了・未捕捉例外の網羅を含め、host UI loop終了直前の`aboutToQuit()`相当通知、Python main未捕捉例外時の`sys.excepthook`と既存hookへの委譲、wrapper所有worker例外、callback例外、`SystemExit`、`atexit`補助経路を整理した。`os._exit()`、fatal error、致命的signal/killはin-process cleanup保証外と明記。承認状態を `.docs/13_detail_design/21-wrapper-python.md` と `.docs/20_todo/dd21-wrapper-python.md` に反映してDD-21を完了。hookの合成・解除、host loop adapter、callback/root drain、worker例外policy、公開API/配布は後続設計へ残す。

2026-10-12: 利用者がDD-PY-01～21を承認したため、詳細設計・作業計画・フェーズ一覧の承認状態を照合し、DD-21を完了に更新した。次フェーズDD-22の計画 `.docs/20_todo/dd22-build-distribution.md` と未承認レビュー案 `.docs/13_detail_design/22-build-distribution.md` を作成。BD-16～22/28～32を前提に、target build/profile、4言語package境界、canonical artifact catalog、manifest/signature/trust、verification stateを判断ID化した。JCS/RFC 8785、Ed25519、SHA-256は候補のまま。toolchain、package ID、catalog運用、trust key custody、support matrixの未決範囲を維持し、DD-22は利用者レビュー待ち。
2026-10-12: 利用者はDD-BUILD-01～11を承認。承認状態を `.docs/13_detail_design/22-build-distribution.md`、`.docs/20_todo/dd22-build-distribution.md`、フェーズ一覧、進捗メモへ反映し、DD-22を完了にした。RFC 8785 JCS、Ed25519、SHA-256は提案範囲として承認された。exact toolchain、package/catalog運用、key custody、正式support matrix、manifest wire schema等の実装詳細は未決として維持。
2026-10-12: 利用者のDD-22承認を受けてDD-23へ着手。承認済み要件、設計判断、DD-01～22、release/version test規約との照合後、作業計画と未承認レビュー案を作成した。4言語共通縦断scenario、契約・ABI・target/package verification、release/version gateを対象にし、未設計Collection詳細、CI/test tooling、性能閾値を暗黙に確定しない。2026-10-14にテストルール・契約へ再分類し、原案ファイルを削除した。

2026-10-14: 要求再編後の現行IDと承認範囲を再照合し、DD-23文書版0.2のレビュー案を更新した。共通scenarioをRQ-03-001～004/RQ-09-001～011、platformをRQ-11、measurementをRQ-13-001～011へ対応させた。要求test対応表のRQ-case台帳、unit-test方針の実装単位管理とDD-23の横断verification設計の役割を分け、要求から下流への参照を作らないことを明記した。BD-16のnative/cross-build/hardware gateを再確認。DD-VERIFY-01～12は未承認のためDD-23の完了チェックは保留。

### 上位設計への確認事項

- [x] **Runtime shutdownのdispatch/完了契約（DD-26へ具体化・適用済み）** — `.docs/12_design/BD-36_runtime-shutdown-dispatch-upper-design.md`、`.docs/12_design/runtime-contract.md` R2～R4、`.docs/13_detail_design/02-runtime-lifetime.md` §6。callback/event中の要求を許可し現在の処理後すぐ優先実行する方針は利用者から提示済み。非同期呼出しは完了を待たず、同期worker呼出しはUI threadでの完了を待つ区分も確認済み。statusによる受付/完了の区別、完了観測、UI loop停止時の同期worker待機・timeout/cancelを確定し、runtime-contract等へ反映する。
