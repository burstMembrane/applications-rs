use applications::{AppInfo, AppInfoContext};

fn main() {
    let mut ctx = AppInfoContext::new();
    ctx.refresh_apps().unwrap(); // must refresh apps before getting them

    let running_apps = ctx.get_running_apps();
    println!("Running Apps: {:#?}", running_apps);

    // dump to json
    let json = serde_json::to_string_pretty(&running_apps).unwrap();
    println!("{}", json);
}
