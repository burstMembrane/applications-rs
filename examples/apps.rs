use applications::{AppInfo, AppInfoContext};

fn main() {
    let mut ctx = AppInfoContext::new();

    let apps = ctx.get_all_apps();
    println!("Apps: {:#?}", apps);

    let frontmost_app = ctx.get_frontmost_application().unwrap();
    println!("Frontmost App: {:#?}", frontmost_app);

    let running_apps = ctx.get_running_apps();
    println!("Running Apps: {:#?}", running_apps);
}
