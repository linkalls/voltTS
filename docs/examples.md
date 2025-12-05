# Examples

VoltTS の最小サンプルを `examples/` に配置しました。`voltts run` で C 生成とビルドを行い、そのまま実行できます。

## 収録サンプル
- `examples/hello.vts`: 標準出力と `log.info` を呼ぶシンプルな Hello World（`main` は `: int` を明示し、最後に `return 0`）。
- `examples/hello_void.vts`: `main` を `: void` で宣言し、戻り値なしで動作する Hello World。
- `examples/fs_echo.vts`: `fs.writeFile` / `fs.readFile` を使ってファイルに書き込み、読み戻した内容を標準出力に出す I/O デモ。
- `examples/std_log_time.vts`: `log` と `time` を組み合わせたランタイム呼び出しの最小例（`await time.now` と `time.sleep`）。
- `examples/std_fs_basic.vts`: `fs` の `writeFile` / `readFile` を `await` 付きで連続呼び出しするランタイム例。
- `examples/control_flow.vts`: `if` / `for` / `while` のインラインブロック構文をまとめて実行するスモークテスト用サンプル。

## 実行方法
- Hello World: `cargo run -- run examples/hello.vts`
- ファイル I/O: `cargo run -- run examples/fs_echo.vts`

> メモ: `fs_echo` は `dist/examples/tmp_fs_demo.txt` にファイルを生成します。必要に応じて削除してください。
