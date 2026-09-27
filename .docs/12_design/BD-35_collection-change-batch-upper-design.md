# MVVM UIライブラリ Collection change batch原子性に関する上位判断

- 状態: BD-35承認済み（2026-09-27）
- 対応: BD-33のCollection change set/batchの検証、適用失敗、Selection整合性
- 根拠: BD-02、BD-03、BD-33、BD-34、R2～R4。DD-00は未承認

## 既存の承認済み契約

- BD-33はCollection mutationを構造化change setで表し、連続更新をbatch化できること、stable item IDを保つこと、worker mutationはUI threadへpostすることを承認した。
- BD-34はSelectionModelをstable IDで管理し、削除されたitemをselection/current itemから除去することを承認した。
- BD-02はRuntime全体の明示UI transactionや複数Propertyの自動rollbackを初版に保証しない。一つのCollectionModel操作内の整合性契約はこの全体transactionとは別に決める必要がある。
- R2～R4はCollection/SelectionのCore object更新をUI threadに制限する。

## BD-35 承認済み判断

1. **一つのCollection change batchを一つの原子的Model mutationとして扱う。** 事前検証はCollectionの構造・ID・index・宣言schemaなど、Modelへ適用できるかを判定する。View入力の値検証やアプリ固有の業務検証を、batch全体のcommit条件にはしない。batchをまたぐProperty更新や複数Collectionをまとめる一般transactionは提供しない。
2. **batchのいずれかのoperationが無効なら、batch全体を拒否する。** invalid index/ID、schema不一致、重複IDなどのModel整合性違反を返し、Collection値、順序、revision、Selection state、公開通知を変更しない。この原子性は同じbatchへ意図的にまとめたModel変更に限る。独立した入力欄・cellの編集commitは既定で別々に適用し、一つの入力が他の入力の検証完了を待たない。
3. **有効batchは一つの新しいCollection revisionとしてcommitする。** batch内operationの順序は適用順として定義する。consumerには中間stateを通知せず、commit後のsnapshotと整合したCollection change notificationを一括で公開する。
4. **SelectionModelはCollection commitと同じ論理更新区間でreconcileする。** commit後に存在しないitem IDをselected/current stateから除去してから、Collection/Selectionの外部callbackを起動する。notificationの細かいcallback順序・回数は後続設計で決めるが、中間の不整合状態をcallbackから観測させない。
5. **Collection batch原子性をBD-02の一般UI transactionと混同しない。** 一つのModelの構造変更だけをall-or-nothingにし、Model変更に伴う別ViewModel Propertyや他Modelの副作用を自動rollbackしない。
6. **batch適用はUI thread上で行う。** workerはbatchを直接適用せず、`runtime_post`でUI threadへ渡す。post自体のshutdown後実行保証は既存Runtime契約に従い、適用されなかったbatchを成功扱いにしない。

## 入力検証との境界

CollectionModelはList／Table／Gridなどの項目データを保持する。CollectionBatchは複数入力欄をまとめて検証するフォーム単位の仕組みではなく、Collection構造や項目データを更新するModel operationのまとまりである。

TextBoxまたは編集中cellは、その入力欄の編集値を個別に検証する。検証に通った入力は対応するProperty／item updateとして個別にModelへ反映でき、別の入力欄の未入力・検証エラーを待たない。検証エラーの値はその入力欄の編集状態に留める。複数fieldを合わせた業務検証や、保存操作まで全体を保留するかはViewModel／アプリケーション側の責務であり、BD-35の原子性からは要求しない。

複数operationを同じbatchへ入れるのは、それらを一体のCollection変更として適用したい場合に限る。たとえば一つのデータ更新で複数rowを差し替える場合は一括拒否が有用だが、利用者が別々のcellを編集した結果を不用意に同じbatchへ結合しない。

## 理由とトレードオフ

- all-or-nothing適用により、複数行/セル更新の途中でinvalidな部分状態がSelectionやGridへ見えるのを防げる。
- 一つのrevisionと一括通知は、virtualized viewが可視範囲を同じデータsnapshotから再取得できる境界を作る。
- Model内の原子性だけに限定するため、複数setterをまとめる一般transactionや広範なrollback機能は導入しない。
- 無効operationひとつでbatch全体を拒否するので、producerはbatch内容を修正して再送する必要がある。そのため独立した入力編集を同じbatchへ不用意にまとめない。
- callback回数・同一UI turnでのdispatch順序・大きなbatch分割は別途定める必要がある。

## この判断では決めないこと

- change batch ABI/schema、revisionの整数幅、operation encoding、batchのbyte/item上限。
- batch内operationの参照index規則、temporary ID、reset後にIDを維持する条件。
- duplicate notificationのcoalescing、callback order、reentrant mutationの扱い。
- batchサイズが大きい場合の分割・yielding・backpressure。
- Collectionと複数Selection/ViewModel Property間の同期、cross-model transactionやrollback。
- 複数入力欄をまたぐ業務validation、フォーム全体のcommit保留、エラー表示UX（ViewModel／アプリケーション側で決める）。
- change batchのserialize/replay、undo/redo、remote synchronization。

## 承認記録

- この全件事前検証はCollection Model整合性の検証であり、各入力欄の値／業務validationをまとめて待たせるものではない。
- 独立入力欄の有効なcommitは個別にModelへ反映でき、別欄のinvalid値により保留されない。

- 一つのCollection batchは事前検証後にall-or-nothingで適用する。
- 失敗時はCollection/Selection/revision/公開通知をすべて変更しない。
- 成功時は一つのrevisionをcommitし、中間stateを通知しない。
- 削除itemのSelection reconciliationを外部callbackより前に完了する。
- これは一般UI transactionではなく、cross-property/model rollbackを含まない。

利用者承認（2026-09-27）。上記1～6を上位判断として採用する。特にCollectionBatchの原子性はCollection Model整合性に限り、独立入力欄のvalidation/commitを相互に待たせない。

本判断はDD-00を承認するものではない。DD-00は未承認のまま維持し、DD-01以降へ進まない。
