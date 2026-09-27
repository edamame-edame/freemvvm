# DD-00 詳細設計の出典と判断の対応表

- 状態: レビュー案（DD-00は未承認。要求仕様の承認状態のみ訂正済み）
- 作成日: 2026-09-26
- 対象: `.docs/20_todo/detail-design-phases.md` のDD-00～DD-23
- 目的: 詳細設計で利用する承認済み判断、未決事項、初期縦断シナリオへの依存を追跡する

本書は既存文書の状態と設計判断を整理する。**新しい設計判断は行わない。** 表中の「設計先」は作業の割当てであり、その文書の内容やAPIを承認するものではない。

## 1. 出典と優先関係

| 出典 | 確認した状態・範囲 | 詳細設計での扱い |
|---|---|---|
| [ADR-0001](../04_approval/adr-0001-rust-core.md) | `Accepted` | Rust Core、versioned C ABI、4言語wrapper、組み込みLinux対応を固定前提とする |
| [要求仕様](../11_requirements/mvvm-ui-library-requirements.md) | `Accepted` | 対象範囲・縦断シナリオ・検証観点の出典とし、未決事項を判断済みとしない |
| [設計骨格](../12_design/design-overview.md) | `Accepted` | 責務分割、依存順、初期縦断シナリオを上位設計として用いる。本文が後続へ送った細則は決定済みとしない |
| [Runtime契約](../12_design/runtime-contract.md) | R1～R4承認済み | 各判断と本文の対応範囲を参照し、後続に残した実装方式は詳細設計で提案する |
| [C ABI仕様案](../12_design/abi-spec.md) | A1～A32承認済み | 判断IDと関連節を参照する。掲載されたC宣言はレビュー用であり、配布用ヘッダ・実装済みAPIではない |
| [MVVM意味論](../12_design/mvvm-semantics.md) | P1～P4、B1～B4、C1～C4、D1～D4、V1～V8承認済み | 値・通知・伝播・Command・Validationの意味論を参照する。対象外と後続設計事項を拡張しない |
| [言語Binding設計](../12_design/language-bindings.md) | L1～L4承認済み | Core Propertyを正本とする接続、明示adapter、dispatcher、寿命管理を参照する。具体的な言語APIは後続設計とする |
| [リリーステスト](../30_release_test/release-test.md)、[バージョンテスト](../31_version_test/version-test.md) | テスト規約 | DD-23の検証項目と、後続の実装・変更時のテスト計画に反映する |

ADRと下位文書が矛盾した場合はAcceptedのADRを優先し、必要なら上位文書の変更としてレビューする。判断IDのない記述を、判断表の承認範囲を超えて確定仕様とみなさない。

## 2. 承認済み判断から詳細設計への対応

| 上位判断ID | 確定している範囲の要約 | 主な設計先 |
|---|---|---|
| ADR-0001 | Rust Core、versioned C ABI、4言語wrapper、組み込みLinux | DD-01～DD-23の共通前提 |
| R1、A1 | 非再利用64-bit handle、共通retain/release | DD-01、DD-02、DD-06 |
| R2、A4 | UI thread、worker release、shutdownと未実行postの後始末 | DD-02、DD-03、DD-06 |
| R3、R4 | callbackの所有・破棄とUI thread、再入時の通知非再帰 | DD-03、DD-04、DD-07 |
| A2、A3 | status/error handle、major付きシンボルとminorによる互換追加 | DD-05、DD-06、DD-22 |
| A5～A8 | Property値の型別受渡し、UTF-8、object参照、失敗区分 | DD-05、DD-07 |
| A9～A12 | Property生成、変更通知購読、明示解除と自動後始末 | DD-02、DD-04、DD-07 |
| P1～P4 | UI threadでのProperty操作、即時確定、遅延・集約通知、購読と終了 | DD-03、DD-07 |
| B1～B4、D1～D4 | Binding初期同期、伝播順、TwoWay/OneWay競合とグラフ制約、解除 | DD-08 |
| A13～A16 | Binding handle、明示解除、エラーsnapshotと通知、作成失敗 | DD-02、DD-08 |
| V1～V8 | Property同期検証とView入力の三値判定、IME入力保持 | DD-09、DD-14 |
| A17～A20 | Property Validatorのcallback、error、解除、再入制限 | DD-04、DD-09 |
| A25～A28 | View入力Validatorの候補、状態と理由、解除 | DD-09、DD-14 |
| A29～A32 | TextBox生成、確定値Property、外部更新、破棄 | DD-09、DD-11、DD-14、DD-15 |
| C1～C4、A21～A24 | 非同期Commandの起動、状態、キャンセル、完了と終了 | DD-03、DD-10、DD-15 |
| L1～L4 | Core Propertyを正本とする4言語接続、明示adapter、dispatcher、明示解除 | DD-18～DD-21 |

この表は主要な受け持ち先を示す。複数領域にまたがる判断は、各詳細設計文書で同じ上位IDへリンクし、境界の整合を確認する。

## 3. 初期縦断シナリオの追跡

[要求仕様の第5節](../11_requirements/mvvm-ui-library-requirements.md)と[設計骨格の第8節](../12_design/design-overview.md)にある初期シナリオをDD-23で一体として検証する。各行は、先に成立させる詳細設計の主な範囲を表す。

| 観測する振る舞い | 既存の根拠 | 主な設計先 |
|---|---|---|
| Window / StackPanel / TextBox / Text / Buttonを構成する | ADR-0001の初期シナリオ、設計骨格第8節 | DD-11～DD-16 |
| TextBox入力が`ViewModel.Name`へTwoWay反映される | B1～B4、V1～V8、A13～A20・A25～A32 | DD-07～DD-09、DD-14～DD-15 |
| `ViewModel.Message`の変更がTextへOneWay反映される | P1～P4、B1～B4、D1～D4 | DD-07～DD-08、DD-15 |
| Buttonで`ViewModel.Save`を実行し、実行可否を扱う | C1～C4、A21～A24 | DD-10、DD-15 |
| focus移動とkeyboard操作ができる | 設計骨格第8節、要求仕様第5・6節 | DD-13～DD-15 |
| accessibility nameを取得できる | 設計骨格第8節、要求仕様第5・7節 | DD-17 |
| C++ / C# / Rust / Python 3で同じ振る舞いを確認する | ADR-0001、L1～L4、要求仕様第5節 | DD-18～DD-23 |

TextBoxの値接続はA29～A32で一部承認されているが、表示・論理ツリー・layout・focus・IMEの全体は後続設計である。シナリオの行をもってそれらの方式を確定しない。

## 4. 未決事項と入口条件

| 未決の範囲 | 確認できる出典 | 次の扱い |
|---|---|---|
| handle表の内部構造、トークン発行の同期、解放後の記録方式 | R1、Runtime契約第2～3節 | DD-01でレビュー案を作る。R1の外側の方式を無断で固定しない |
| Runtime停止時の内部参照と画面所有物の具体的な破棄順 | Runtime契約第3節、A11～A14、A32 | DD-02、DD-11で分割して扱う |
| イベントループ統合、負荷上限・公平性、通知配信回の上限 | Runtime契約第5節、MVVM意味論第12節 | DD-03、DD-07、DD-16の境界で扱う。platform依存の判断は基本設計を先に確認する |
| 浮動小数点・カスタム値、型変換、Collection、追加Control | MVVM意味論第1・3・16節、C ABI仕様案第19節 | 初期縦断シナリオの外側として後続の基本設計・計画へ送る |
| UI tree、Layout、Input/Focus/IME、Control表示、描画、platform、Accessibilityの方式 | 設計骨格第6・9節、C ABI仕様案第17・19・20節 | DD-11～DD-17の前に該当する `.docs/12_design/` の小範囲レビュー判断を用意する |
| 言語別のAPI名・例外表現、Python wrapper方式、package形式、target matrix | 言語Binding設計第8節、要求仕様第12節、設計骨格第9節 | DD-18～DD-22で扱い、要件・基本設計の未決判断が必要なら先に戻す |
| ライセンス、正式な対象OS優先順位、native controlと独自描画の選択 | 要求仕様第12節、設計骨格第9節 | 詳細設計のみで決めず、上位文書の判断としてレビューする |

DD-01～DD-10は、表の承認済み判断を前提に小範囲のレビュー案を作れる。個々の詳細設計は承認を受けるまで確定仕様としない。DD-11以降は、該当する基本設計の判断が揃った範囲から進める。

## 5. DD-00レビュー項目と結果

| 確認項目 | 現在の結果 |
|---|---|
| 出典の状態と承認済みIDを正しく転記したか | 要求仕様の`Accepted`への訂正は反映済み。DD-00の確認待ち |
| 初期縦断シナリオに必要な設計先に抜けがないか | DD-00の確認待ち |
| 未決事項を承認済みとして扱っていないか | DD-00の確認待ち |

- 利用者確認: 未実施。要求仕様の承認状態の訂正はDD-00の承認を意味しない
- 修正・承認結果: 未記入
- DD-01への引継ぎ: R1/A1を根拠に、Handle登録と検証だけを小さなレビュー案として作成する
