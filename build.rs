use std::{env, error::Error, fs};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = get_env("OUT_DIR")?;

    generate_from_xml_file(
        &get_env("FPRINT_DEVICE_XML")?,
        &format!("{out_dir}/dbus-fprint-device.rs"),
    )?;
    generate_from_xml_file(
        &get_env("FPRINT_MANAGER_XML")?,
        &format!("{out_dir}/dbus-fprint-manager.rs"),
    )?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn generate_from_xml_file(from_xml: &str, to_rs: &str) -> Result<(), Box<dyn Error>> {
    let opts = dbus_codegen::GenOpts {
        methodtype: None,
        connectiontype: dbus_codegen::ConnectionType::Nonblock,
        skipprefix: Some("net.reactivated.Fprint.".into()),
        ..Default::default()
    };
    let xml = fs::read_to_string(from_xml)?;
    let code = dbus_codegen::generate(&xml, &opts)?;

    fs::write(to_rs, code)?;

    Ok(())
}

fn get_env(var: &str) -> Result<String, Box<dyn Error>> {
    Ok(env::var_os(var)
        .ok_or_else(|| format!("{var} not set"))?
        .to_string_lossy()
        .into())
}
