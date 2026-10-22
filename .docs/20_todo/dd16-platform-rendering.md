# DD-16 描画とPlatform Adapter 作業計画

- 状態: 完了（2026-10-12、DD-PLAT-01～10承認済み）
- 着手日: 2026-10-12
- 対象: Core RendererとPlatform Adapterの接続、Window/surface lifecycle、frame scheduling、UI loop bridge、raw input/IME接続
- 上位出典: Accepted ADR-0001、Accepted要件 §6～8/12、BD-04～13/36、BD-23/24、承認済みRuntime R1～R4
- 先行詳細設計: 承認済みDD-03、DD-11～15。DD-12の単位/scale、DD-13/14のOS event細部等は未決境界を維持する
- 対応する詳細設計: `.docs/13_detail_design/16-platform-rendering.md`

## 目的

共通描画方式とOS接続を、Rust Core内の意味論およびOSごとのsurface/input/IME責務に分け、初期Controlを複数Platformへ同じ契約で接続する境界をレビューする。DD-03の抽象`UiLoopBridge`とDD-12のLayout結果を、Platformが使うタイミングに結び付ける。承認済みBD-12の共通Renderer・Platform Adapter責務およびWindows → Linux x64 → embedded Linux ARMv7/AArch64 → その他の検証順を変更しない。

## 小分けにした作業範囲

1. **境界と所有**: Core、Renderer、Platform Adapter、native Window/surface、graphics resourceの責務・寿命。
2. **Frame contract**: Property/Tree/Layout dirty、layout snapshot、描画要求、frame完成までの順序と重複集約。
3. **Window lifecycle**: Core WindowとPlatform peerの対応、作成/表示/resize/close/lost/recoveryの状態と失敗境界。
4. **Event loop接続**: DD-03のwake/dispatch turnとframe/input workを接続し、UI thread制約と公平性を維持する。
5. **InputとIME接続**: raw event正規化、surface/root座標・bounds/hit-testへの接続、FocusとIME sessionのPlatform開始/終了。
6. **検証条件と未決事項**: 端末依存検証、resource失敗・surface喪失・shutdown競合を整理し、BD/ABI/後続DDへ残す事項を明示。

## 作業手順

1. ADR-0001、要件、BD-04～13/23/24/36とR1～R4について、状態と承認範囲を確認する。
2. DD-03、DD-11～15を読み、UI thread、UiLoopBridge、Logical/Visual Tree、Measure/Arrange、Input/Focus/IME、Controlの承認済み接点と未決項目を抽出する。
3. Core RendererとPlatform Adapterの責務、Window/surface lifecycle、frame要求からLayout/描画への実行順を判断ID付きで提案する。
4. input eventはBD-07/DD-13、IME sessionはBD-10/24/DD-14の共通意味論を維持し、OS固有データと共通Core eventの変換境界だけを扱う。
5. WindowsとLinux x64の初期検証の観測点を提案し、embedded Linux実機・他OSの順序はBD-12を維持する。backend/API/libraryの具体選択は上位判断なしに固定しない。
6. `.docs/13_detail_design/16-platform-rendering.md` に未承認判断、処理フロー、Window/frame状態図、失敗・shutdown時の検証、未決事項を保存する。
7. フェーズ一覧と状況メモを作業中へ更新し、リンク・状態・`git diff --check`を確認する。提案の承認後に同じ文書・計画へ承認記録を反映する。

## 承認境界と停止条件

DD-16は2026-10-12にDD-PLAT-01～10の利用者承認を受け、判断・計画・フェーズ状況へ記録して完了した。Renderer API/ABI symbol、具体graphics/windowing/IME library、公開寸法単位・DPI/scale/rounding、Accessibility OS API、正式support/CI gate、Window close request後の判断・Core Window寿命は未決のまま後続へ残す。DD-17は利用者の次フェーズ着手指示を受けて開始する。

## 対象外

- 共通Control意味論・Window/StackPanel/Text/TextBox/Button契約（DD-15で承認済み）
- Layoutの寸法単位、座標原点、DPI/scale変換、pixel rounding（BD-06/DD-12未決）
- Input route、Focus遷移、TextBox/IME semantic state（DD-13/14で承認済み）
- OS Accessibility role/state/action mappingとAT実機検証（DD-17）
- 公開C ABI関数・event struct・wrapperごとの公開型と具体signature（DD-06、DD-18～21）
- 個別OSのbackend/library/driver/version、GPU API選定と正式target/CI release gate（上位Platform/Build判断）
- Control visual template/theme、font shapingの詳細、Controlごとのpixel品質基準（後続Rendering/Text設計）

## 完了条件

- Core Renderer / Platform Adapter / DD-03 event loop bridgeの責任・呼出しthread・所有寿命が矛盾なく記述される。
- Layout snapshotからframe描画、raw eventから共通Input、IME sessionからTextBox eventへの接続順と失敗時の状態が追跡できる。
- Window/surfaceの作成・表示・resize・close/lostとRuntime shutdown時の状態境界を記す。上位未決の意味論は明示的に保留する。
- normal、resize/dirty coalescing、input/IME、surface/device failure、wake failure、shutdown競合の検証条件がある。
- 利用者は2026-10-12にDD-PLAT-01～10を承認した。判断結果を同文書、作業計画、フェーズ状況へ反映済み。未決事項は維持する。DD-17への着手は別途指示を必要とし、同日に指示を受けて開始済み。
