# 要求・設計分離 / 全BD・DD文書の調査台帳

- 文書版: 0.6
- 状態: 全案反映承認済み・責務分離の正本適用完了（2026-10-17）。未設計課題は別計画へ移管
- 調査日: 2026-10-12
- 計画: [要求・基本設計・詳細設計の責務分離とID整理計画](requirements-design-separation.md)
- 上位根拠: [要求仕様](../11_requirements/reqirements-specification.md)、[文書管理要求](../11_requirements/RQ-16_document-governance.md)
- 全案件レビュー: [66項目の案・適用記録索引](requirements-design-separation-review/index.md)、[coverage確認](requirements-design-separation-review/coverage.md)

## 現在の適用結果

[全案適用結果](requirements-design-separation-application-result.md)と[未設計の後続課題](requirements-design-separation-follow-up.md)を参照する。以下の走査・案作成時点の差異や未承認表示は当時の記録であり、現在の承認状態は本節と承認台帳を正本とする。

## 1. 読み方と走査範囲

`.docs/12_design/` の42文書（BD-00～36と名称にBDを含まない5文書）、`.docs/13_detail_design/` のDD-00～22およびDD-23再分類メモを確認した。判断表だけでなく、本文の契約、後続・対象外事項、検証例、承認記録も対象とした。現行RQの要求本文と照合し、旧要求の節番号だけを対応根拠にしない。

以下の `B00～B36 / S01～S05 / D00～D23 / X01～X10` は**調査用項目ID**であり、新しい要求ID・設計IDではない。チェック欄は正本整理の完了を示す。全件のレビュー案は作成済みで、正本適用済みの範囲だけをチェックする。下表の調査時点の論点と、最新の案・解決記録は一括レビュー索引から区別して参照する。

| 区分 | 意味・後続処理 |
|---|---|
| 重複 | 現行RQに主旨がある。独立した要求定義をBD/DDから除き、上位根拠と設計上の制約として引用する。意味を保つ要約・設計説明まで削除する必要はない |
| 補足候補 | RQに主旨はあるが、BD/DDだけに具体的な外部保証・例外・除外条件がある。保証する条件はRQへ移すか明記し、設計にはそれを満たす方式だけを残す |
| 不整合 | 要求と設計、または設計相互で意味・対象段階・承認範囲が一致しない。機械的に移管せず差異を解決する |
| 未決・候補 | 選択肢、将来意向、未採用機能。要求の採否待ちと設計方式待ちに分け、どちらも確定扱いにしない |
| 別成果物 | APIの宣言・公開形、secure-designの対策、具体テスト・CI運用、進行履歴などへ整理する |

designとbasic-designは同一のBDを指す。RQは利用者から見える提供機能・条件・例外・除外・保証・結果を定める。BDは層の責務、データと所有の流れ、主要状態遷移などRQを満たす上位手段を示す。DDはクラス/interface、method・signature・呼出し順、詳細な処理・データフロー・所有参照・ライフサイクル・状態遷移・言語固有実現を具体化する。表中の「API」はDD内のinterface設計、「secure」はSDを表し、独立API設計への移管先ではない。API別章に関する過去方針は経緯として保持し、現在はDD内の章・分冊で整理する。

RQとの対応は同一意味の保証を全てカバーすることを意味しない。「補足候補」の外部動作・例外・保証は要求追加案としてレビューし、元BD/DDの設計承認を要求承認へ読み替えない。利用者から観測できる既定値も製品契約ならRQに置く。内部実装だけに関わる値や選択は設計に残す。

## 2. 先に解く横断課題

| 完了 | 項目ID | 対象箇所・発見内容 | 対応to-do |
|---|---|---|---|
| [x] | X01 | `abi-spec.md` A11/A14/A19はreleaseだけでは解除しない。DD-LIFE-04/05は最終公開releaseで自動解除。DD-PROP-08はreleaseだけではcancelしない。DD-VALID-06もDD-LIFEとDD-PROPを同時参照 | release・cancel・終了時の外部契約と例外をRQ-05/07へ置き、採用済み契約の内部実現として登録owner・参照グラフ・in-flight処理・状態遷移・cleanup順をBD/DDへ記載する。既存承認との矛盾と変更対象を示してレビューする |
| [x] | X02 | RQ-04-002は単精度・倍精度をABIで扱える要求。BD-01は初期PropertyのF32非採用、DD-PROP-01/03はF64のみ | ABI全体とProperty kind、初期段階と製品範囲を区別し、直ちに矛盾とは断定しない。F32をどのAPI・段階で満たすか対応を明記する。BOOL/I64/UTF8/F64/DECIMALの採用範囲・数値保証もRQ-02-002/04へ割り当てる |
| [x] | X03 | BD-36-01/02/04、DD-DSP-08、DD-ABI-09、wrapper群にshutdownの外部保証が分散。runtime-contract R2はUI thread限定の旧記述。BD-36-03は実装提案・未確認のまま | callback内の受付、非同期の非待機、worker同期の完了待機、優先実行、停止済みloop時の保証をRQ-02-013/RQ-06/RQ-05へ整理する。停止loopのtimeout/cancelは未決の要求境界と実現方式を分ける。BD-36-03を他IDと一括承認しない |
| [x] | X04 | DD-PY-20/21と§4.1/8/11に、main/host終了・一般例外・host loop終了時のcleanup保証と具体hookが混在。保証対象の記述に対し移植可能な検知元・in-flight handshakeは未決 | 通常終了の対象経路と例外・対象外をRQ-03-004/02-013/05/06に明記する案を作る。`os._exit()`、fatal error、killは保証外を維持。coordinator、hook合成、root drainは設計に残し、実現未確定なのに保証済み実装と記さない |
| [x] | X05 | BD-33～35はCollection/Grid/Tableの意味論・最適化を扱う。RQ-09-019/020/022は次段階採用、TableView/ComboBox/TreeViewはRQ-09-027～029の未採用候補 | 共通Collection/Selection要求とGrid例・個別Control候補を分ける。Grid設計例からTableView採用を導かない。stable ID、選択順、失敗原子性、revision・通知整合性の要求差分を確認する |
| [x] | X06 | DD-LAYOUT-04/05、DD-CONTROL-03/04/05/11、DD-INPUT-07は方向・既定値・有効性・機能除外等を追加。RQ-09-007は方向/既定値を要求で固定しない。DD-15§7は一部を未決として維持 | RQで固定しないことと設計で選ぶことを区別する。実現詳細は設計に残し、初期機能の追加/削減になるものだけ要求レビューする。DD-15の判断表と未決欄の承認境界をID内の文単位で明記する |
| [x] | X07 | BD-03はPython静的警告を任意とする旧例外。BD-15/RQ-15-007/013は解析可能範囲の警告を採用。DD-CB-08/09、DD-PY-17はmarker・禁止事項検知・base-call検査を候補化 | 採用済みの警告能力と、未採用の検知拡張・言語差を分ける。診断能力・保証限界はRQ候補、parser/marker計測は設計、構文はAPI、DD-24は未着手候補として維持する |
| [x] | X08 | DD-BUILD-09はJCS/Ed25519/SHA-256を候補と記すが、§10/11は提案範囲で利用者承認済みと記録。BD-29～32は具体暗号方式を後続へ残す | 要求は署名・独立trust root・検証失敗時拒否等、方式選択はsecure-designへ整理する。アルゴリズムは承認記録を根拠に整理し、未承認へ戻さない。exact parameters/envelope/test vectors等の未決は残す |
| [x] | X09 | BD-00、DD-00、設計骨格、DD-23に旧要求§5の縦断シナリオ、旧要求§12、後続DDの未承認記録、旧ファイル名参照が残る | 現行RQへ対応付け、過去時点の記録と現行仕様を分離する。RQ08廃止を尊重し具体シナリオを要求へ復活させない。BDからDDへの割当は作業計画/移行台帳に置き、仕様の根拠参照は下流から上流とする |
| [x] | X10 | RQ-15には複数要求が1 IDへ集約され、BD/DDにも要求・方式・承認が同じIDにある。DD内のAPI具体化は計画DD完了後の整理を待つ方針、DD-23は未承認 | 意味単位の分割・統合と旧新対応表を準備する。既存RQの安易な再採番・RQ08再利用は行わない。DD内のAPI具体化の本格着手条件、DD-23未承認、各文書の未決事項を保った順序で進める |

## 3. BD番号付き全37文書

「RQ照合・要求へ整理する候補」は要求の正本を決める作業、「設計等に残すもの」は方式・根拠を失わないための作業である。同じ行に複数の意味がある場合は後続の移行台帳で文単位に分割する。

| 完了 / 項目 | 文書・対象箇所 | RQ照合・要求へ整理する候補 | 設計等に残すもの・具体to-do |
|---|---|---|---|
| [ ] B00 | [BD-00](../12_design/BD-00_basic-design-resumption.md) §1～4、特に§2対応表・§3履歴 | 旧要求§12からの判断列挙をRQ-15-001～022等へ対応付ける。独立した要求定義は持たせない | 再開計画・DD割当・承認経緯は計画/履歴。旧prefixなしリンクと当時のDD未承認記述を現行状態と区別する（X09） |
| [ ] B01 | [BD-01](../12_design/BD-01_value-types-upper-design.md) 初期公開範囲表、五値型契約、8バイト案、末尾Collection/Grid案 | 補足候補: 五値型とnull/custom/OBJECT除外、F64のNaN/±0/∞、DECIMAL値域・等価性・範囲外拒否。RQ-02-002、RQ-04-001～003に主旨はあるが全ての数値条件はない（X02） | sign/scale/coefficientの配置、ABI符号化はBD/DD/API。15バイト比較案は未採用調査。Grid案はBD-33との差分・採用履歴を残し共通model側へ整理。parse/formatの将来意向を採用済みAPIにしない |
| [ ] B02 | [BD-02](../12_design/BD-02_ui-transaction-upper-design.md) 判断・規則表 | 重複: 即時setter、後続配信、一般transaction/rollback非提供はRQ-07-005/006/009、RQ-15-010。同期flush非提供は補足確認 | 外側処理区間と通知集約の方式・理由を残し、製品範囲の独立定義をRQ参照へ置換 |
| [ ] B03 | [BD-03](../12_design/BD-03_runtime-notification-upper-design.md) 契約表・限界・停止後 | 重複: RQ-07-008、RQ-15-007。補足候補: 独立連鎖保護、確定値保持、保留/停止診断、静的誤検知・停止保証の限界 | 因果追跡・作業枠・復旧方式は設計。Python旧例外とBD-15の採用差分を履歴化（X07） |
| [ ] B04 | [BD-04](../12_design/BD-04_ui-tree-lifetime-upper-design.md) 所属/寿命表・レビュー項目 | RQ-02-007/013、RQ-05-002/003に重複。補足候補: Detached存続・再所属、親破棄で独立所有参照を無効にしない、複数親/循環禁止 | Core内強参照、peerとCoreの寿命分離は設計。操作可能範囲・再生成失敗の外部結果は要求境界から確認 |
| [ ] B05 | [BD-05](../12_design/BD-05_logical-visual-tree-upper-design.md) 判断表・BD-04整合・決めないこと | 重複: RQ-02-007、RQ-09-026。内部template変更で論理構成を変えない保証をRQ正本へ対応 | Logical/Visualの責務、対応表、所有者は設計。DataContext/resource/name scope等は機能採否と継承方式を分ける |
| [ ] B06 | [BD-06](../12_design/BD-06_layout-upper-design.md) 判断表・Sizer/Locationerカスタマイズ | RQ-02-008、RQ-09-007/008に重複。設定可能性と独自アルゴリズム拡張の初期非採用を区別 | Core配置/Platform環境値、Measure/Arrangeは承認済み方式・制約として維持。単位/rounding、独自拡張の採否と方式を分離 |
| [ ] B07 | [BD-07](../12_design/BD-07_input-event-boundary-upper-design.md) 責務表・後続項目 | RQ-02-009、RQ-09-010、RQ-11-012に重複。初期入力種別・cancel/capture/gesture等は採用範囲確認 | raw event正規化・変換位置は設計。未決一覧を「機能要求の採否」と「配送方式」に仕分ける |
| [ ] B08 | [BD-08](../12_design/BD-08_focus-upper-design.md) 管理単位表・後続項目 | RQ-02-010、RQ-09-009/010に重複。補足候補: 非active時focusなし、detach時無効化、OS候補復帰・multi-seat除外 | Runtime一元管理、Visual→Logical対応は設計。FocusScope/navigationは要求候補、復帰手順は設計候補 |
| [ ] B09 | [BD-09](../12_design/BD-09_ime-focus-transition-upper-design.md) 状況表・保証しない範囲 | RQ-02-011、RQ-15-006に重複。補足候補: 再Focusで編集継続、OS候補/session復元保証なし、detach時保持 | Core編集値とnative sessionの分離・処理順は設計。Escape/Undo候補を採用済みにしない |
| [ ] B10 | [BD-10](../12_design/BD-10_ime-platform-boundary-upper-design.md) 責務表・範囲 | RQ-02-011、RQ-15-006、RQ-04境界に重複。補足候補: 4言語から共通編集状態観測 | IME/candidate UIとCore編集状態の責務分離、event接点は設計。surrounding text等の採否とOS API選択を分ける |
| [ ] B11 | [BD-11](../12_design/BD-11_initial-control-contract-upper-design.md) Control責務表・決めないこと | 初期5Control、Widget/Container分類と機能はRQ-09-001～007、RQ-15-004へ集約済み | 分類構造・subsystem接続は設計。Window一rootの要求は現行RQ-09-001を参照し、当時の「子数未決」と現在の契約を区別 |
| [ ] B12 | [BD-12](../12_design/BD-12_rendering-platform-upper-design.md) 描画/対応順/実機表 | 重複: RQ-11、RQ-13、RQ-15-001～003/017。対応優先順と正式supportは同義にしない | Core/Renderer/Adapterの構造、Linux共通資産活用は設計。性能測定caseはtest、正式matrix採否は要求 |
| [ ] B13 | [BD-13](../12_design/BD-13_accessibility-upper-design.md) Control表・推奨契約・決めないこと | role/name/state/value/focus/action、Logical基礎はRQ-10-001～009、RQ-15-018に重複。補足候補: 空name、初期各Controlのaction範囲 | snapshot/OS写像の責務を残す。label/fallback/AT製品/認証保証の採否とprovider実現を分ける |
| [ ] B14 | [BD-14](../12_design/BD-14_python-wrapper-upper-design.md) 判断・初期wrapper契約 | ctypes/extension不要はRQ-12-007、RQ-15-013に重複。公開raw型非露出やparityはRQ-01/03/04と照合 | ctypes選択理由、prototype/root管理は設計。Python処理系・最低版のsupport採否は要求候補 |
| [ ] B15 | [BD-15](../12_design/BD-15_python-lint-mcp-upper-design.md) 境界表・増分解析・静的範囲 | RQ-15-007/013に重複。補足候補: dynamic境界を安全扱いしない、同一入力diagnostic一致、stale結果を最新と返さない | analyzer/MCP/CLI分担、fingerprint/atomic generationは設計。tool schemaはAPI、workspace/file/cache隔離はsecure-design（X07） |
| [ ] B16 | [BD-16](../12_design/BD-16_build-ci-target-matrix-upper-design.md) CI表・検証状態定義 | RQ-11-001～004、RQ-12-009/010、RQ-15-017に重複。build/runtime/hardwareの保証範囲と正式support採否を分ける | cross-build構成は設計、具体job/board gateはtest/CI計画。代表board一つを両architectureの実機保証にしない |
| [ ] B17 | [BD-17](../12_design/BD-17_python-package-distribution-upper-design.md) 配布表・順序・後続 | RQ-12-007/008、RQ-15-016に重複。補足候補: native未取得を利用可能と表示しない、ABI互換依存、ARM未確認tag拒否 | wrapper/native package分離の構成、探索方式を設計へ。具体index/tag/versionは未決。導入経路とsupport保証を分ける |
| [ ] B18 | [BD-18](../12_design/BD-18_csharp-nuget-native-assets-upper-design.md) 判断・配布構成 | NuGet同梱/RID配置/初期RIDはRQ-12-006へ集約済み。TFM・最低OS・ARM採用は要求候補 | native loader、依存library配置は配布/DD内のAPI具体化。サポートmatrixの独立定義を残さない |
| [ ] B19 | [BD-19](../12_design/BD-19_cpp-cmake-package-upper-design.md) 判断・配布境界 | CMake/relocatable consumer/初期targetはRQ-12-004へ集約済み。最低C++/CMakeの保証は要求候補 | header/source/export target構成は設計。vcpkg/Conan候補、API/exception方針の所管を分ける |
| [ ] B20 | [BD-20](../12_design/BD-20_rust-cargo-package-upper-design.md) 判断・配布表・import契約 | -sys/safe、別runtime、明示target/profileはRQ-12-005へ集約済み。補足候補: 不適合artifactの利用前拒否 | crate/feature/build/link構造は設計/API。registry/cacheは配布、credentials/trustはsecure-design |
| [ ] B21 | [BD-21](../12_design/BD-21_license-upper-design.md) 採用判断・Distribution最低方針 | 文書の中心はライセンス・再配布条件という要求。RQ-15-014と照合し、NOTICE/改変表示/依存license維持の漏れを確認 | ライセンス選択経緯は履歴/ADR、収録処理・inventory検証はrelease/test。商標/CLA/DCO/docs/assets/SBOMは未決候補 |
| [ ] B22 | [BD-22](../12_design/BD-22_support-lifecycle-upper-design.md) 判断・EOL条件 | 文書の中心は保守要求。RQ-15-015に主旨あり。補足候補: 12か月通知の例外条件、EOL後の公開物/記録の継続アクセス | 運用手順・support状態のデータ表現は配布/運用設計。SLA/LTS/正式matrix追加は要求承認へ |
| [ ] B23 | [BD-23](../12_design/BD-23_text-shaping-upper-design.md) 判断・capability matrix・未決 | Latin/日本語・UTF8・保証外scriptsはRQ-15-005に主旨あり。「最低保証候補」と現行要求の採用状態を対応付ける | font discovery/shaping/rasterization分担はBD、font/OS/library候補は方式比較、corpusはtest。grapheme/caret保証の採否を先に確認 |
| [ ] B24 | [BD-24](../12_design/BD-24_ime-edit-session-upper-design.md) 判断1～5・event表 | RQ-15-006、RQ-02-011、RQ-09-005に重複。補足候補: preedit非検証、commit一度、明示cancel非伝播、OS復元保証なし | session ID/snapshot/Adapter順序は設計。本文末のDD00未承認は当時の進行記録へ（X09） |
| [ ] B25 | [BD-25](../12_design/BD-25_async-validation-upper-design.md) 判断1～5・将来境界 | 同期短時間のみ、async pending非提供、VM側stale識別はRQ-15-008に主旨あり。pendingとIncompleteの区別を要求へ対応付ける | AsyncCommand/状態Propertyによる構成例は設計/利用例。将来Core async validation・cross-field等は採否待ち |
| [ ] B26 | [BD-26](../12_design/BD-26_async-command-progress-upper-design.md) 判断1～5・停止後保証 | 最新進捗/終端一度/古い結果破棄はRQ-06-004、RQ-15-009に重複。全sample/停止後UI反映非保証を確認 | progress slot/coalescing/wake/operation照合は設計。ログ機能を進捗契約から追加しない |
| [ ] B27 | [BD-27](../12_design/BD-27_error-object-upper-design.md) 判断1～5 | RQ-02-012、RQ-04-007、RQ-15-011に重複。補足候補: code意味再利用禁止、message変更可能、任意cause/diagnostic・snapshot不変 | domain/code表・handle/API/lifetime方式はDD/API。diagnostic機密情報/size/depthはsecure-design。表示文言要望と内部診断を分ける |
| [ ] B28 | [BD-28](../12_design/BD-28_abi-release-version-upper-design.md) 判断1～5 | version/ABI major/互換minorはRQ-15-012に主旨あり。補足候補: wrapper独立版とminimum runtime宣言、公開後互換保証 | symbol/discovery/metadata構造はAPI/配布設計。初回stable番号・pre-release運用は未決のrelease方針 |
| [ ] B29 | [BD-29](../12_design/BD-29_artifact-provenance-upper-design.md) 判断1～5 | immutable/署名/照合/失敗拒否はRQ-15-016、RQ-12-001～003に主旨あり。必要metadata項目・alias差替え禁止を分解候補にする | manifest field/schema、cache identityは設計、署名検証・trustはsecure-design。stable前具体化のgateはrelease計画 |
| [ ] B30 | [BD-30](../12_design/BD-30_artifact-distribution-upper-design.md) 判断1～5 | RQ-12-004～008、RQ-15-016に主旨あり。補足候補: 通常installの暗黙更新なし、explicit fetch、offline利用範囲 | catalog/resolver/lock/cache構成を設計、認証境界をsecure-designへ。URL等未決を要求未採用と混同しない |
| [ ] B31 | [BD-31](../12_design/BD-31_artifact-signing-trust-upper-design.md) 判断1～5 | 独立trust root・unknown/revoked拒否・認証rotation・offline bundleはRQ-15-016に主旨あり。失効/SLA/staleの要求境界は補足候補 | trust chain/鍵保管/失効配信・期限の実現はsecure-design。具体期限に要求根拠が必要かを確認してから数値化 |
| [ ] B32 | [BD-32](../12_design/BD-32_artifact-manifest-schema-upper-design.md) 判断1～5 | RQ-15-016のschema互換規則を分解候補。全意味内容署名、unknown critical拒否・optional許容・曖昧parse拒否の保証を確認 | serialization/wire fields/parserはAPI/secure/DD。runtime versionとschema versionの役割を設計に残す |
| [ ] B33 | [BD-33](../12_design/BD-33_collection-grid-upper-design.md) 判断1～6・Grid例 | RQ-02-005/006、RQ-09-019/020/022に主旨あり。補足候補: column ID、ID再利用禁止、編集対象整合。Grid/Table対象段階はX05 | independent model/change set/可視範囲+overscan/一時editorは設計。全cell恒久Binding非生成は性能実現方式として評価 |
| [ ] B34 | [BD-34](../12_design/BD-34_selection-model-upper-design.md) 判断1～6 | RQ-02-006、RQ-09-020に主旨あり。補足候補: 外部選択順、削除時後継を自動選択しない、Single無効更新全拒否、Collection差替え時ID非流用 | ID集合/current/collection identity構造は設計。gesture/navigation/mode変更APIは未決の機能採否とAPIに分離 |
| [ ] B35 | [BD-35](../12_design/BD-35_collection-change-batch-upper-design.md) 判断1～6・入力検証境界 | RQ-02-005、RQ-09-019、RQ-07-009に主旨あり。補足候補: 拒否時revision/selection/通知不変、成功時中間不整合非公開 | prepare/commit/reconcile・一revision/change通知は設計。Model整合性検証とフォーム業務validationを要求上も区別 |
| [ ] B36 | [BD-36](../12_design/BD-36_runtime-shutdown-dispatch-upper-design.md) BD-36-01～04・§3/5 | 同期/非同期/callback時のshutdown要求保証をRQ-02-013/06へ整理する候補（X03）。01/02/04確認済みと03未確認を区別 | priority control task/dispatch context/Adapter境界は設計。API別status・完了観測・loop停止時解除はAPI/要求境界に分ける |

## 4. BD番号のない基本設計全5文書

| 完了 / 項目 | 文書・対象箇所 | RQ照合・要求へ整理する候補 | 設計等に残すもの・具体to-do |
|---|---|---|---|
| [ ] S01 | [設計骨格](../12_design/design-overview.md) §2/3/7/8/9/10/11 | 対象言語、初期Control、platform範囲はRQ-01/03/09/11/14に重複。§9の旧保留を現行RQと区別 | §3～6の責務/依存構造はBD。§7/11の進行順は計画、§8の具体シナリオはtest。§10のtestへの根拠参照と役割を整理（X09） |
| [ ] S02 | [Runtime契約](../12_design/runtime-contract.md) R1～R4、§2～8 | stale検出、独立参照、UI専用、destroy一度/解除後開始なし、失敗非破壊はRQ-02/04/05/06に主旨あり。補足候補: token非再利用・枯渇失敗、停止後操作、callback種別例外 | handle表/token/公開参照と内部参照/キュー破棄は設計。R2とBD-36の差分を整理（X03）。§8具体試験はtestへ |
| [ ] S03 | [C ABI仕様](../12_design/abi-spec.md) A1～A32、§2～22 | 一つのAに外部保証とC宣言が混在。特にA4/8/10～12/14/16/19～28/31/32の所有・入力保持・失敗・状態の保証をRQ-04/05/06/07/09/15へ照合 | 型幅・symbol・関数署名・status/enum値・buffer形はDD内のAPI具体化、参照構造はBD/DD、検証節はtest。A7の将来OBJECT予約は採用しない。A11/14/19とDD-02差分はX01 |
| [ ] S04 | [MVVM意味論](../12_design/mvvm-semantics.md) P1～4/B1～4/C1～4/D1～4/V1～8、§2～16 | Property/通知順、Binding更新/競合/拒否/解除、Command終端、Accept/Incomplete/RejectはRQ-02/06/07/09/15に主旨あり。補足候補: 同値検証、購読順、入力保持/外部上書き、部分起動失敗境界 | queue/同期group/DAG/operation ID/error保持構造は設計。§3のF64未決や§8.3のrelease未決を後続承認と対応付け、古い保留を新要求にしない。具体確認例はtest |
| [ ] S05 | [言語Binding](../12_design/language-bindings.md) L1～L4、§2～8 | 4言語parity・Core正本はRQ-01/03、worker更新はRQ-06に重複。補足候補: 明示adapter・任意field自動発見非保証、明示cleanup/closure保持 | Core Property facade/adapter構造と寿命管理は設計。型名例はAPI例で正式名にしない。float等の旧未決とBD-01/DD-07、UI限定shutdownとBD-36を整合 |

## 5. DD全24文書

| 完了 / 項目 | 文書・対象箇所 | RQ照合・要求へ整理する候補 | 設計等に残すもの・具体to-do |
|---|---|---|---|
| [x] D00 | [DD-00](../13_detail_design/00-decision-map.md) §1～5 | 現行RQへの対応、BD-36判断別状態、承認境界と上流参照を反映。初期縦断case/要求test参照は除去 | DD割当・後続不足を[DD-00整合計画](dd00-current-rq-alignment.md)へ移管。DD-00 v0.2は2026-10-15承認 |
| [x] D01 | [DD-01](../13_detail_design/01-handle-registry.md) DD-HND-01～06、§1～6 | RQ/Runtime/ABIの根拠リンクを各記述へ配置。DD-01 v0.2でtoken/registry/検証方式をDDに整理し、レビューcaseと下流参照を除去。RQ/APIの外部保証候補は[別レビュー案](dd01-rq-alignment-review.md)へ移管（2026-10-16） | 既存caseはTC-CORE-OBJ-01、TC-LIFE-01、TC-ABI-REP-01。内部競合・枯渇のunit-testは実装計画で対応。数値status/codeはDD-06に保持 |
| [x] D02 | [DD-02](../13_detail_design/02-runtime-lifetime.md) DD-LIFE-01～08、§3～6 | 承認済みRQ-05/07とRuntime/BD根拠を対応付けた。enqueue失敗時token保持、shutdown冪等性、停止後release/UI拒否はRQ候補として[別レビュー](dd02-rq-alignment-review.md)へ分離。Detached API別動作はD11へ | DD-02 v0.4で承認済みownership/state/cleanup設計を保持し、状態矛盾、確認case、下流DD参照を整理（2026-10-16）。既存test caseはTC-LIFE-01、TC-OWN-01、TC-CALLBACK-01、TC-THREAD-01 |
| [ ] D03 | [DD-03](../13_detail_design/03-dispatcher.md) DD-DSP-01～08、§4/5/7 | RQ-06-001/002、RQ-15-007に主旨あり。補足候補: FIFO受付順、非再帰、一度実行またはdestroy、shutdown優先。loop停止時は未決（X03） | 一turn一件、wake coalescing、bridge、queue件数上限なしは設計選択。上限なしを製品の無制限保証にしない |
| [ ] D04 | [DD-04](../13_detail_design/04-callback-lifecycle.md) DD-CB-01～09、§4～7/9 | RQ-05-006～009、RQ-04-015～017に重複。補足候補: 通知例外後の他購読継続。marker/事前検知は候補、必須機能化しない（X07） | closure状態/in-flight safe pointはDD。marker構文はAPI、rule精度/言語差は解析設計。DD-24の計画だけ整理し本体へ進まない |
| [ ] D05 | [DD-05](../13_detail_design/05-boundary-values-errors.md) DD-VAL-01～07、§3～6 | RQ-04-001/003/006/007、RQ-05-004/005、RQ-15-011に主旨あり。補足候補: 埋込NUL、不正UTF8拒否、容量不足非部分書込、詳細OOMでも元status維持、失敗非破壊 | pointer/length/caller buffer、出力初期化値はAPI。copy/prepare/commitはDD。一般transactionへの拡大を防ぐ |
| [ ] D06 | [DD-06](../13_detail_design/06-abi-surface.md) DD-ABI-01～10、§3～8 | RQ-04-006/009/010、RQ-15-011/012に主旨あり。補足候補: 利用者code自動採番、未知status保持、feature/code非再利用、shutdown受付と完了の区別（X03） | symbol形式/数値status/domain範囲/UI_PENDING/bridge宣言はAPI。自動採番の安定catalog・生成時期は設計。完了待ちAPIを新設しない |
| [ ] D07 | [DD-07](../13_detail_design/07-property-notification.md) DD-PROP-01～10、§4～9/11 | RQ-02-002、RQ-07-005/006/008、RQ-15-007に主旨あり。五値型・比較/表現保持・初期通知なしを補足照合。§11文字列↔数値内製変換は将来意向 | kind ID/DECIMAL wire、128作業枠、generation/subscriber snapshotはDD/API。release仕様はX01。将来converterを初期要求にしない |
| [ ] D08 | [DD-08](../13_detail_design/08-binding-graph.md) DD-BIND-01～09、§4 | RQ-07-001～011、RQ-15-007に重複。補足候補: 同source重複拒否、同値初期同期、独立連鎖/確定値保護、診断観測 | DAG/同期group/因果ID/先行状態再現/graph世代/128sliceはDD。診断APIはAPI、fingerprint上限・衝突等は未決設計。未定義error catalogをDD-22が担当済みとしない |
| [ ] D09 | [DD-09](../13_detail_design/09-validation-input.md) DD-VALID-01～10、§5～7 | RQ-02-009/011、RQ-07-010、RQ-09-005、RQ-15-006/008に主旨あり。補足候補: Validator最大一つ/登録時非再検証/禁止再入、編集origin間一致、通常RejectとIME Reject差 | 型別signature/UTF8offset/input状態PropertyはAPI。closure管理/評価commitはDD。Accept後のTextBox自身Validator拒否は要求結果の未決として切り出す |
| [ ] D10 | [DD-10](../13_detail_design/10-async-command.md) DD-CMD-01～12、§3～9 | RQ-02-004、RQ-06-004、RQ-09-006、RQ-15-009に主旨あり。補足候補: execute成功とoperation失敗の区別、重複cancel、start部分起動、close/release差、状態の保持期間 | operation record/ID、cancel flag、readonly Property/遷移commitはDD/API。状態図を維持し、screen owner/停止後保証と要求差分を追跡 |
| [ ] D11 | [DD-11](../13_detail_design/11-ui-tree-lifetime.md) DD-TREE-01～08、§4/5/7 | RQ-02-007/013、RQ-09-001/003に主旨あり。補足候補: detach後Property/Binding継続、subtree維持、atomic reparent非提供、追加失敗でDetached維持 | Logical強参照/Visualowner、prepare/commit、shutdown cleanupはDD。公開child操作形はAPI。Detached許可操作とpeer復旧失敗の利用者結果を確認 |
| [ ] D12 | [DD-12](../13_detail_design/12-layout.md) DD-LAYOUT-01～08、§4/7 | RQ-02-008、RQ-09-007/008に主旨あり。方向/Align/既定値・寸法制約・独自拡張除外の位置づけをX06で確認 | Measure/Arrange集約、dirty伝播、数値表現は設計。128等と同様に既定値を機械的に要求化しない。単位/overflow/clippingの外部保証と方式を分ける |
| [ ] D13 | [DD-13](../13_detail_design/13-input-focus.md) DD-INPUT-01～10、§3/8/10 | RQ-02-009/010、RQ-09-009/010に主旨あり。補足候補: Focus eligibility/復帰/自動代替なし、FocusLost→Gained、入力未buffer、route範囲 | 正規化/順序/feedback抑制は設計、bubble/tunnelや公開eventはAPI/設計境界を確認。Tab/ShiftTab/Ctrl+Z等のkey割当は将来メモのまま維持 |
| [ ] D14 | [DD-14](../13_detail_design/14-textbox-ime.md) DD-TEXT-01～12、§5/7/8 | RQ-09-005、RQ-15-006に主旨あり。補足候補: cancel時input reason、外部値で編集上書き。DD-TEXT-09/10の拒否/退避失敗時結果は未決要求境界 | editing transaction/session snapshot/position変換層はDD/API。DD-TEXT-11の内部単位は設計待ち、grapheme操作保証は要求待ちとして分ける。選択肢承認を採用と誤記しない |
| [ ] D15 | [DD-15](../13_detail_design/15-controls-v1.md) DD-CONTROL-01～12、§4/7/9 | RQ-09-001～011、RQ-10に主旨あり。補足候補: 空nameとfallback非提供、effective enabled、Focusable既定、visibility/直接Click非提供、CommandなしButton、共有Command寿命（X06） | abstract分類/typed Command参照/activation一本化は設計、Property名/関連設定はAPI。未決欄と承認表の粒度を揃える |
| [ ] D16 | [DD-16](../13_detail_design/16-platform-rendering.md) DD-PLAT-01～10、§3～5/7/9 | RQ-11-008～013、RQ-15-002/003/006に主旨あり。補足候補: peer喪失でもCore状態保持、失敗frame非成功、closeでRuntimeを暗黙shutdownしない | snapshot/frame dirty/peer lifetime/teardownはDD。accept/veto/defer・再open・複数backendは採否待ち、具体graphics/OS/scaleは設計待ち |
| [ ] D17 | [DD-17](../13_detail_design/17-accessibility.md) DD-ACC-01～10、§4～8/10 | RQ-10-001～009、RQ-15-018に主旨あり。補足候補: node identity安定、stale query/action、AT queryがkeyboard focusを変えない、空name非合成、OS参照によるCore延命なし | snapshot/provider/thread-safe action/通知集約はDD。UIA/AT-SPIは方式、bus必須範囲/認証/AT製品supportは要求待ち、具体OSprofileは設計/test |
| [ ] D18 | [DD-18](../13_detail_design/18-wrapper-cpp.md) DD-CPP-01～15、§3～9 | RQ-03-001、RQ-04-016、RQ-05に主旨あり。補足候補: 例外無効build、利用者exception handler、destructor失敗でもtoken消失なし、明示shutdown保証 | move-only/borrowed/RAII/trampoline/事前node/cleanup registryはDD。公開result/handler型・最低標準はAPI/support境界。API章保留を維持（X10） |
| [ ] D19 | [DD-19](../13_detail_design/19-wrapper-csharp.md) DD-CS-01～14、§3～10 | RQ-03-002、RQ-05/06、RQ-12-006に主旨あり。補足候補: Dispose冪等/競合時安全、handler/非throw結果、失敗token保持、reflection非提供 | ulong/lease/static trampoline/GC root/ticketはDD。SafeHandle採否/interop/TFM/API名は未決。C++の要求をC#へ自動継承していないか照合 |
| [ ] D20 | [DD-20](../13_detail_design/20-wrapper-rust.md) DD-RS-01～15、§3～10 | RQ-03-003、RQ-04-015、RQ-05/06、RQ-12-005に主旨あり。補足候補: safe層保証、Drop token保全、panic=abort/二重panicの継続cleanup保証外 | owned/borrowedフロー、Send/Sync marker、Send+'static、Box/trampoline/cleanup ticketはDD/API。MSRV/edition/supportと内部表現を分け、所有権図を維持 |
| [ ] D21 | [DD-21](../13_detail_design/21-wrapper-python.md) DD-PY-01～21、§4.1/6/8/11/12 | RQ-03-004、RQ-04-017、RQ-05/06、RQ-15-013に主旨あり。補足候補: close/context、cleanup token保全、base-chain-up検知、通常終了/一般例外網羅と強制終了除外（X04/07） | ctypes型/rootregistry/lock/read-only snapshot/coordinator/hookはDD。GIL非依存を全API任意thread化としない。free-threaded正式supportは未決、guard対象method一覧はAPI/解析 |
| [ ] D22 | [DD-22](../13_detail_design/22-build-distribution.md) DD-BUILD-01～11、§3～10/11 | RQ-12、RQ-15-015～017に重複。補足候補: 同一binary identity、検証状態三段階、明示fetch、target非fallback、schema拒否条件 | profile/manifestwire/catalog/cacheは配布/DD、JCS/Ed25519/SHA-256・trustchainはsecure-design（X08）。license/EOL/gateは要求・運用・testに分ける |
| [x] D23 | 旧DD-23案を[Requirements Test Manifest](../30_release_test/requirements-test-manifest.md)と既存のtest担当文書へ再分類 | 要求case/evidenceの共通契約はmanifest、RQ-case mappingは要求test対応表、release/version gateは各規約、実装単位testはunit-test方針へ分離 | DDとしての具体設計判断がないため詳細設計フェーズから除外。旧原案は削除済み。manifest文書版0.2・共通ルールは2026-10-14利用者承認済み |

## 6. 全文書に共通する整理項目

- [x] **要求根拠の明示**: 各設計判断から該当RQの条件へ対応を付ける。章全体へのリンクだけで保証追加を隠さない。対応なしは要求追加候補・設計詳細・調査候補のどれかを記す。
- [x] **IDの意味単位**: 一つの旧IDを、要求条件・方式・公開宣言・未決候補に分けて台帳へ記録する。新IDが必要な箇所だけ発番する。
- [x] **承認の文単位化**: 「本書承認済み」でも本文の選択肢・候補・未決は未決。特にBD-36、DD-04/14/15/17/21/22/23の採用範囲を維持する。
- [x] **履歴表**: BD/DDの単発承認文を削除せず、当時の意味・日付・承認者を保存して承認履歴/改訂履歴の表を付ける。設計上の承認を要求上の承認日として転記しない。
- [x] **参照の向き**: 要求からBD/DD/API/secure/testへの根拠リンクを作らない。BDからDDへの引継ぎ一覧等は仕様根拠ではなく計画側へまとめる。移行台帳は旧新の対応を管理する補助成果物とする。
- [x] **検証例の正本**: BD/DDの検証節は原則test候補として移すかtest根拠の説明へ縮める。設計を理解するための計算例・状態図・所有図は設計に残せる。要求から具体テスト手順を導く段階を維持する。
- [x] **未決の担当**: 「DD-22/23で決める」等の一括委任を、要求採否・API・secure-design・詳細設計・運用・testごとに分ける。予定された担当が実際には決めていない項目を完了扱いにしない。

## 7. 承認履歴

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-10-12 | 全BD/DD走査・対象リストとto-do作成 | 利用者 | 今回の依頼による作業範囲。個々の移管・要求追加・ID変更の承認ではない |
| — | 0.1の整理候補・後続実施順 | — | レビュー待ち |

## 8. 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 0.1 | 2026-10-12 | BD37件・基本設計5件・DD24件の対象箇所、RQ照合、移管候補、横断課題を記録 | Codex | 調査・to-do作成指示に基づく。仕様変更は未実施 |
| 0.2 | 2026-10-12 | RQ/BD/DD/SDの分類を追記し、独立API設計の前提をDD内の具体化に更新 | Codex | 今回の役割指定に対応。対象66文書の仕様本文は未改訂 |
| 0.3 | 2026-10-12 | designとbasic-designを同一BDとする分類上の呼称を明記 | Codex | 今回の統一指示に対応。個別移管・IDは変更なし |
| 0.4 | 2026-10-12 | 外部動作・保証・例外をRQ、BD/DDをflow・ownership・state・interface等の実現手段として分類 | Codex | 今回の利用者指示に対応。台帳各項目の個別移管は未実施 |
| 0.5 | 2026-10-17 | 全66項目とX01～10の一括レビュー入口、作成済みと適用済みの区別を追加 | Codex | 案作成の指示。残件の正本適用は未承認 |

## 2026-10-17 適用結果

全66台帳項目の分離を適用した。D23は要求testへの既存移管を維持し、廃止文書を再作成していない。

### 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全レビュー案の反映 | 利用者 | 「すべて反映して下さい。」により採用・正本適用を承認 |

### 改訂履歴

| 版 | 日付 | 内容 |
| --- | --- | --- |
| 0.6 | 2026-10-17 | 承認と正本適用結果を追記。作成時点の比較・引用を保持 |
