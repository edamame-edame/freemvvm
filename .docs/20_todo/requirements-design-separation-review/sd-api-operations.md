# SEP-06 / DD内API・SD・運用の整理レビュー案

- 文書版: 0.2
- 状態: 全案反映承認済み・正本適用済み（2026-10-17）。具体化未了は適用結果を参照
- 作成日: 2026-10-17
- 上位計画: [分離計画](../requirements-design-separation.md)、[SD段階計画](../secure-design-phases.md)
- 入口: [一括レビュー索引](index.md)

## DD内APIの構成案

SEP-API-01: C ABIの宣言・symbol/型/enum/status catalogはDD-06を中心とするABI分冊、Property/Binding/入力/Commandの操作は各機能DD、言語class/trait/methodはDD-18～21へ収録する。abi-spec.mdは基本境界・層責務と旧A ID対応を残す。独立したAPI設計工程は作らず、計画済みDD整理後に章間のAPI表を統合する。

| 対象 | DDで具体化する操作とデータ | 保持する所有・状態境界 | 未決を確定しない項目 |
|---|---|---|---|
| Handle/Runtime | create/retain/release、post、bridge、shutdownの宣言 | owned/borrowed token、closure成功handoff、受付/完了 | stopped loop解除と新規completion観測API |
| Property/Binding | 五型create/read/write、subscribe/cancel、Binding create/cancel/error | copy/commit、generation、active registry | F32利用API、WatchDog公開設定の適用時点 |
| Validator/TextBox | typed callback、編集値/選択/input reason取得 | candidate借用、理由handle消費、editing transaction | 自身Property拒否、cancel reason、OOM退避、位置単位 |
| Command | execute/cancel/complete/close/state | operation一回終端、worker別owner | 部分start失敗、operation枯渇code、screen owner |
| Tree/Layout/Control | child add/remove、size/Align、typed Command参照 | parent強/child弱、bounds commit | 再所属peer失敗、visibility、未接続Buttonの結果 |
| C++ | move owner/borrow view、result、handler | noexcept destructor、pending node | 最低標準、公開名/overload、handler交換同期 |
| C# | IDisposable owner、call lease、typed result | static callback root、cleanup ticket | SafeHandle最終採否、TFM、interop generator |
| Rust | Owned/Borrowed/Dispatcher、Result | lifetime、Send/Sync境界、Drop ticket | receiver形、crate名、MSRV、poison recovery |
| Python | owner/with/close、ctypes prototype、host registration | 明示lock、root/ticket、coordinator | base-call必須method、handler、host終了通知の実装 |

各公開操作の表には入力/出力、owned/borrowed、thread/context、成功commit点、失敗時所有、適用RQ/BDを持たせる。これは表の構成案であり、未定義methodを実装済みAPIとして公開する指示ではない。

## SD段階への分離案

既存SD-00～05の段階を保ち、今回の分離範囲を次のレビュー判断にする。正式なSD本文の完成・対策の実装完了は今回記録しない。

| レビューID / 既存段階 | 対象と具体的な案 | 上流条件・残る選択 |
|---|---|---|
| SEP-SD-00 / SD-00 | 資産をnative bytes、manifest、trust metadata、ABI入力、callback root、解析workspace/cache、diagnostic/evidenceに分ける | RQ-04/05/06/12、artifact要求候補の採否を識別 |
| SEP-SD-01 / SD-01 | 外部artifact→parser、catalog→trust、ABI caller→Core、analyzer request→workspace、OS query→UI snapshotを信頼境界とする案 | 同一processの任意pointerを安全にdereferenceできるという保証を追加しない |
| SEP-SD-02 / SD-02 | 改ざん/鍵差替え/失効無視/target混同/曖昧parse/資源枯渇/所有の競合/秘密情報混入を要求に対応付ける | 新たな外部拒否条件・size limitの利用者保証はRQへ戻す |
| SEP-SD-03 / SD-03 | JCS/Ed25519/SHA-256の既承認提案範囲を保持し、parser構文検査→一意payload→署名とtrust→schema/target/ABI/digest判定→staging原子公開へ整理 | signature envelope、parameters、重複/Unicode処理、parser上限、trust expiry/revocation期限は未決 |
| SEP-SD-04 / SD-04 | tamper/unknown key/revoked/stale bundle/duplicate field/unknown critical/cache差替えの反例をTC-ARTIFACTへ、局所parser/registry不変条件をUTへ分ける | 要求候補未採用の期待値を必須passにしない |
| SEP-SD-05 / SD-05 | DDのtoken/closure/root/snapshot所有と、SDの入力検査/信頼判断/資源制限をinterface単位で突合する | 対策と機能要求が矛盾したらRQ-16-004の差異記録と変更承認へ |

ABI入力は値検査・length/shape検査・copy-before-commitを防御責務として追跡する。callback/rootの新規entry停止とin-flight drainはDDが所有し、SDは信頼境界での悪用/資源枯渇条件を検討する。診断とevidenceの秘匿項目・size/depth/保管権限はSD/運用へ分け、messageの存在を秘密保持保証と読み替えない。

## build/運用へ移す内容

SEP-OPS-01: BD-21のlicense選択経緯は履歴、条件はRQ、収録はdistribution DD、inventory/noticeの検査はrelease-testへ分ける。

SEP-OPS-02: BD-22の期間・例外・EOL公開物アクセスはRQ、ABI major×profile状態表の更新、告知担当と日付は運用記録へ分ける。SLA/LTSを今回追加しない。

SEP-OPS-03: 開発target順・DD割当・将来Control/marker/keybinding候補はto-doへ。正式supportはRQとprofile証拠、build job/実機suiteはtest/CI計画へ分ける。secret保存先、registry URL、鍵保管とretentionは具体化待ちを保持する。

## 具体化を開始する条件

RQ採否と上流制約が確定した項目から、DD内のclass/method/API表とSDの防御interfaceを具体化する。未決一覧を作ることと新しいAPI機能を採用することを分け、Collection/Selection/DD-24等の追加詳細設計は別着手として追跡する。

## 承認履歴

| 日付 | 対象 | 承認者 | 結果 |
|---|---|---|---|
| 2026-10-17 | SEP-06 / API・SD・運用分離 | — | 一括作成指示に基づくレビュー案。採用・正本適用は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 状態 |
|---|---|---|---|
| 0.1 | 2026-10-17 | 現行文書から責務分離・改訂文案・影響を作成 | レビュー待ち |

## 2026-10-17 適用結果

正本は [適用結果](../requirements-design-separation-application-result.md) と [正式要求ID移行](../../11_requirements/requirements-separation-id-migration.md) に記録した。以下の比較・引用・選択肢は作成時点の案として保持する。採用した選択は推奨案、未提示の実装詳細は後続課題として扱った。

### 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全レビュー案の反映 | 利用者 | 「すべて反映して下さい。」により採用・正本適用を承認 |

### 改訂履歴

| 版 | 日付 | 内容 |
| --- | --- | --- |
| 0.2 | 2026-10-17 | 承認と正本適用結果を追記。作成時点の比較・引用を保持 |
