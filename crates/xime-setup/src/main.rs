//! 设置应用入口。
//!
//! 三个平台共用这个二进制：产品差异（显示名、标识、主页、版本）走命令行参数，
//! 未传则按 Xime 默认值跑。用法见 `xime-setup --help`。

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print!("{}", xime_setup_lib::profile::HELP);
        return Ok(());
    }

    let meta = match xime_setup_lib::profile::metadata_from_args(args) {
        Ok(meta) => meta,
        Err(err) => {
            eprintln!("{err}\n\n{}", xime_setup_lib::profile::HELP);
            std::process::exit(2);
        }
    };

    xime_setup_lib::run_with(meta)
}
