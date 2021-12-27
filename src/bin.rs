use openfare_js_lib;
use openfare_lib::extension::FromLib;

fn main() {
    let env = env_logger::Env::new().filter_or("OPENFARE_JS_LOG", "off");
    env_logger::Builder::from_env(env).init();

    let mut extension = openfare_js_lib::JsExtension::new();
    openfare_lib::extension::commands::run(&mut extension).unwrap();
}
