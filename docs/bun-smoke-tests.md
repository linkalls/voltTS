# Bun Smoke Tests for `voltts`

実際に `bun test` で CLI スモークを回すためのメモです。Node.js ではなく Bun を使うという方針を具体化し、Rust テストに加えて JS ベースで CLI を叩く手順を残しています。

## 目的
- Bun ランタイムで `voltts` の基本的なヘルプ出力を検証する。
- JS 側のスモークを書くときは **Node.js ではなく Bun** を使う、という開発方針をそのまま反映する。
- Rust 側の統合テストに加え、外部からの CLI 呼び出しをもう一段簡単に再現できるようにする。

## 前提
- Rust toolchain と C コンパイラがインストールされていること（`cargo run -- --help` が動く状態）。
- Bun がインストール済みであること。macOS/Linux の場合は `curl -fsSL https://bun.sh/install | bash` などで導入し、`bun --version` で確認する。

## 実行方法
`tests/bun/cli_smoke.test.ts` が Bun 用の最小スモークです。`voltts --help` の成否と出力を確認します。

```sh
bun test tests/bun/cli_smoke.test.ts --timeout 120000
```

> `cargo run` を内部で呼ぶため、初回は依存ビルドで時間がかかる場合があります。`--timeout` を十分大きめに設定してください。

## テスト内容の概要
- `Bun.spawnSync` で `cargo run -- --help` を起動
- 正常終了コードをチェック
- 標準出力に `voltts` と `USAGE` が含まれることを確認

## 今後の拡張案
- `examples/` 以下の `.vts` を `voltts run` で実行するケースを追加
- `voltts test` の統合テスト（Rust 側での実行が実装されたら、Bun 側でも CLI 経路のみ検証する）
- `bun test` での並列実行を考慮した一時ディレクトリの生成・掃除

## 既知の注意点
- Bun が未インストールの場合、このスモークは実行できません。CI に組み込むときは Bun のセットアップステップを追加してください。
- `cargo run` を呼ぶため、C コンパイラと Rust toolchain がない環境では失敗します。Docker ベースの CI の場合は `build-essential`/`clang` などが必要です。
