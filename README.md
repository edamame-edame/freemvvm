# FreeMvvm

FreeMvvm は、Rust Core と versioned C ABI を基盤に、C++ / C# / Rust / Python から共通の MVVM UI を構築するためのライブラリです。

現在は Runtime、Property、Binding、Async Command などの実装基盤を整備しています。UI controls、view tree、layout、input、rendering、platform integration は未完成のため、UI ライブラリとして利用できる段階には達していません。公開リリースおよびパッケージ配布は行っていません。

- [開発環境と実行コマンド](DEVELOPMENT.md)
- [要求仕様](.docs/11_requirements/reqirements-specification.md)
- [基本設計](.docs/12_design/README.md)
- [詳細設計](.docs/13_detail_design/README.md)
- [テスト設計](.docs/32_unit_test/README.md)

ローカルで生成する native library、managed assembly、wheel は開発・検証用です。公開済みまたはリリース済みの成果物を示すものではありません。
