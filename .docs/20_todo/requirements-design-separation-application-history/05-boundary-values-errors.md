# 適用前記録 / 05-boundary-values-errors.md

- 記録日: 2026-10-17
- 区分: 2026-10-17改訂前の履歴。現行仕様として使用しない
- 元文書: [05-boundary-values-errors.md](../../13_detail_design/05-boundary-values-errors.md)

---

# DD-05 値の受渡しと失敗契約

- 状態: 内容承認済み（2026-10-07）
- 作成日: 2026-10-07
- 対象: C ABI境界のscalar値、UTF-8 byte列、caller buffer、`ui_status`、optional error handle、失敗時出力
- 上位判断: ADR-0001、Runtime R1～R4、ABI A2/A5～A8/A15/A18、基本設計BD-01/27
- 先行詳細設計: DD-01 Handle登録、DD-02 所有権と終了、DD-03 Dispatcher、DD-04 Callbackと再入（承認済み）
- 後続: DD-06 ABIシンボルと互換性、DD-07 Property、DD-09 Validation、DD-10 Command、DD-18～21 各wrapper
- 作業計画: [DD-05 作業計画](../dd05-boundary-values-errors.md)

本書のDD-VAL-01～07は利用者承認済み（2026-10-07）。既承認のA2/A5～A8/A15/A18、BD-01/27、DD-01～04を変更しない。後続欄に記した未決事項は各担当設計へ送る。C宣言、擬似コード、例示statusは設計レビュー用であり、配布用ヘッダまたは実装済みAPIを意味しない。

## 1. 範囲と境界

このフェーズでは、言語間で共通となる値・byte列・errorの受渡しと、失敗した呼出しで観測される出力・副作用を扱う。値型の意味やAPI固有の成功・拒否規則は、その所有設計に従う。

| 対象 | DD-05で提案する範囲 | 後続へ送る内容 |
|---|---|---|
| 固定幅scalar | C ABI型で値渡しし、暗黙の型変換を追加しない | Propertyごとの関数シンボルとF64/DECIMAL ABI詳細はDD-07、各wrapper |
| UTF-8 | byte長、入力借用とcopy、caller buffer出力、容量不足時の規則 | Property値・TextBox・Errorごとの具体APIはDD-07、DD-14、DD-06 |
| error | statusと任意error handleの関係、snapshot、不在時の動作 | 完全なdomain/code catalogと数値割当はDD-06・各機能設計 |
| 失敗時出力 | required output初期化、optional error出力、部分状態変更の境界 | APIごとのtransaction/rollbackと個別結果意味論は各DD |
| buffer | 既存のbyte列に対するpointer/capacity/length | 配列・Collection・一貫snapshot ABIはCollection設計 |

## 2. レビュー判断

| ID | 承認済み判断 | 出典・境界 | 状態 |
|---|---|---|---|
| DD-VAL-01 | 固定幅scalarはC ABIで定めた型に値として渡す。wrapperは明示的な型対応を行い、範囲外整数の切り詰め、異なるProperty kind間の暗黙変換、言語オブジェクトの直接受渡しを行わない。 | ADR-0001、A5、BD-01、L1～L3。F64/DECIMALの具体署名・符号化・生成/read/writeはDD-07とwrapperへ送る。 | 承認済み（2026-10-07） |
| DD-VAL-02 | UTF-8入力はpointerとbyte lengthで受け、呼出し中のみ借用する。長さが正でpointerがnullなら`UI_INVALID_ARGUMENT`、nullと長さ0は空byte列を表す。有効なUTF-8だけ受理し、保持する場合は呼出しが戻る前にcopyする。埋め込みNULはbyte列中の値として許す。 | A6/A9/A29、BD-01、ABI §2/7/9/20。callback候補など別契約で一時借用と明記する入力にはその寿命規則を適用する。 | 承認済み（2026-10-07） |
| DD-VAL-03 | byte列getterはcaller bufferへcopyし、byte数をlength/capacityで表す。`buffer == NULL && capacity == 0`は容量照会、NUL終端は付けない。容量不足なら`UI_BUFFER_TOO_SMALL`と必要byte数を返し、buffer本体は一部も書き換えない。 | A6、ABI §2.1/7.2。容量照会と後続copyの間で値が変わらないsnapshotは保証せず、同一snapshotが必要なAPIは所有設計で別途定める。 | 承認済み（2026-10-07） |
| DD-VAL-04 | `ui_status`を呼出し結果の一次判定とする。`out_error`は任意で、非nullの場合は呼出し開始時に0へ初期化し、失敗詳細を生成できたときだけ新しい所有error handleを返す。成功時は0。 | A2、A15/A18、DD-01。失敗時のError handle所有権は呼出し側がretain/release規則に従って管理する。 | 承認済み（2026-10-07） |
| DD-VAL-05 | Error handleは失敗時点のimmutable snapshotとする。機械分岐はstatus/domain/codeを用い、message文面に依存しない。`out_error == NULL`、またはerror snapshotを作れない資源不足時も、元のAPI statusを返し、詳細なしで失敗を表す。 | A2/A15/A18、BD-27。causeとdiagnosticは区別し、取得API・catalog詳細を本判断で追加しない。 | 承認済み（2026-10-07） |
| DD-VAL-06 | 各APIは処理を始める前に、存在するrequired out pointerを可能な範囲で初期化する。失敗時のhandle outputは0、scalar/bool/length outputは0を基本とし、必要容量を通知する`UI_BUFFER_TOO_SMALL`等、定義済みの個別例外だけ上書きする。required out pointer自体がnullの場合は`UI_INVALID_ARGUMENT`とし、その指す出力は書けない。 | Runtime §7、ABI §2.1、DD-01。feature query等、既存APIで異なる出力例外はそれぞれの契約を明記する。 | 承認済み（2026-10-07） |
| DD-VAL-07 | 呼出しが入力検証・必要copy・出力確保に失敗した場合、その呼出しに属するCore状態と通知を部分更新しない。状態を変えるAPIは検証とcopyを済ませてからcommitする。これは全APIを跨ぐtransactionやcallback内のアプリ副作用rollbackを意味しない。 | A8、ABI §7.4/9.3、BD-35。Collection batchの全件適用/全件拒否はBD-35のCollection変更だけに適用する。post成功/失敗時のclosure所有権はDD-04の規則に従う。 | 承認済み（2026-10-07） |

### 2.1 `out_error`と失敗出力の形（概念）

以下は上記判断の読みやすさのための概念表であり、APIごとの優先順位や公開署名を追加するものではない。

| 呼出し結果 | status | required outputs | optional `out_error` |
|---|---|---|---|
| 成功 | `UI_OK` | 成功値を設定 | 0 |
| API失敗・詳細生成成功 | 失敗理由に対応するstatus | 失敗規則に従い初期値。個別例外のみ明示 | 新しい所有Error handle |
| API失敗・詳細なし/生成不能 | 元の失敗status | 失敗規則に従い初期値 | 0 |
| buffer不足 | `UI_BUFFER_TOO_SMALL` | bufferは不変、必要byte数を設定 | 個別APIの契約に従う。通常の追加Errorは必須にしない案 |

statusの優先順位、domain/code値、出力が複数あるAPIの個別例外はDD-06または対象API設計で固定する。

## 3. UTF-8とbufferの使用規則

1. `length`と`capacity`はbyte単位であり、文字数・code point数・UTF-16 code unit数ではない。
2. 入力pointerは呼出し中のみ有効とする。Coreが値を保持するときは呼出しが戻るまでに所有copyを作る。copy前に入力が不正と分かった場合、状態を変更しない。
3. 空文字列はnull pointerと0 lengthで渡せる。正のlengthとnull pointerの組合せを拒否する。length 0で非null pointerの場合、pointer先を読み取らない。
4. 出力getterは保持値をcaller bufferへcopyする。埋め込みNULがある値はlengthで区別し、末尾NULを追加しない。
5. 容量照会のあとにcopyを呼ぶ場合、間に値が変わる可能性がある。`UI_BUFFER_TOO_SMALL`なら新しい必要lengthを使って再試行する。同じ時点のsnapshotが必要なら、API側で別の明示手段を設計する。
6. 未定義の配列・Collectionをこの規則で暗黙に設計しない。既存のbyte列以外の集約値は、個別に要素数・byte数・snapshot・所有権を定める。

## 4. 型とerror意味論の境界

- BOOL、I64、UTF8、F64、DECIMALの値意味論はA5/BD-01の範囲を維持する。F64は承認済みのbinary64、NaN・無限大受理、NaN相互等価と±0等価の規則を変更しない。
- DECIMALはBD-01の8-byte値域・scale・係数・範囲拒否を維持する。本書ではC ABI上の符号化方式、endian、型別関数、operator/parse/formatを確定しない。
- nullabilityは値kindとは別の未決事項であり、handle 0または空文字列からnullを推定しない。既存A7のnullable OBJECT規則以外に一般null契約を追加しない。
- `UI_VALIDATION_FAILED`等の機能別statusは意味論を所有する設計に従う。statusをdomain/codeや説明messageで代用しない。
- Errorのcauseは原因の連鎖、diagnosticは任意の調査補助情報である。diagnosticの存在、完全性、安定性を通常エラー処理の条件にしない。

## 5. 検証ケース

- 4言語のwrapperで、bool、符号付き64-bit、binary64、DECIMALの許容値を同じ意味で渡す。型不一致・整数範囲外は暗黙変換や切り詰めをせず拒否する。
- UTF-8入力で、空、マルチバイト、埋め込みNUL、不正sequence、null pointerと正length、長さ0を確認する。保持値が入力bufferの後続変更に影響されない。
- caller bufferで容量照会、正確な容量、過小容量、空値、埋め込みNULを試し、buffer不足時に出力bufferのbyte列が変わらず、必要byte数が返ることを確認する。
- getterの容量照会後に値を変更し、retryが新しい必要長を返せることを確認する。同一snapshotは保証しない。
- required out pointerがnull、無効handle、kind違い、wrong thread、closed Runtime、validation rejection、allocation failureで、statusと定義済みout値を確認する。
- `out_error`を省略、指定して成功、指定して失敗、error生成不能の各条件で、statusを維持しerror handleが0または所有handleになることを確認する。Error snapshotが後続操作で変化しない。
- 失敗するsetter/create操作が値・通知・登録状態を途中更新しないことを確認する。API個別の既承認callback side effectやCollection batch契約を共通rollbackと混同しない。

## 6. 未決境界と後続担当

| 未決事項 | 後続 |
|---|---|
| API全体のstatus優先順位、ABI symbolと正式なstatus/domain/code数値catalog | DD-06 |
| F64/DECIMAL kind-specific Property create/get/set/Validator署名、DECIMAL固定8-byteのABI符号化 | DD-07、その後DD-18～21 |
| Property setter・Binding・Validationのcommit/notification/error状態 | DD-07～09 |
| Command result/error snapshotとoperation statusの具体API | DD-10 |
| 配列・Collection data、batch、snapshot consistency | Collection専用設計と上位判断 |
| C++/C#/Rust/Pythonの具体的なerror型、例外/Result変換、文字列runtime表現 | DD-18～21 |
| cause取得、diagnostic payload/size/depth、message localization | DD-06または対象機能設計 |

## 7. 承認記録

利用者は2026-10-07に「承認します」と確認した。DD-VAL-01～07を内容承認済みとして記録した。後続欄に残した範囲はそれぞれの担当設計で具体化し、本書の承認でDD-06以降の判断や実装は自動的に承認されない。
