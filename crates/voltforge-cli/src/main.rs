//! Línea de comandos de VoltForge.

use std::process::ExitCode;

use voltforge_project::{calculate, example, PanelResult, Project, ProjectResult, RunResult};

const HELP: &str = "\
VoltForge: cálculo de instalaciones eléctricas de baja tensión

Uso:
  voltforge ejemplo [archivo.vfp]     escribe un proyecto de ejemplo
  voltforge calcular <archivo.vfp>    calcula un proyecto y muestra el cuadro de cargas
  voltforge calcular <archivo> --json  entrega el resultado completo en JSON
  voltforge memoria <archivo.vfp> [salida.pdf]
                                       escribe la memoria de cálculo en PDF
  voltforge ayuda                      muestra esta ayuda
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.first().map(String::as_str) {
        Some("ejemplo") => write_example(args.get(1).map(String::as_str)),
        Some("calcular") => match args.get(1) {
            Some(path) => compute(path, args.iter().any(|a| a == "--json")),
            None => Err("falta el archivo del proyecto".to_string()),
        },
        Some("memoria") => match args.get(1) {
            Some(path) => report(path, args.get(2).map(String::as_str)),
            None => Err("falta el archivo del proyecto".to_string()),
        },
        Some("ayuda" | "--help" | "-h") | None => {
            print!("{HELP}");
            Ok(())
        }
        Some(other) => Err(format!("comando desconocido: «{other}»\n\n{HELP}")),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn write_example(path: Option<&str>) -> Result<(), String> {
    let json = example::house().to_json();
    match path {
        Some(path) => {
            std::fs::write(path, json).map_err(|e| format!("no se pudo escribir {path}: {e}"))?;
            println!("Proyecto de ejemplo escrito en {path}");
        }
        None => println!("{json}"),
    }
    Ok(())
}

fn compute(path: &str, as_json: bool) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("no se pudo leer {path}: {e}"))?;
    let project = Project::from_json(&text)?;
    let result = calculate(&project)?;
    if as_json {
        let json = serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?;
        println!("{json}");
    } else {
        print_report(&result);
    }
    Ok(())
}

fn report(path: &str, output: Option<&str>) -> Result<(), String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("no se pudo leer {path}: {e}"))?;
    let project = Project::from_json(&text)?;
    let result = calculate(&project)?;
    let pdf = voltforge_reports::project_pdf(&project, &result)?;
    let output = match output {
        Some(output) => std::path::PathBuf::from(output),
        None => std::path::Path::new(path).with_extension("pdf"),
    };
    std::fs::write(&output, pdf)
        .map_err(|e| format!("no se pudo escribir {}: {e}", output.display()))?;
    println!("Memoria de cálculo escrita en {}", output.display());
    Ok(())
}

fn print_report(result: &ProjectResult) {
    println!("{}", result.project);
    println!("Perfil: {}", result.profile);
    for panel in &result.panels {
        print_panel(panel);
    }
    println!(
        "\nResumen: {} cumplen, {} no cumplen, {} no evaluados",
        result.complies, result.fails, result.not_evaluated
    );
    for warning in &result.warnings {
        println!("Aviso: {warning}");
    }
    println!();
    for (level, _, detail) in result.verdict.lines("la norma") {
        println!("{level}: {detail}");
    }
}

fn print_panel(panel: &PanelResult) {
    println!("\n{}", panel.name);
    println!(
        "  Carga conectada {:.0} W, calculada {:.0} VA = {:.0} W (factor resultante {:.2}), fp {:.2}",
        panel.connected_w, panel.demanded_va, panel.demanded_w, panel.demand_factor, panel.power_factor
    );
    for line in &panel.demand {
        println!(
            "    {:<30} {:>10.0} VA → {:>10.0} VA  {}",
            line.label, line.connected_va, line.demanded_va, line.detail
        );
    }
    let [a, b, c] = panel.phase_va;
    match panel.imbalance_percent {
        Some(imbalance) => {
            println!("  Fases: A {a:.0} VA, B {b:.0} VA, C {c:.0} VA, desbalance {imbalance:.1} %")
        }
        None => println!("  Fases: sin asignar en todos los circuitos"),
    }
    println!("\n  ID     Descripción                      W     I A   L m  Calibre Tierra Tubo   ITM          e %   Σe %  Estado");
    print_row(&panel.feeder, true);
    for circuit in &panel.circuits {
        print_row(circuit, false);
    }
    for item in std::iter::once(&panel.feeder).chain(&panel.circuits) {
        for check in item
            .checks
            .iter()
            .filter(|c| c.status != voltforge_project::Status::Complies)
        {
            println!(
                "  {} {}: {} ({})",
                item.id,
                check.name,
                check.status.label().to_lowercase(),
                check.detail
            );
        }
        if let Some(run) = &item.run {
            if let (true, Some(recommended)) = (run.forced, &run.recommended_size) {
                if recommended != &run.size {
                    println!(
                        "  {} Recomendación: calibre {} en lugar del {} fijado",
                        item.id, recommended.0, run.size.0
                    );
                }
            }
        }
    }
}

fn print_row(item: &RunResult, feeder: bool) {
    let id = if feeder { "ALIM" } else { item.id.as_str() };
    let name: String = item.name.chars().take(25).collect();
    match &item.run {
        Some(run) => {
            let size = if run.conductors_per_phase > 1 {
                format!("{}×{}", run.conductors_per_phase, run.size.0)
            } else {
                run.size.0.clone()
            };
            let conduit = run
                .conduit
                .as_ref()
                .map_or("-".to_string(), |c| format!("{} mm", c.metric_designator));
            println!(
                "  {:<7}{:<26}{:>8.0}{:>8.2}{:>6.0}  {:<8}{:<7}{:<7}{:<9}{:>7.2}{:>7.2}  {}",
                id,
                name,
                item.load_w,
                run.nominal_a,
                run.length_m,
                size,
                run.grounding_size.0,
                conduit,
                format!("{}P-{:.0}", run.poles, run.breaker_a),
                run.drop_percent,
                run.cumulative_drop_percent,
                item.status.label()
            );
        }
        None => println!(
            "  {:<7}{:<26}{:>8.0}  {}",
            id,
            name,
            item.load_w,
            item.status.label()
        ),
    }
}
