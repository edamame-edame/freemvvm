# MVVM UIライブラリ 初期描画方式と検証順序に関する上位判断案

- 状態: BD-12承認済み（2026-09-26）
- 対応: 要求仕様第8・12節の初期Control描画方式と描画検証の開始順
- 根拠: Accepted ADR-0001、要求仕様第5・6・8・10節、承認済みBD-05～BD-11。DD-00は未承認

## BD-12 承認された判断

**初期Controlの見た目は共通Rendererによる独自描画とし、OSのnative controlを各Controlの意味・状態のsource-of-truthにしない。CoreのVisual Tree／Layoutが共通boundsと描画に必要な情報を持ち、RendererがPlatformの表示面へ描く。Platform AdapterはWindow／surface、入力、IME、Accessibility等のOS接続を担当する。対応・検証はWindows、Linux x64、組み込みLinux ARMv7/AArch64、その他の順に進める。Linux x64で整えた共通ソース・Renderer・build/test資産を使い、ARM向けartifactはLinux x64環境からクロスコンパイルする。Linux backendはADR-0001の候補順どおりDRM/KMS + EGL/OpenGL ESを第一候補とする。**

| 項目 | 承認された上位契約 | 根拠・境界 |
|---|---|---|
| Control描画 | Window、StackPanel、TextBox、Text、Buttonは共通Rendererで描く。Platform native controlは見た目やControl stateのsource-of-truthにしない。 | 共通Control意味論、Core Layout、hit-testとbackend間の一貫性。OSのIME session／候補UIはBD-10どおりPlatformが担う。 |
| Core／Renderer境界 | Core Visual TreeとLayoutが意味上の要素・共通boundsを供給し、Rendererが描画する。Platformごとのbackend差し替えを許す。Renderer API、描画command/scene、damage領域は後続設計。 | ADR-0001はRendererをCoreから分離しsubsystem差し替えを要求。BD-05／BD-06がVisual TreeとCore Layoutの責務を定める。 |
| Platform Adapter | Window/surface生成、display/GPU接続、raw input、IME、Accessibility等をOSと接続する。OSごとに異なる装飾・native peerの扱いはこの境界内で整理する。 | ADR-0001、BD-07、BD-10。native windowingを使うこととnative controlを使うことは別の選択。 |
| 対応・検証の優先順 | Windows → Linux x64 → 組み込みLinux ARMv7/AArch64 → その他。Windows開発環境で最初の起動・描画・入力を確認し、Linux x64でLinux backendとcross-build flowを確立してからembedded boardへ進む。 | Windowsから始める利用者方針。Linux x64はADR-0001のtarget候補、組み込みLinuxは正式対象。公開リリース順・正式CI gateは別判断。 |
| Linux x64資産の活用 | Linux x64と組み込みLinuxでCore、共通Renderer、C ABI、可能な限りplatform backend source・build/test scriptを共有する。Linux x64をhostにしてARMv7/AArch64向けnative artifactをcross-compileし、target別のlinker・sysroot・native libraryを設定する。x64 binaryをARMへ流用しない。 | ADR-0001はtarget triple別artifact、Yocto SDK、sysroot、linker等の固定を規定。 |
| 組み込みLinux実機検証 | Linux x64での動作・cross-build確認後、正式対象であるLinux ARMv7/AArch64の代表boardで実機検証する。ADR-0001記載の候補順でDRM/KMS + EGL/OpenGL ES、Wayland、X11、framebuffer/software rendererを確認し、要求仕様の性能項目を計測する。 | 組み込みLinuxは正式対象。backend候補順と実機benchmarkはADR-0001／要求仕様に記載済み。 |
| その他のOS | 上記の後にWindows以外のarchitecture、macOS等へ展開する。対象候補は維持し、正確なtarget matrix・各OS backend・リリースgateは配布／CI判断に残す。 | ADR-0001と要求仕様第8～9節は候補targetを記載。 |

## 推奨理由と代替案

- **独自描画を推奨する理由:** CoreがControl、Layout、Focus、Inputの意味を共通管理しているため、OS native controlの個別挙動へ依存させない方が、4言語wrapperと複数OSで同じControl契約を保ちやすい。独自描画なら共通Visual TreeとCore boundsをRendererへそのままつなげる。
- **native controlを主方式としない理由:** 各OSの見た目や操作感へ自然に馴染む一方、Controlごとの状態・描画・IME接続をnative APIへ写す差が大きく、Control意味論の共通化と組み込みLinux backend検証が複雑になる可能性がある。
- **native OS接続は引き続き使う:** 独自描画はOS APIを排除する判断ではない。Window/surface、graphics context、raw input、IME candidate UI、Accessibility bridgeはPlatform Adapterが接続する。
- **性能を仕様だけで断定しない:** 描画FPS、memory、startup、GPU synchronization等を代表boardで計測する要求は維持する。必要な最適化はRenderer APIを具体化する段階で実測に基づいて選ぶ。
- **Windowsから始める理由:** 開発環境で起動・描画を早期に確認して反復しやすくする。これは最初の対応順であり、公開サポートや正式リリースのgateは後で定める。
- **Linux x64からARMへ進む理由:** 共通コードとcross-build手順をLinux x64で先に検証し、組み込み向けのtarget-specific linker／sysroot調整を分離できる。ただしx64で成功したbuildはARM上の実動作を保証しない。
- **組み込みLinuxで別途必要な検証:** GPU driver、display mode/rotation、input device、IME、power management、性能はboard固有であり、cross-compileやLinux x64テストだけでは代替できない。

## この判断では決めないこと

- Renderer API、render command/scene graph、invalidation・partial redraw、GPU資源寿命。
- OpenGL ESのversion、具体graphics library、software renderer実装、GPU capability fallback。
- Windowsのgraphics API/backend、Linux代表boardの機種、driver/version、解像度・rotation・refresh rate。
- Window decorationやnative titlebarの扱い、各OSのwindowing API詳細。
- Text shaping、Unicode grapheme/RTL範囲、font fallback、Textのpixel rendering品質基準。
- 各OSの初期サポート順位、正式CI target matrix、リリースblocking条件。
- Accessibilityのrole/state/actionと各Platform APIへの写像。初期Controlでの要件は別判断で具体化する。

## 承認記録

2026-09-26に利用者承認。初期Controlは共通Rendererで独自描画し、OS native controlをControl意味論のsource-of-truthにしない。Core Visual Tree／LayoutからRendererへ共通情報を渡し、Platform Adapterが表示面・input・IME・Accessibility等のOS接続を担う。対応・検証の順序はWindows → Linux x64 → 組み込みLinux ARMv7/AArch64 → その他。Linux x64で共通資産とcross-build flowを整え、ARM向けはtarget別toolchain／sysrootを用いてcross-compileする。組み込みLinuxの第一描画候補はDRM/KMS + EGL/OpenGL ESとし、既存backend検証順および実機性能計測を維持する。x64 binaryをARMで流用しない。公開サポート順、CI gate、Renderer API等は後続判断とする。DD-00の承認やDD-01以降へ進む判断ではない。
