# カテゴリ: Platform / Windows TextServiceからScene・Window rendererへの接続

- CoreのTextService seam、text control snapshot、same-revision Layout/Scene経路は`UNIT-CORE-084/085/086/087`で実装済み。
- `UNIT-CORE-089` adds optional `PlatformWindowPeer::text_service()`, stores that service with `WindowLifecycle`, and uses it before the Runtime-wide fallback. The per-Window fake test verifies two peers do not mix requests, Runtime fallback is used only when no peer service exists, and failure retains the previous complete Layout/dirty state. Local Core/workspace tests, Clippy, Rust 1.85 check, formatting and document links passed; GitHub CI is pending.
- Windows DirectWrite shapingとfont catalogの個別接続部品は`UNIT-PLATFORM-WINDOWS-004`で実装済みだが、Windows factoryからRuntimeへのTextService供給と、Core text requestから該当Window rendererのface catalogまでのcall flowは別実装範囲。
- Windows TextService/renderer間のCOM object、thread affinity、font identity/catalog lifetimeは承認済みDDと照合してから接続方式を決める。実装都合だけでcross-thread COM共有やmarshalingを追加しない。
- 現在のCoreはRuntime-wide TextServiceをUI threadから同期呼出しするが、Windows DirectWrite resource/rendererはWindow worker所有。face registrationとshapingも同じworkerで行う既存前提があるため、per-Window TextService sessionをDDでレビューするまで接続実装に進まない。
- 根拠文書: `.docs/20_todo/windows-text-scene-producer-integration.md`、`.docs/13_detail_design/platform/internal-layers.md`、`.docs/13_detail_design/ui/controls.md`、`.docs/20_todo/windows-text-service-directwrite-integration.md`。
- fake-service、Windows target build、headless DirectWrite smokeはWindows GUIでの可視表示・実機font profile確認を意味しない。
