uniffi::setup_scaffolding!();

#[uniffi::export]
fn say_hello(name: String) -> String {
    format!("Привет от qqqqwuzzt, {}!", name)
}