use applications::{AppInfo, AppInfoContext};

fn main() {
    let mut ctx = AppInfoContext::new();

    let running_apps = ctx.get_running_apps();

    // dump to json
    let json = serde_json::to_string_pretty(&running_apps).unwrap();
    println!("{}", json);

    // do it again
    let running_apps = ctx.get_running_apps();
    println!("{:#?}", running_apps);
}
