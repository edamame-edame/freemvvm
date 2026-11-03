# Scope: C ABI Input event callback and borrowed payload

- カテゴリ: 実装記憶 / C ABI / UI Input
- 対象: `UNIT-ABI-022`、`.docs/13_detail_design/runtime-objects/abi.md` v1.6

## 学んだこと

- Feature 9は、subscriptionと5種類のInput callback/payload APIが揃ってから有効にする。kind値とevent mask bitは異なり、maskは`1 << (kind - 1)`。
- Input event handleはcallback中だけ有効で、event用retain/releaseは提供しない。callback中のWidget handleは明示retainでき、callback終了時に借用tokenだけを解放する。借用Widget handleに対するreleaseは拒否する。
- UTF-8 Text getterはnull buffer・capacity 0で必要byte数を返せる。buffer不足時は部分書き込みをせず必要長を返し、出力長は不正引数時も先に0へ初期化する。
- この環境でCMakeの`dev` presetが要求するNinjaを利用できない場合、Visual Studio generatorでC/C++ header probeとABI smokeを構成・ビルド・実行できる。これは同じCMake targetのローカル代替検証であり、CIのNinja preset成功記録とは区別する。
