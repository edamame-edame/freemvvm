# MVVM UIライブラリ CollectionModel／Gridに関する上位判断案

- 状態: BD-33承認済み（2026-09-27）
- 対応: Collectionを値型・UI treeと分ける境界、Gridの大量データ表示と編集の責務
- 根拠: 要求仕様第4・6節、BD-01、R2～R4、A5～A8、C1～C4。DD-00は未承認

## 既存の承認済み契約

- A5／BD-01は初期公開Property値型にBOOL、I64、UTF8、F64、独自DECIMALを定め、CollectionをProperty value kindに含めない。
- 要求仕様第4節はCollectionModel、SelectionModel、差分通知、virtualizationをCore機能として要求する。List、ComboBox、Tree、Tableに共通するModel/Selection契約を個別Controlより先に設計する。
- BD-04～BD-06はWidget treeの所有とLayout境界を定めたが、データ件数とWidget数を一対一にする判断はしていない。
- R2～R4はCore object操作をUI threadに制限する。workerからのUI更新はRuntime postを使う。
- `value-types-upper-design.md`の検討案はstable item/row ID、bulk visible-range取得、差分通知、Grid cell virtualization、編集状態の一時保持を提案しているが、BD-01では未承認だった。

## BD-33 承認済み判断

1. **CollectionModelとSelectionModelはCoreの独立objectとし、Property value kindにもWidget treeにも含めない。** ViewModelまたはhost-language adapterはmodel handleをControlへ接続し、任意言語のobject graphをC ABI越しに直接保持させない。要素データには宣言済みschemaと初期公開値型を使い、custom object値を要求しない。
2. **各論理項目はindexと別の安定IDで識別する。** Collectionの順序変更・挿入・削除でIDを使い回さない。Table/Gridの列も安定したcolumn IDで識別し、SelectionModelと編集中のcellはindexではなくitem ID／column IDを参照する。
3. **更新は個別セルBindingの全件再配信ではなく構造化change setで伝える。** ordered collectionのinsert/remove/move/item updateとresetを区別し、連続更新を一つのchange batchへまとめられる。workerはmodelを直接変更せず、UI threadのpost callbackでbatchを適用する。
4. **大量表示Controlはvisible rangeとoverscan分だけView/Cellを生成・再利用する。** Gridの全dataset cellに恒久WidgetまたはProperty/TwoWay Bindingを作らない。Modelから可視範囲をまとめてsnapshot取得し、編集対象だけ一時的なeditor stateと同期Validatorを保持する。確定時にitem ID／column IDを使ってmodelへ反映する。
5. **Selectionはstable item IDを参照し、Collection changeに合わせて更新する。** 選択済みitemの削除時にそのIDをSelectionから外す。編集中item/columnの削除・移動・更新競合時のcommit/cancel規則はControl詳細設計に残す。
6. **CollectionModelとSelectionModelの共通契約をList/ComboBox/Table系Controlより先に設計する。** 初期版の具体容量、virtualization threshold、差分API ABI、高水準adapterの構文、Tree階層modelはこの判断では固定しない。要求仕様の製品機能からCollection/Selectionを外す判断でもない。

## Grid利用時の流れ

データ側は行データと変更batchをmodelへ渡す。Gridは現在のviewportから必要な行・列だけを読み、画面に出るcellを少数のWidgetとして再利用する。利用者は全cellの生成・Binding・破棄を個別に管理せず、collection modelとcolumn定義を提供する。

編集中の値は可視cellの再利用に巻き込まれないようitem ID／column IDに結び付ける。編集対象が消えた場合の扱いは上位ではID参照を要求し、編集値の維持・拒否表示・commit可否はGrid/TextBox詳細設計で定める。

## 理由とトレードオフ

- Model item数をWidget/Property数へ展開しないため、多数行・多数cellでもUI tree、Binding通知、C ABI callbackがdataset全体に比例して膨らみにくい。
- stable IDを使うと、並べ替えや挿入後も選択と編集中対象をindexのずれから区別できる。
- change setとvisible-range snapshotにより、利用者は差分通知や可視cellのrecyclingを毎回手作業で組み立てずに済む。
- 独立CollectionModelは任意の既存言語collectionをそのまま渡す方式よりadapter実装が必要だが、ABI越しの所有権と変更順を制御しやすい。
- 変更batchの競合や編集中行削除は単純なObservable Listより規則が増えるため、具体API/UXはTable/List実装前に別途設計する。

## この判断では決めないこと

- row item/column schemaの具体型、property key、ID生成方法、ID再利用禁止期間。
- change setのC ABI、callback形式、batch原子性、通知の再入・順序規則。
- 可視範囲取得関数、cache、overscan数、recycling pool、virtualization性能目標。
- 選択の単一/複数、range selection、focus、keyboard navigation、accessibility semantics。
- 編集中に行/列が削除・並替・外部更新された場合の確定、cancel、入力保持とerror表示。
- TreeView向けhierarchical model、filter/sort/group view、paging/remote data source protocol。
- 4言語用adapter/API名・async enumeration・lifetimeとdispose具体規則。

## 承認記録

- 利用者承認（2026-09-27）。上記1～6を上位判断として採用する。
- CollectionModel/SelectionModelをProperty value kind、Widget treeから独立したCore objectとする。
- stable item ID／column IDと構造化change setでSelectionおよび編集中targetを追跡する。
- Gridはvisible range＋overscanだけをWidget化し、dataset全体へのper-cell恒久Bindingを作らない。
- UI model更新はUI threadで行い、workerからはpostを使う。
- ABI/API、性能値、Tree model、edit conflict規則は詳細設計・個別Control判断に残す。

item schema、change set ABI、virtualization閾値、Tree階層拡張、edit conflict、selection mode等は別途判断または詳細設計とする。DD-00は未承認のため、DD-01以降へ進まない.
