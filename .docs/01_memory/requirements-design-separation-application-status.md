# 要求・設計分離 / 2026-10-17全案適用の引継ぎ

- 区分: requirements-design-separation / application / current-status
- 利用者指示: 全レビュー案を反映

- 正本の要求は249 ID。既存165を保持し84を追加。RQ-15の22旧IDは機能章へ統合し再利用しない。RQ-08も復活しない。
- 文書別62案を全適用。D00/01/02の補足要求も正式化。65既存BD/DDを更新し、ABI宣言分冊を追加。
- 要求の受入確認はrequirements-test-assertionsへ、設計の試験列挙はdesign-test-case-transferへ移した。90 logical caseと249 RQを対応付け、実行合格とは区別。
- TextBox自身Property拒否は案B、IME Cancelは開始state/reasonも復元。CommandなしButtonは表示・no-op。公開既定/範囲をRQへ明記。
- shutdownはhost stopped通知とUI contextを使う待機解除方針。callback同期は受付後return、worker同期は完了またはUI実行不可の確定でreturn。停止通知をcleanup完了へ換算しない。
- Pythonは登録したmain/host境界で通常終了を集約。個別owner cleanupとRuntime shutdown coordinatorを分ける。GIL非依存は明示同期/readonly/copyを意味し、全API任意thread化ではない。
- SD-00～05に責務と防御範囲を配置。JCS/Ed25519/SHA-256の承認を維持。envelope/schema/鍵保管/期限等は未設計。
- F32利用API、Collection/Selection専用DD、TextBox回復・内部位置・通知順、言語API/最低版、marker拡張は後続課題。全案適用をこれらの完成へ換算しない。
- 適用前のBD/DD本文はto-do配下の履歴記録へ保存。現行の根拠はRQ→BD→DD/SDの依存を逆に参照する。

- 適用結果: [正本適用結果](../20_todo/requirements-design-separation-application-result.md)
- 後続: [未設計事項](../20_todo/requirements-design-separation-follow-up.md)

- 追加照合: 旧A20のValidator内shutdown禁止は、RQ-05の終了要求受付例外として整合。受付record登録は現在callbackを中断せず、cleanupのみ後続safe pointで実行する。DD-06宣言のget_versionは3要素、UI_PENDING=14へ整合済み。
- 設計IDの対応は331判断。D1～D4の縮約group/DAG/prepare/commit/cancel方式も保持した。
