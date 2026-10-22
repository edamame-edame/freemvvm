# C++ Runtime wrapper APIとcleanup境界

- 更新日: 2026-10-22
- 状態: 計画・詳細設計v1.2承認済み（利用者、2026-10-22）。Runtime wrapperとCMake packageの実装・Windows local test完了。CI検証は未完了
- 対象: C++ wrapperやCMake packageを実装する前に、最初の公開Runtime所有APIと、native release失敗時の寿命境界を決める。

## 次にこの判断を行う理由

C++17 consumerからraw C ABIを呼ぶsmokeは`f3c3f78`で完了した。[RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004)は、public C++ wrapperと共通native libraryを含むCMake install/export packageを要求する。承認済みの[C++ wrapper設計v1.2](../13_detail_design/wrappers/cpp.md)では、move-only Runtime、明示shutdown/close、C++17 non-throw Result、release失敗時にRuntime別queueを保全するsupervisor寿命を定めている。Runtime wrapperとrelocatable packageのWindowsローカル実装・検証は完了し、次の残作業はWindows/Linux CI evidenceである。

## 承認済み根拠

- 要求: [RQ-03-001](../11_requirements/RQ-03_language-support.md#rq-03-001)、[RQ-03-005](../11_requirements/RQ-03_language-support.md#rq-03-005)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-016](../11_requirements/RQ-04_abi-boundary.md#rq-04-016)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)、[RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。versioned C ABIを持つRust Coreを採用。
- 詳細設計: [C++ wrapper](../13_detail_design/wrappers/cpp.md) v1.2全文承認済み、[共通C ABI](../13_detail_design/runtime-objects/abi.md) v1.2、[ビルド・配布](../13_detail_design/build/toolchain-packages.md) v1.1
- 既存の試験観点: [UT-WRAP-CPP-001](../32_unit_test/wrappers.md#ut-wrap-cpp-001)、[TC-WRAP-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)

## 承認済み実装範囲

最初の公開APIはRuntimeの作成、UI thread判定、明示的なshutdown/request-shutdown、明示的なtoken closeに限定する。基底APIはC++17で使えるnon-throwの結果型とし、throwing convenience APIは後続に分ける。Runtimeの破棄でshutdownを暗黙に開始しない。公開名と結果型の厳密な構造は、この案だけでは確定しない。

所有権については、`handle_release`に失敗してもowned tokenを捨てない承認済み規則を維持する。owned tokenを得るABI呼び出しの前にcleanup nodeを確保し、destructor内のrelease失敗時にはtokenと予約済みnodeを追加allocationなしで移す。Runtimeごとに独立したcleanup queueを持つが、最後のwrapperが破棄されてもqueueを失わないsupervisorが必要になる。Runtime専用queueをprocess-wide supervisorへ登録し、supervisorがRuntime単位でUI safe point/Closed shell drainを管理する案を推奨する。process-wide部分は保管・寿命管理だけを担い、Runtime間のcleanup状態や順序は混ぜない。未処理releaseがある間はC ABI libraryをロード状態に保ち、全件drain済みか未解決として明示されるまで解除しない。

### 承認を求める推奨案

- 最初にmove-onlyの`freemvvm::Runtime`を公開する。操作は`create`、`is_ui_thread`、`shutdown`、`request_shutdown`、明示的な`close`に絞る。destructionではtokenをreleaseするだけでshutdownを始めない。
- raw `ui_status`と、返された場合はowned Error snapshotを保持する、小さなC++17対応の`Result<T>`/`Result<void>`を基底経路にする。throwing convenience、Property wrapper、callback adapterは後続にする。
- Runtimeごとの共有control blockが、事前確保したintrusive pending-release listを管理し、owned tokenまたは未処理releaseがある間はnative moduleをpinする。Runtimeに結び付くwrapperは同じcontrol blockを共有する。destructorはrelease失敗をlistへallocationなしで移し、明示closeは直近の失敗を返す。control blockが最後のwrapperより長く生存できるよう、process-wide supervisorはpending nodeを持つcontrol blockを強参照で保持する。supervisorはRuntime間のqueueを混ぜず、該当Runtimeのsafe point/Closed shellで再試行する。未解決nodeがある間はcontrol blockとnative moduleを保持し、release関数を呼べる状態を失わない。

この範囲は2026-10-22にC++ wrapper設計v1.2全文として承認された。supervisor workerはC ABI releaseだけを任意threadから再試行し、UI finalizerはCore Runtime契約へ委ねる。process終了時の未解決tokenはin-process完了を保証せず、OS別pinning方式とworkerの具体的retry backoffは実装制約として扱う。

公開API・cleanup owner方式は承認済み設計v1.2に記録した。新しい外部動作は追加せず、実装は同設計に従う。

承認済みC ABIとwrapper要求を超える新しい外部動作は追加しない。linked native targetは実行ファイルの依存としてロードされ、pending cleanup stateがrelease関数を参照する期間中に明示unloadしない。

## レビュー後の作業

- [x] 最初のRuntime API、Result型、close/shutdown区別、cleanup supervisorをC++ wrapper設計v1.2へ反映し、利用者承認を記録する。
- [x] UNIT-WRAP-CPP-001とUT-WRAP-CPP-RT-001を`.docs/32_unit_test/wrappers.md`および`tools/units.yaml`へsource変更前に登録する。
- [x] 選択した最小Runtime wrapperを実装・検証する。UT-WRAP-CPP-RT-001、C++ ABI smoke、Windows全体checkに成功（2026-10-22）。Linux CIは未確認。
- [x] wrapperとnative runtimeをrelocatableなCMake install/export packageへ追加し、移動したprefixを使う別のdownstream consumerで利用を確認する。UT-PKG-CPP-001成功（2026-10-22）。Linux CIは未確認。
- [ ] WindowsとLinuxのCI evidenceを、ローカルhostでの検証と区別して記録する。

## 改訂履歴

| 日付 | 変更 | 承認との対応 |
| --- | --- | --- |
| 2026-10-22 | raw ABI smoke完了後、最初のC++ Runtime surfaceとcleanup owner方式を提案 | レビュー案。本書によるAPI・動作の承認なし |
| 2026-10-22 | 最終wrapper破棄後もpending releaseを保持するprocess-wide supervisor案を追加 | 利用者が計画を承認。旧案の寿命gapを修正 |
| 2026-10-22 | C++ wrapper設計v1.2全文承認とfocused test登録を記録 | 利用者が詳細設計v1.2全文を承認。UNIT-WRAP-CPP-001をplannedで登録し、実装開始 |
| 2026-10-22 | FreeMvvm::cpp / FreeMvvm::runtimeのinstall/exportとrelocated consumer testを追加 | 既承認RQ-12-004・build DDに基づく。UT-PKG-CPP-001をplanned登録後に実装しWindows local test成功 |
| 2026-10-22 | CI checkoutに存在しないローカルarchiveへのリンクを通常テキストへ変更 | 初回Windows/Linux CIのdocument-link gateで発見。archiveのGit除外方針は維持 |
