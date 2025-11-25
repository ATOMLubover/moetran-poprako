// Windows 平台 release 模式下避免出现额外控制台窗口（必须保留）
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    moetran_native_lib::run()
}
