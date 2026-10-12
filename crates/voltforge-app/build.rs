//! En Windows, mete el ícono y los datos del programa en el ejecutable, para
//! que el Explorador y la barra de tareas lo muestren como VoltForge.

fn main() {
    println!("cargo:rerun-if-changed=../../assets/icon/voltforge.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("../../assets/icon/voltforge.ico")
            .set("ProductName", "VoltForge")
            .set(
                "FileDescription",
                "VoltForge: cálculo de instalaciones eléctricas",
            )
            .set("CompanyName", "Proyecto VoltForge")
            .set("LegalCopyright", "MIT o Apache-2.0");
        if let Err(e) = res.compile() {
            // Sin las herramientas de recursos el programa funciona igual,
            // solo sin ícono; no debe impedir compilar.
            println!("cargo:warning=No se pudo agregar el ícono: {e}");
        }
    }
}
