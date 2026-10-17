# RQ-12: ビルドと配布

- 文書版: 0.5
- 状態: 承認済み（利用者、2026-10-17）
- 更新日: 2026-10-17
- 入口: [要求仕様索引](reqirements-specification.md)

## 概要

各target向けの共通native runtimeとC ABIを生成し、4言語のwrapperから適切なtarget artifactを利用できるようにする。Build/link検証と実targetでのRuntime検証を別の結果として扱う。

本章は提供機能・利用条件・外部から観測できる結果を定める。各IDの対象段階を区別し、将来候補の列挙を採用承認とはしない。

## 要求本文

| 要求ID | 項目 | 要求詳細 | 対象段階 |
| --- | --- | --- | --- |
| <a id="rq-12-001"></a>RQ-12-001 | target triple別native artifact | target tripleごとにRust Coreを含むnative C ABI runtime artifactを生成する。cross-compile artifactは生成対象のtargetを明示し、別targetのartifactを流用・誤表示しない。 | 現行 |
| <a id="rq-12-002"></a>RQ-12-002 | OS・CPU architecture別ライブラリ | OSおよびCPU architectureごとに対応するnative libraryを提供する。ABI majorとtarget profileごとの正式support範囲を識別可能にする。 | 現行 |
| <a id="rq-12-003"></a>RQ-12-003 | 4言語共通のC ABI artifact | 同一target向けのC ABI native runtimeをC++、C#、Rust、Python wrapperから共有する。各wrapperが独自にCoreを複製したり、異なるABI runtimeを暗黙に使ったりしない。 | 現行 |
| <a id="rq-12-004"></a>RQ-12-004 | C++ CMake integration | C++の標準integrationをCMake install/export packageとする。public C++ wrapperと対応targetの共通C ABI native libraryを含み、downstream projectは`find_package(... CONFIG)`で利用する。初期targetはWindows x64とLinux x64。vcpkg/Conan registryは本要求で指定しない。 | 現行 |
| <a id="rq-12-005"></a>RQ-12-005 | Rust crateとnative runtime | Rust配布をraw FFIの`-sys` crateとsafe API crateに分け、Rust Coreはwrapper crateへ含めず共通C ABI runtimeを別artifactとして使う。別環境生成artifactの取得・取込経路をfeatureで有効にし、Cargo `--target`でtarget triple、named artifact profileでboard/toolchain差を選択する。featureをtarget指定の代替にしない。registry、fetch command、cache、signature運用は本要求で指定しない。 | 現行 |
| <a id="rq-12-006"></a>RQ-12-006 | C# NuGet・RID assets | C# managed wrapperとRID別native C ABI libraryを同一NuGet packageの`runtimes/{rid}/native/`に含める。初期RIDは`win-x64`と`linux-x64`。組み込みARM RID packageはprofile互換性確認後に判断する。feed/package IDの実配置と、RQ-03-014に定めるTFM基準を区別する。 | 現行 |
| <a id="rq-12-007"></a>RQ-12-007 | Python ctypes wrapper | 初期Python wrapperはPython標準ライブラリ`ctypes`でC ABIを呼び出し、compiled extensionを必須としない。 | 現行 |
| <a id="rq-12-008"></a>RQ-12-008 | Python wrapper/native runtime distribution | Python wrapper distributionとtarget別native runtime distributionを分ける。初期native wheel対象はWindows x64とLinux x64。組み込みLinux ARMはprofile互換性が確認されるまでmanylinux/musllinux wheelと表示しない。package名、index、exact wheel tagは本要求で指定しない。 | 現行 |
| <a id="rq-12-009"></a>RQ-12-009 | CI toolchainの固定 | CIで使用するYocto SDK、sysroot、linker、C library設定を固定し、target build条件を記録する。 | 現行 |
| <a id="rq-12-010"></a>RQ-12-010 | CI target追加順と実機検証gate | 初期CI targetをWindows x64、Linux x64、組み込みLinux ARMv7/AArch64の順で追加する。Linux x64 hostからARM向けをcross-compileする。ARM cross-build/link gateと代表実機上のRuntime/hardware profile gateを区別する。exact compiler/sysrootはbuild profileに記録し、初期profileと代表boardはRQ-11-024に従う。 | 現行 |
| <a id="rq-12-011"></a>RQ-12-011 | artifact適合 | target/profile/ABI/featureに適合するnative artifactを選べ、不適合を利用前に拒否すること。 | 初期共通基盤 |
| <a id="rq-12-012"></a>RQ-12-012 | native identity共有 | 同じnative binaryは言語package間で同じidentityを保つこと。 | 初期共通基盤 |
| <a id="rq-12-013"></a>RQ-12-013 | 明示取得とoffline利用 | 通常導入で固定版を暗黙更新せず、取得・更新を明示でき、[期限・失効条件](RQ-12_build-distribution.md#rq-12-023)を満たす検証済みartifactをoffline利用できること。 | 初期共通基盤 |
| <a id="rq-12-014"></a>RQ-12-014 | artifactの信頼と検証失敗 | 署名、内容digest、適合条件と既存の信頼根拠を検証してから公式artifactを利用すること。取得先で初めて得た鍵を自動信頼せず、未知・失効・検証不能なら拒否すること。認証済み鍵更新と明示trust bundleによるoffline検証を提供すること。 | 初期共通基盤 |
| <a id="rq-12-015"></a>RQ-12-015 | manifestの互換・曖昧性 | 未知schema major、未知必須項目、重複・曖昧で一意に解釈できないmanifestを拒否すること。未知optional項目は署名検証後に無視できるが、target・ABI・信頼判定に必要な情報をoptionalとして扱わないこと。 | 初期共通基盤 |
| <a id="rq-12-016"></a>RQ-12-016 | license/noticeの継承 | first-party source、header、wrapper、source package、binary SDKへ既承認のApache-2.0方針を適用し、再配布で必要なlicense/noticeと変更表示を保持すること。third-partyのlicense条件を独立して維持すること。 | 初期共通基盤 |
| <a id="rq-12-017"></a>RQ-12-017 | major保守期間 | 最新安定ABI majorを通常保守し、直前majorは新安定major後24か月、security/重大障害修正に限って保守すること。 | 初期共通基盤 |
| <a id="rq-12-018"></a>RQ-12-018 | 保守状態公表 | ABI majorとtarget profileごとに保守状態を公表すること。 | 初期共通基盤 |
| <a id="rq-12-019"></a>RQ-12-019 | 非推奨とsecurity例外 | profile非推奨は原則12か月前に通知し、security上の短縮時は理由と影響を示すこと。 | 初期共通基盤 |
| <a id="rq-12-020"></a>RQ-12-020 | EOL後の参照 | EOL後も最終公開物・support表・告知を参照できること。 | 初期共通基盤 |
| <a id="rq-12-021"></a>RQ-12-021 | 再配布許諾範囲 | first-party source、改変source、改変binary、app binaryの再配布を許可すること。third-partyの個別license条件は独立して維持すること。repository公開、CLA/DCO、docs/assetsのlicense、SBOM運用の追加を含めないこと。 | 初期共通基盤 |
| <a id="rq-12-022"></a>RQ-12-022 | manifest資源上限 | manifestの資源制限を構成でき、初期既定値はbytes1 MiB/depth32/member4096/string64 KiB/signature8とすること。制限超過を明示拒否し、許容される新manifestへの変更とpolicy変更を追跡できること。 | 初期共通基盤 |
| <a id="rq-12-023"></a>RQ-12-023 | offline期限・rollback/失効 | 明示trust bundleでも期限切れ・rollback・検証不能を拒否すること。期間をroot365日/manifest30日/revocation7日とし、長期offlineは別profileの承認済み期間を用いること。取得していない最新失効を即時検知すると保証しないこと。 | 初期共通基盤 |
| <a id="rq-12-024"></a>RQ-12-024 | 保守・証拠の保存 | EOL後も最終artifact・metadata・support告知を参照可能にすること。内部test logはEOL+24か月、security監査logは2年という期間で運用責任を定めること。 | 初期共通基盤 |
| <a id="rq-12-025"></a>RQ-12-025 | 初期link形式 | 初期配布は共通native dynamic runtimeを利用し、各言語へ独立Coreを静的同梱するvariantを提供対象に含めないこと。 | 初期共通基盤 |


## 適用境界

承認は要求本文の範囲で完結する。API名・内部データ構造・処理方式の確定を前提にしない。既存ADRによるRust Core・versioned C ABI等の明示的な構成制約は維持する。

## 承認履歴

| 日付 | 対象版・ID / 範囲 | 承認者 | 結果・根拠 |
|---|---|---|---|
| 2026-09-26 | 旧要求仕様の該当範囲 | 利用者 | Acceptedを継承。後日追記の承認日・未決範囲は旧版で確認 |
| 2026-10-12 | 要求承認管理・章分割・RQ ID化 | 利用者 | 今回の指示による管理方針。旧本文に記載済みの承認事項を要求側で管理する |
| 2026-10-12 | RQ-12全掲載ID / 再編集版0.3 | 利用者 | 要求詳細と受入確認を承認 |

## 改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 | 承認履歴への対応 |
|---|---|---|---|---|
| 旧版 | 2026-09-25 | 旧要求仕様を作成。2026-09-26にAccepted | 旧文書参照 | 旧要求仕様の承認記録 |
| 0.1 | 2026-10-12 | 章分割、RQ ID、受入確認案・設計/testリンク、要求承認履歴の集約 | Codex | 管理方針の指示に基づく再編集。版全体のレビューは未完了 |
| 0.2 | 2026-10-12 | 下流参照リンクを除去し、参照方向を明記 | Codex | 要求本文・承認状態は変更なし |
| 0.3 | 2026-10-12 | 各言語の配布条件、targetの適合判定、CI gateと受入結果を要求別に明記 | Codex | 再編集版を利用者が同日承認 |

## 今回の承認履歴

| 日付 | 対象版・範囲 | 承認者 | 結果 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.4 / 全レビュー案の要求補足・意味単位移管 | 利用者 | 全レビュー案の反映指示により承認。実装・試験実行の完了とは区別する |

## 今回の改訂履歴

| 版 | 日付 | 変更内容・理由 | 作成者 |
| --- | --- | --- | --- |
| 0.4 | 2026-10-17 | 要求と実現手段・検証・作業記録を分離し、正式IDと参照を対応付けた | Codex |

## 承認履歴（具体化適用）

| 日付 | 対象版・範囲 | 承認者 | 結果・根拠 |
| --- | --- | --- | --- |
| 2026-10-17 | 0.5 / 要求変更 RQ-12-022, RQ-12-023, RQ-12-024, RQ-12-025 | 利用者 | 全件反映を指示。設計採用・本文適用を承認。実装・公開・試験合格を意味しない |

## 改訂履歴（具体化適用）

| 版 | 日付 | 変更内容 | 作成者 |
| --- | --- | --- | --- |
| 0.5 | 2026-10-17 | 要求変更 RQ-12-022, RQ-12-023, RQ-12-024, RQ-12-025を正本へ適用し、旧判断との対応を保持 | Codex |
