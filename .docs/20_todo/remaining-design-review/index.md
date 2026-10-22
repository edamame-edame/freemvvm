# 未設計事項 / 全件レビュー案

- 文書版: 0.2
- 状態: 全200判断承認済み・正本適用済み（2026-10-17）
- 作業計画: [一括作成計画](../remaining-design-batch-review.md)
- 元課題: [分離後の後続課題](../requirements-design-separation-follow-up.md)

## 全体

20分野・169判断と、要求変更31件をまとめた。推奨/代替、interface/class、所有・call/data flow・状態遷移を記載した。利用者の全件反映指示を受け、要求・基本設計・詳細設計・セキュア設計・test契約へ適用した。実装・実行結果は未証明のまま。

| 入口 | 内容 |
| --- | --- |
| [承認台帳](approval-register.md) | 169方式/範囲判断 + 31要求変更。すべて承認済み・適用済み |
| [要求へ戻す31件](rq-changes.md) | 利用者可視の結果・制限・minimum・offline条件を独立にレビュー |
| [元課題の全件対応](scope-inventory.md) | 現行文書と移管記録の394箇所を分類し各分野へ対応付け |
| [実環境の採取値](external-inputs.md) | 方式案と、未取得SDK/鍵/board/実行証跡の違い |

## 分野別レビュー

| 分野 | 案 | 判断数 | 主な内容 |
| --- | --- | --- | --- |
| 01 | [F32の提供経路・数値変換](01-value.md) | 7 | F32は型別変換関数の入力/出力で提供し、初期Property kindを増やさない。float→doubleは全有限値を正確に拡張する。 |
| 02 | [Collection・Selectionの所有、batch、snapshot](02-collection.md) | 9 | CollectionModelをschema+ordered RowStore+item index+revisionで構成。schemaは作成時固定で列ごとに既存五kindを指定する。 |
| 03 | [Host境界・shutdown完了観測・cleanup drain](03-runtime.md) | 9 | 既存runtime_shutdownは同期指定の互換入口として保持し、新symbol runtime_request_shutdown(mode,out_request)を追加する。 |
| 04 | [通知・非収束・Command・エラーcatalog](04-event.md) | 10 | 循環履歴はchainごと256 snapshot、全体8 MiBを初期内部budgetとする。到達時はUnknownへ移し、循環と断定しない。 |
| 05 | [編集API・Unicode位置・IME回復](05-textbox.md) | 9 | Core内部の正本位置はUTF8 byte offset、境界はUnicode scalar boundary。OS UTF16/codepoint位置はTextIndexMapで変換する。 |
| 06 | [Layout・座標・Input route・Tree所属](06-geometry.md) | 9 | Layoutはf64 DIP、root client左上原点、x右/y下。1 DIP=1/96 inch相当の論理量とし、scale_x/yを環境値で保持する。 |
| 07 | [Control API・Window close・描画backend](07-platform.md) | 9 | 公開名はWindow/StackPanel/Text/TextBox/Button、Widget/Containerは抽象view。名前は4言語で対応させC ABIはsnake_caseにする。 |
| 08 | [Text engine・IME OS接続・Accessibility](08-access.md) | 9 | Windows textはDirectWrite、LinuxはFontconfig+HarfBuzz+FreeType、配布fontはNoto Sans/Noto Sans CJK JPの許諾確認済みsubset候補。 |
| 09 | [C++公開型・handler・cleanup同期](09-cpp.md) | 7 | C++17をminimumとし、freemvvm namespaceのOwned<T>/Borrowed<T>/Result<T>/UiError/Decimal64を公開する。 |
| 10 | [C# token owner・interop・Dispose・load](10-csharp.md) | 7 | minimum TFMはnet10.0、language C#14、core facadeはunsafeを内部へ隠す案。C#利用者向けminimumを要求変更として明記する。 |
| 11 | [Rust公開型・receiver・registry・panic](11-rust.md) | 7 | crate名はfreemvvm-sys / freemvvm、edition2024、MSRV1.85.0。最低版testと固定release toolchainを別jobで持つ。 |
| 12 | [Python API・通常終了handshake・特殊method](12-python.md) | 10 | minimum CPython3.13、通常GIL版を正式profile候補、3.14tを追加検証profileとする。PyPy/subinterpreterは初期正式support外。 |
| 13 | [callback識別・禁止検知・lint/MCP](13-analyzer.md) | 8 | Analyzer IRはCallbackKind/CallTarget/RuntimeAlias/Effect/SourceSpanを持つ。禁止効果、警告効果、Unknownを分けてreportする。 |
| 14 | [署名envelope・schema・parser](14-manifest.md) | 9 | schema major1のUTF8 JSON envelope {payload,signatures}。payloadをJCS化しdomain separatorを先頭に付けPure Ed25519で署名する。 |
| 15 | [鍵管理・失効・offline・staging](15-trust.md) | 9 | root/targets/revocationの役割を分ける。rootは2-of-3 offline鍵、targetsは1-of-2 release鍵をKMS/HSM、失効metadataはroot認証roleで発行する。 |
| 16 | [最低版・target matrix・toolchain lock・保守](16-build.md) | 9 | 初期正式profile候補はWindows11 x64、Debian12 glibc x64。ARMv7/AArch64は両方cross-build、代表実機はRaspberry Pi4 ModelB 4GBのAArch64を選ぶ。 |
| 17 | [package名・loader・artifact取得・catalog](17-package.md) | 8 | native filenameはfreemvvm_abi1.dll / libfreemvvm_abi1.so。package ID案はPython freemvvm、native freemvvm-runtime-<profile>、NuGet FreeMvvm、CMake FreeMvvm、crate freemvvm/freemvvm-sys。 |
| 18 | [実装unit台帳・test ID・CI・scenario](18-test.md) | 9 | 単位IDはUNIT-<domain>-NNN、実testはUT-/REQT-/ABI-/WRAP-/INT-/SYS-/PLAT-/PKG-/REL-/VER-を境界に合わせる。論理TCと別fieldにする。 |
| 19 | [次段階UI・将来候補の範囲と拡張点](19-scope.md) | 8 | 次段階Controlは既存採用リストを保持し、CheckBox/ToggleButtonはBOOL Property、ProgressBarは進捗値、ScrollViewerはviewportの独立modelへ接続する案。 |
| 20 | [共通API表・feature・status優先順・互換移行](20-api.md) | 7 | DD内API表を共通schemaで統合し、入力/出力、owned/borrowed、thread/context、commit、failure ownership、feature/versionを必須fieldにする。 |

## 今回要求側で承認した変更

- Python通常cleanupの協調条件：有限に戻らない利用者callbackや、cleanup前に終了する外部UI hostまで無条件に回収する保証は実現できない。管理境界と除外条件を要求案へ明記した。
- offline期限：7日revocation候補は長期airgapへ影響する。長期offline profileという代替と一緒に判断する。
- Container Focus、初期text表示、OS同期action待機、minimum/target profileは外部範囲への影響を要求変更として記録し、独立した要求IDで承認した。
- keybinding/UndoRedo、custom/nullable/OBJECT、Grid/Table等の未採用範囲をmandatoryへ追加しない。次段階採用済みUIの具体案と区別する。

## 適用結果

[全200件の正本適用表](../remaining-design-application-result.md)から正式IDと本文を開ける。[詳細設計索引](../../13_detail_design/detailed-design-specification.md)と[セキュア設計索引](../../14_secure_design/00-security-design-map.md)を更新した。

## レビュー時の適用順案

要求変更の採否 → 共通API/status/host境界 → 値/Collection/TextBox/UI → 4wrapper → SD/配布profile → unit/test inventoryの順で差分を反映する。実現案の承認だけで関連する要求変更を承認した扱いにしない。判断IDごとの部分承認も可能。

## レビュー案作成時の検査

25文書・90表・16個のMermaidコードブロック・2089リンクを静的検査し、リンク先/anchor、表の列数、ID重複、200件の承認台帳対応にエラーはなかった。元課題394箇所を照合し、正本98ファイルのSHA-256が作業開始時と一致することを確認した。Mermaidの描画検証、製品コード/CI/testの実装・実行は行っていない。

## 承認履歴

| 日付 | 範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 全未設計レビュー案の作成 | 利用者 | 一括生成を指示。採用・正本反映・実装は未承認 |

## 改訂履歴

| 版 | 日付 | 内容 | 作成者 |
| --- | --- | --- | --- |
| 0.1 | 2026-10-17 | 未設計事項の一括レビュー案を作成 | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.2 / 全案採用と正本への入口 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.2 | 2026-10-17 | 全案採用と正本への入口を正本へ適用し、旧判断との対応を保持 | Codex |
