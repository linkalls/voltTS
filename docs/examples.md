# Examples

VoltTS の最小サンプルを `examples/` に配置しました。`voltts run` で C 生成とビルドを行い、そのまま実行できます。

## 収録サンプル
- `examples/hello.vts`: 標準出力と `log.info` を呼ぶシンプルな Hello World。
- `examples/fs_echo.vts`: `fs.writeFile` / `fs.readFile` を使ってファイルに書き込み、読み戻した内容を標準出力に出す I/O デモ。

## 実行方法
- Hello World: `cargo run -- run examples/hello.vts`
- ファイル I/O: `cargo run -- run examples/fs_echo.vts`

> メモ: `fs_echo` は `dist/examples/tmp_fs_demo.txt` にファイルを生成します。必要に応じて削除してください。
