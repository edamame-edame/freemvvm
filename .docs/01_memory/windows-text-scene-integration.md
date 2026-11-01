# カテゴリ: Platform / Windows TextServiceからScene・Window rendererへの接続

- CoreのTextService seam、text control snapshot、same-revision Layout/Scene経路は`UNIT-CORE-084/085/086/087`で実装済み。
- `UNIT-CORE-089` adds optional `PlatformWindowPeer::text_service()`, stores that service with `WindowLifecycle`, and uses it before the Runtime-wide fallback. The per-Window fake test verifies two peers do not mix requests, Runtime fallback is used only when no peer service exists, and failure retains the previous complete Layout/dirty state. Local Core/workspace tests, Clippy, Rust 1.85 check, formatting and document links passed; GitHub CI is pending.
- Windows DirectWrite shapingとfont catalogの個別接続部品は`UNIT-PLATFORM-WINDOWS-004`で実装済みだが、Windows factoryからRuntimeへのTextService供給と、Core text requestから該当Window rendererのface catalogまでのcall flowは別実装範囲。
- Windows TextServiceは`WindowsPeer`からpeer state共有のendpointを返し、Core requestを同期で同peerのbackendへ送る。Win32 backendは`ResolveText` commandをWindow workerへdispatchし、renderer内のDirectWrite `FontResources`がsystem message fontを14 DIPでshapeする。
- DirectWrite callbackは実際に選択されたfont faceのfamily/style/indexとfile digestを記録し、face COM objectを同じrenderer catalogへ保持する。Coreへ返すのはmetricsと`ResolvedGlyphRun`の値であり、COM objectはworker外へ渡さない。
- close後またはworker/renderer unavailable時のresolveはfailureにする。UI smokeはheadless fake testと区別し、0.0.1 release qualificationで確認する。
- `UNIT-PLATFORM-WINDOWS-005`のfocused 2 tests、Windows Platform 17 passed/2 ignored、workspace all-targets tests、workspace Clippy、Rust 1.85 check、format、doc links、units manifest parseがlocal pass。Windows GUI profile smokeと同revision CIは別確認事項。
- 根拠文書: `.docs/20_todo/windows-text-scene-producer-integration.md`、`.docs/13_detail_design/platform/internal-layers.md`、`.docs/13_detail_design/ui/controls.md`、`.docs/20_todo/windows-text-service-directwrite-integration.md`。
- fake-service、Windows target build、headless DirectWrite smokeはWindows GUIでの可視表示・実機font profile確認を意味しない。
