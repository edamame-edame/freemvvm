# C++ Runtime wrapper APIとcleanup境界

- 更新日: 2026-10-22
- 状態: レビュー案。実装未着手
- 対象: C++ wrapperやCMake packageを実装する前に、最初の公開Runtime所有APIと、native release失敗時の寿命境界を決める。

## 次にこの判断を行う理由

C++17 consumerからraw C ABIを呼ぶsmokeは`f3c3f78`で完了した。[RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004)は、public C++ wrapperと共通native libraryを含むCMake install/export packageを要求している。承認済みの[C++ wrapper設計](../13_detail_design/wrappers/cpp.md)では、move-only所有、明示shutdown、non-throw結果経路、release失敗の保全を定めている。一方、公開型・Resultの名前、cleanup registryの寿命、library unloadの規則は未決である。これらを決める前にpackageを作ると、未承認のAPIを固定してしまう。

## 承認済み根拠

- 要求: [RQ-03-001](../11_requirements/RQ-03_language-support.md#rq-03-001)、[RQ-03-005](../11_requirements/RQ-03_language-support.md#rq-03-005)、[RQ-04-006](../11_requirements/RQ-04_abi-boundary.md#rq-04-006)、[RQ-04-016](../11_requirements/RQ-04_abi-boundary.md#rq-04-016)、[RQ-05-012](../11_requirements/RQ-05_ownership-callbacks.md#rq-05-012)、[RQ-06-002](../11_requirements/RQ-06_threading-dispatch.md#rq-06-002)、[RQ-12-004](../11_requirements/RQ-12_build-distribution.md#rq-12-004)
- アーキテクチャ: [ADR-0001](../04_approval/adr-0001-rust-core.md)。versioned C ABIを持つRust Coreを採用。
- 詳細設計: [C++ wrapper](../13_detail_design/wrappers/cpp.md) v1.1、[共通C ABI](../13_detail_design/runtime-objects/abi.md) v1.2、[ビルド・配布](../13_detail_design/build/toolchain-packages.md) v1.1
- 既存の試験観点: [UT-WRAP-CPP-001](../32_unit_test/wrappers.md#ut-wrap-cpp-001)、[TC-WRAP-CPP-01](../30_release_test/requirement-test-matrix.md#case-tc-wrap-cpp-01)

## レビュー案

最初の公開APIはRuntimeの作成、UI thread判定、明示的なshutdown/request-shutdown、明示的なtoken closeに限定する。基底APIはC++17で使えるnon-throwの結果型とし、throwing convenience APIは後続に分ける。Runtimeの破棄でshutdownを暗黙に開始しない。公開名と結果型の厳密な構造は、この案だけでは確定しない。

所有権については、`handle_release`に失敗してもowned tokenを捨てない承認済み規則を維持する。owned tokenを得るABI呼び出しの前にcleanup nodeを確保し、destructor内のrelease失敗時にはtokenと予約済みnodeを追加allocationなしで移す。別Runtime間でcleanup状態を共有しないよう、process-global registryよりRuntime単位のcleanup ownerを推奨する。未処理releaseがある間はC ABI libraryをロード状態に保ち、全件drain済みか未解決として明示されるまで解除しない。

### 承認を求める推奨案

- 最初にmove-onlyの`freemvvm::Runtime`を公開する。操作は`create`、`is_ui_thread`、`shutdown`、`request_shutdown`、明示的な`close`に絞る。destructionではtokenをreleaseするだけでshutdownを始めない。
- raw `ui_status`と、返された場合はowned Error snapshotを保持する、小さなC++17対応の`Result<T>`/`Result<void>`を基底経路にする。throwing convenience、Property wrapper、callback adapterは後続にする。
- Runtimeごとに共有の内部control blockを持たせ、事前確保したintrusive pending-release listを管理し、owned tokenまたは未処理releaseがある間はnative moduleをpinする。Runtimeに結び付くwrapperは同じcontrol blockを共有する。destructorはrelease失敗をlistへallocationなしで移し、明示closeは直近の失敗を返す。safe point/shutdownでlistを再試行し、未解決nodeがある間はrelease関数を呼べる状態を失わないようcontrol blockとnative moduleを保持する。

これは提案であり、承認済み設計ですでに決定された内容ではない。公開APIの範囲とcleanup ownerの方式について、この案を承認するか、別案を指定してほしい。

承認済みC ABIとwrapper要求を超える新しい外部動作は追加しない。ただしdestructorから失敗releaseを安全に引き渡すには、library unloadの規則を実装前に定める必要がある。

## レビュー後の作業

- [ ] 選択した公開名、結果型、明示close/shutdownの区別、cleanup ownerの寿命を承認済みC++ wrapper設計へ反映する。
- [ ] source変更前に最初のwrapper実装とfocused testを`tools/units.yaml`および`.docs/32_unit_test/`へ登録する。
- [ ] 選択した最小Runtime wrapperを実装・検証する。
- [ ] wrapperとnative runtimeをrelocatableなCMake install/export packageへ追加し、別のdownstream consumerで利用を確認する。
- [ ] WindowsとLinuxのCI evidenceを、ローカルhostでの検証と区別して記録する。

## 改訂履歴

| 日付 | 変更 | 承認との対応 |
| --- | --- | --- |
| 2026-10-22 | raw ABI smoke完了後、最初のC++ Runtime surfaceとcleanup owner方式を提案 | レビュー案。本書によるAPI・動作の承認なし |
