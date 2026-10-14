//! En Windows, mete el ícono y los datos del programa en el ejecutable, para
//! que el Explorador y la barra de tareas lo muestren como IEEStudio.

fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon/ieestudio.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icon/ieestudio.ico")
            .set("ProductName", "IEEStudio")
            .set(
                "FileDescription",
                "IEEStudio: cálculo de instalaciones eléctricas",
            )
            .set("CompanyName", "Proyecto IEEStudio")
            .set("LegalCopyright", "MIT o Apache-2.0");
        if let Err(e) = res.compile() {
            // Sin las herramientas de recursos el programa funciona igual,
            // solo sin ícono; no debe impedir compilar.
            println!("cargo:warning=No se pudo agregar el ícono: {e}");
        }
    }
}
