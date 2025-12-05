pub const SAMPLE_MAIN: &str = r#"import { log } from "log"
import { time } from "time"
import { fs } from "fs"
import { helper } from "./support/log_helper"

async fn main() {
    log.info("booting VoltTS prototype (async demo)")
    time.now()

    await time.sleep(50)
    log.warn("demo sleep (await)")

    fs.writeFile("tmp_fs.txt", "hello fs runtime")
    fs.readFile("tmp_fs.txt")

    helper()

    log.info("demo done")
    log.error("demo complete")
}
"#;

pub const SAMPLE_HELPER: &str = r#"import { log } from "log"
import { time } from "time"

export fn helper() {
    log.info("helper start")
    time.sleep(10)
    log.warn("helper end")
}
"#;
