# 0.0.1 Input / Focus event path実装開始計画

- 計画版: 0.1
- 状態: 計画承認済み（利用者、2026-11-02）。フェーズ1設計・フェーズ2試験登録は承認済み・push済み。version branch同期とCI確認を完了し、`impl-rust/core-input-focus`を作成・push済み。フェーズ4ではFocus祖先無効化・Core pointer hit-test・Button primary pointer activationをpush済み。`UNIT-CORE-091`全体とC ABIは継続中
- 対象version: `0.0.1`
- 親計画: [Input / Focus event path次工程計画](input-focus-event-path.md) 0.2（承認済み）
- 目的: 承認済み共通設計に残る実装境界を閉じ、Core/C ABI・4 wrapper・Windows Adapterを依存順に実装・検証できる状態にする

## 現状と開始条件

共通Input/Focus設計とUI試験設計は承認済みだが、4言語wrapperのborrowed event facade、key identity/modifierとprimary pointer buttonの数値表、Windows keyboard/IME message mappingは未確定である。これらを実装都合で決めず、wrapper設計・試験設計の文書単位レビューで確定する。

現在の作業branchは`impl-rust/core-window-text-service`で、`origin/version/0.0.1`より96 commit進み、release branchだけにあるclose-intent commitが1件ある。実装branchを切る前にこの差を明示的に同期する必要がある。確認時点ではworktreeはclean。

## 作業範囲

初期共通機能は承認済み範囲に限定する。

- KeyDown / KeyUp、TextCommit、PrimaryPointerDown / PrimaryPointerUpをCoreへ配送する。
- Widget単位のtyped Input subscription、target-to-Logical ancestor bubbling、handledによる後続処理停止を提供する。
- Focus getter、disable/detach/shutdown時の状態無効化、Buttonのgeneration-bound pending pressを実装する。
- callback内だけ有効なborrowed event viewを4 wrapperから安全に公開する。
- Windows native eventを値copyしてgeneration付きsinkへ順序よく投入する。WndProcからuser callbackを呼ばない。

Pointer move/wheel/capture、Tab traversal、shortcut、public Focus notification、OS固有IME composition APIは追加しない。

## フェーズ

| 順 | 成果物 | 完了条件 | 状態 |
| --- | --- | --- | --- |
| 1. API境界の具体化 | Rust/C++/C#/Python wrapper DD、共通C ABI DD、Windows platform DD | 各言語のevent型・callback署名・subscription cancel/dispose・event view寿命・例外隔離、C ABI feature/payload、key/modifier/button表、Windows key/text/IME取得経路と順序を文書化し、全対象文書の承認を得る | 完了。対象DD全件承認・push済み |
| 2. 試験・unit登録 | `.docs/32_unit_test/wrappers.md`、`ui.md`追補、`.docs/32_unit_test/implementation-test-inventory.md`、`tools/units.yaml` | 各unitにtest path・command・RQ/TC・承認済み設計文書/版/節を登録。`planned`で登録し、実装着手時にpath確認後`ready`へ更新する | 利用者承認済み、commit `8992b04` push済み。文書リンク・manifest整合pass |
| 3. Branch同期 | `version/0.0.1` → Core APIs child | `origin/version/0.0.1`を現Core APIs branchへ取り込み、close-intent差分が保持されたこととCI/差分を確認してpush。次に`impl-rust/core-input-focus`を同期済みCore APIs branchから作成 | 完了。version commit `39b65b8`をmerge commit `f2047b5`で同期。差分はrelease TODO 1ファイルのみ。Windows/Ubuntu CI runs `41377564736` success。child `impl-rust/core-input-focus`を作成・push済み |
| 4. Core / C ABI | `UNIT-CORE-091`、`UNIT-ABI-022`候補 | route/reentrancy/lifetime/focus/default actionのCore試験後、borrowed event callback/getter/handled setterをABI probesとreal ABI smokeで検証。各unitは同一revisionで必要gates pass後に日本語commit/push |
| 5. Language wrappers | Rust、C++、C#、Python各wrapper unit | 4本をCore Input branchの同じ確定commitから独立に切る。各unitでfake ABI試験と利用者向けconsumer試験を実施し、callback root/cancel/borrow lifetime/例外隔離を確認。同一unitの試験pass後に各branchを日本語commit/push |
| 6. Windows Adapter | `UNIT-PLATFORM-WINDOWS-007`候補 | Windows branchはCore Input branchから独立に切る。fake message mapping、stale generation、event order、shutdownを試験し、Windows build/CIを通す。実機IME/keyboard/mouse smokeはrelease qualificationに残す |
| 7. 集約レビュー | Core・4 wrappers・Windows Adapterの変更一式 | unit台帳、対応試験、同revision CI、release testの未実施項目を照合。関連TODOが全て完了してからまとめてmerge PRを作成 |

## Unit候補と責務

以下のIDは空き番号確認に基づく候補で、試験設計・path確定後に正式登録する。番号の確定だけで設計承認や実装完了を意味しない。

| 候補 | 責務と主な試験 |
| --- | --- |
| `UNIT-CORE-091` | InputRouter、route snapshot、handled、callback中detach/disable、Focus無効化、Button pending press。`UT-INPUT-001..005`、`UT-FOCUS-001..004` |
| `UNIT-ABI-022` | feature 9、typed callback、callback中のみ有効なborrowed event view、payload getters、handled setter。C/C++ header probeと実ABI smokeを含める |
| `UNIT-WRAP-RS-INPUT-001` | Rust borrowed event facade、closure root、Subscription解除・drop中callback |
| `UNIT-WRAP-CPP-INPUT-001` | C++ borrowed facade、move-only Subscription、例外無効/有効両profileのtrampoline隔離 |
| `UNIT-WRAP-CS-INPUT-001` | C# event view、delegate GC root、Dispose/callback競合、例外handler |
| `UNIT-WRAP-PY-INPUT-001` | Python event facade、callable root、GIL/foreign callback境界、例外隔離とGC |
| `UNIT-PLATFORM-WINDOWS-007` | Win32 keyboard/TextCommit/primary pointer/focus messageからCore sinkへのgeneration-aware copy/order |

Pythonには既存のwrapper unit IDがないため、初回登録時に既存の命名・CI選択規約との整合を確認する。`.docs/32_unit_test/implementation-test-inventory.md`のunit/schema規則に従い、planned状態を実行可能扱いしない。

## Branch依存

```text
main
└─ version/0.0.1
   └─ impl-rust/core-window-text-service (close-intent commitを同期)
      └─ impl-rust/core-input-focus
         ├─ impl-rust-wrapper/input-focus
         ├─ impl-cpp-wrapper/input-focus
         ├─ impl-csharp-wrapper/input-focus
         ├─ impl-python-wrapper/input-focus
         └─ impl-rust/platform-adapter-windows-input
```

Core branch同期と新規実装branch作成はこの計画のレビュー承認後に行う。wrapperは相互に親とせず、Windows Adapterもwrapper branchを親にしない。実装完了後の統合はCore APIsを先に、その後4 wrapperとPlatform Adapterをまとめる。

## 試験・完了条件

- Core/UIのfake event試験でroute順序、handled、detach/disable後の停止、非再入、Focus遷移、Buttonのpress/release・stale generationを個別に確認する。
- C ABIではcallback中だけborrowed handleが使え、callback終了後は無効化されること、unknown event値拒否、feature query、C/C++ header compile、実ABI経由の登録/解除を確認する。
- wrapperごとにcallback state rootの確保/登録失敗/解除/close/drop、callback自己解除、callback例外の隔離、短命event viewの保存拒否または明示copyを確認する。
- Windows fakeではnative message mapping・keyboard/textの区別・IME commit・DIP変換済みpointer座標・event順・stale window generation・shutdownを確認する。OS実機確認はCI合格と分ける。
- 各実装unitのtestとsourceは同一revisionで検証する。必要なworkspace check、Rust minimum、C/C++/C#/Python checks、document links、`git diff --check`をunit境界に応じて行う。
- Unit全件の状態・証跡が揃うまでrelease全体の完了・実機合格とは扱わない。

## レビュー対象

この計画0.1の実行は利用者承認済み。フェーズ1の設計文書とフェーズ2の試験設計・unit登録案はそれぞれ文書単位で承認済み。フェーズ3のbranch同期・CI確認も完了し、現在は承認済み範囲に沿ってフェーズ4以降を実装中。

## レビュー履歴

| 日付 | 対象 | 状態 | 備考 |
| --- | --- | --- | --- |
| 2026-11-01 | 計画0.1 | レビュー案 | 承認済み共通設計からwrapper/API詳細化・unit登録・branch同期・実装順を具体化 |
| 2026-11-02 | 計画0.1 | 利用者承認 | Input/Focus event path計画を承認。フェーズ1から開始し、個別DDと試験設計は文書単位で別途レビュー |
| 2026-11-02 | フェーズ1 | 完了 | ABI・4 wrapper・Windows native mappingのDDを承認済み・push済み |
| 2026-11-02 | フェーズ2 | 利用者承認・完了 | wrapper/platform試験設計・planned unit 7件をcommit `8992b04`へ記録、push済み |
| 2026-11-02 | フェーズ3 | 完了 | version commit `39b65b8`を同期。merge commit `f2047b5`のWindows/Ubuntu CI成功後、Core child `impl-rust/core-input-focus`を作成・push |
| 2026-11-02 | フェーズ4 | 進行中 | Core Focus eligibilityへLogical祖先の`IsEnabled`確認を追加し、直接Property変更・Binding伝播共通の後処理で無効subtreeのFocus/candidateを解除。Layout/ControlTree snapshotからclient clip・ancestor clip・topmost順を使うpointer hit-testを追加。Input route snapshot、typed event payload validation、handled、Widget subscription、callback中detach再検証、self-cancel/panic時のlifecycle解放、keyboard/TextCommitのFocus target選択、pointer hit-test target選択を追加。route/subscription一式をcommit `7b37ac4`へpush。target選択testを含むCore全192件、workspace Clippy/format、文書リンク、diff checkがpass。Button default actionとC ABI、routeの非再入/queued deliveryは未完了 |
| 2026-11-03 | フェーズ4 | 進行中 | 再入InputをRuntime queueへ積み、dispatch_turnあたり1件ずつ、callback復帰後に配送する処理を追加。shutdown時は未配送queueを破棄し、残件は次のnormal wakeを予約する。自己再入試験を含むCore全193件、workspace Clippy、format、文書リンク、diff checkがpass。Button default actionとC ABIは継続中 |
| 2026-11-03 | フェーズ4 | 進行中 | Buttonの未handled primary downでweak pending pressを記録し、新downで置換。未handled upで同Button・同Window generationを照合して一度だけCommandを実行する既定動作を追加。handled、target不一致、祖先disable、detach、Window close/reopenでは実行しない。UT-INPUT-005を含むCore全202件、workspace Clippy/format、Rust 1.85 check、文書リンク、diff checkがpass。C ABIは継続中 |
| 2026-11-03 | フェーズ4 / UNIT-ABI-022 | 実装・local検証完了、同revision CI待ち | ABI feature 9、5種Input callback、callback-scoped event getter/setter、borrowed Widget retain、外部cancel/self-cancelとdestroy、C/C++ header、C ABI smokeを追加。C ABI全40件、workspace Clippy/format、Rust 1.85 check、文書リンク、diff check、Windows CMake C/C++ header probe・ABI runtime smokeがpass。local default Ninja presetはNinja未導入のためVS 2026 generatorで同じCMake targetsを検証。最新実装revisionを作業branchへpush後、同revision CIを確認 |
