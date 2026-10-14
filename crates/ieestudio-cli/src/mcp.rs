//! Servidor MCP (Model Context Protocol) por entrada y salida estándar.
//!
//! Un agente de IA lanza `ieestudio mcp` y le habla en JSON-RPC 2.0, un
//! mensaje por línea. Las herramientas exponen el mismo motor que la
//! aplicación: calcular un proyecto, el cálculo rápido de un circuito, la
//! memoria en PDF, el cuadro de cargas en Excel, el unifilar en DXF y las
//! reglas normativas con su cláusula. El servidor no aprueba nada: cada
//! resultado lleva el veredicto en tres niveles y la aprobación profesional
//! siempre queda pendiente.

use std::io::{BufRead, Write};

use ieestudio_project::quick::{quick_circuit, QuickInput};
use ieestudio_project::{calculate, example, Project};
use ieestudio_rules::{Profile, Rule};
use serde_json::{json, Map, Value};

/// Versiones del protocolo que entiende el servidor, la más nueva primero.
const PROTOCOL_VERSIONS: [&str; 3] = ["2025-06-18", "2025-03-26", "2024-11-05"];

const INSTRUCTIONS: &str = "IEEStudio calcula instalaciones eléctricas de baja tensión con la NOM-001-SEDE-2012. \
Un proyecto es un JSON (formato .iees): pide `proyecto_ejemplo` para ver la estructura, modifícalo y \
calcúlalo con `calcular_proyecto`. Cada tramo queda como Complies (cumple), Fails (no cumple) o \
NotEvaluated (falta un dato o la regla no está implementada); nunca supongas que un tramo no evaluado cumple. \
El programa no otorga aprobación profesional: la da quien firma el proyecto.";

/// Atiende la entrada estándar hasta que se cierra.
pub fn serve() -> Result<(), String> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let line = line.map_err(|e| e.to_string())?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match serde_json::from_str::<Value>(&line) {
            Ok(message) => handle(&message),
            Err(e) => Some(error(Value::Null, -32700, &format!("JSON inválido: {e}"))),
        };
        if let Some(reply) = reply {
            writeln!(stdout, "{reply}").map_err(|e| e.to_string())?;
            stdout.flush().map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn error(id: Value, code: i64, message: &str) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "error": {"code": code, "message": message}})
}

fn ok(id: Value, result: Value) -> Value {
    json!({"jsonrpc": "2.0", "id": id, "result": result})
}

/// Respuesta a un mensaje. `None` para las notificaciones.
pub fn handle(message: &Value) -> Option<Value> {
    let id = message.get("id").cloned();
    let method = message.get("method").and_then(Value::as_str).unwrap_or("");
    let params = message.get("params").cloned().unwrap_or(Value::Null);
    let id = id?; // una notificación no lleva id ni respuesta
    Some(match method {
        "initialize" => {
            let asked = params
                .get("protocolVersion")
                .and_then(Value::as_str)
                .unwrap_or(PROTOCOL_VERSIONS[0]);
            let version = PROTOCOL_VERSIONS
                .iter()
                .find(|v| **v == asked)
                .unwrap_or(&PROTOCOL_VERSIONS[0]);
            ok(
                id,
                json!({
                    "protocolVersion": version,
                    "capabilities": {"tools": {}},
                    "serverInfo": {"name": "ieestudio", "version": env!("CARGO_PKG_VERSION")},
                    "instructions": INSTRUCTIONS,
                }),
            )
        }
        "ping" => ok(id, json!({})),
        "tools/list" => ok(id, json!({"tools": tools()})),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let result = match call(name, &args) {
                Ok(text) => json!({"content": [{"type": "text", "text": text}], "isError": false}),
                Err(e) => json!({"content": [{"type": "text", "text": e}], "isError": true}),
            };
            ok(id, result)
        }
        _ => error(id, -32601, &format!("método desconocido: {method}")),
    })
}

fn project_source() -> Value {
    json!({
        "proyecto": {"type": "object", "description": "Proyecto completo en el formato .iees (JSON)."},
        "ruta": {"type": "string", "description": "Ruta de un archivo .iees. Se usa si no se da `proyecto`."}
    })
}

fn tools() -> Value {
    json!([
        {
            "name": "proyecto_ejemplo",
            "description": "Devuelve un proyecto de ejemplo (casa habitación) en el formato .iees, para conocer la estructura: fuente, tableros, alimentadores, áreas y circuitos con sus cargas.",
            "inputSchema": {"type": "object", "properties": {}}
        },
        {
            "name": "calcular_proyecto",
            "description": "Calcula un proyecto completo: carga de cada alimentador según el artículo 220, corriente, interruptor, conductor, puesta a tierra, tubería, caída de tensión, cortocircuito y el veredicto en tres niveles. Sin `detalle` omite el procedimiento paso a paso.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "proyecto": project_source()["proyecto"],
                    "ruta": project_source()["ruta"],
                    "detalle": {"type": "boolean", "description": "Incluir el procedimiento paso a paso de cada tramo."}
                }
            }
        },
        {
            "name": "calculo_rapido",
            "description": "Calcula un solo circuito sin proyecto y verifica el calibre dado. Los campos que no se den toman los del ejemplo: alimentador trifásico de 61.7 kW, 220Y/127 V, 95 m, 3/0 AWG de cobre THW en EMT.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "entrada": {
                        "type": "object",
                        "description": "Campos de QuickInput: system {line_to_line_v, line_to_neutral_v}, kind (LineToNeutral, LineToLine, ThreePhase), load_in_amperes, power_w, current_a, power_factor, continuous, length_m, size (\"12\", \"3/0\", \"250\"), conductors_per_phase, material (Copper, Aluminum), insulation (Thw, Thhn, Thwn2, Xhhw2…), has_neutral, conduit (Emt, Imc, Rmc, Pvc40, Pvc80), ambient_c, limit_percent, resistance_temperature_c, motor {hp, motor_type, nameplate_a, service_factor_marked}."
                    }
                }
            }
        },
        {
            "name": "escribir_memoria",
            "description": "Escribe la memoria de cálculo del proyecto en PDF.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "proyecto": project_source()["proyecto"],
                    "ruta": project_source()["ruta"],
                    "salida": {"type": "string", "description": "Ruta del PDF que se escribe."}
                },
                "required": ["salida"]
            }
        },
        {
            "name": "exportar_excel",
            "description": "Escribe el cuadro de cargas del proyecto en un libro de Excel: tableros, circuitos y resultados.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "proyecto": project_source()["proyecto"],
                    "ruta": project_source()["ruta"],
                    "salida": {"type": "string", "description": "Ruta del .xlsx que se escribe."}
                },
                "required": ["salida"]
            }
        },
        {
            "name": "exportar_dxf",
            "description": "Escribe el diagrama unifilar del proyecto en DXF.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "proyecto": project_source()["proyecto"],
                    "ruta": project_source()["ruta"],
                    "salida": {"type": "string", "description": "Ruta del .dxf que se escribe."}
                },
                "required": ["salida"]
            }
        },
        {
            "name": "reglas_normativas",
            "description": "Lista las reglas que aplica el motor con su cláusula de la NOM-001-SEDE-2012 y una descripción propia.",
            "inputSchema": {"type": "object", "properties": {}}
        }
    ])
}

fn load_project(args: &Value) -> Result<Project, String> {
    match (
        args.get("proyecto"),
        args.get("ruta").and_then(Value::as_str),
    ) {
        (Some(p), _) if !p.is_null() => Project::from_json(&p.to_string()),
        (_, Some(path)) => {
            let text = std::fs::read_to_string(path)
                .map_err(|e| format!("no se pudo leer {path}: {e}"))?;
            Project::from_json(&text)
        }
        _ => Err("falta `proyecto` o `ruta`".into()),
    }
}

fn output(args: &Value) -> Result<&str, String> {
    args.get("salida")
        .and_then(Value::as_str)
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| "falta `salida`".to_string())
}

/// Quita el procedimiento de cada tramo para una respuesta compacta.
fn strip_steps(value: &mut Value) {
    match value {
        Value::Object(map) => {
            map.remove("steps");
            for v in map.values_mut() {
                strip_steps(v);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(strip_steps),
        _ => {}
    }
}

/// Mezcla los campos dados sobre los de omisión.
fn merge(base: &mut Value, patch: &Value) {
    match (base, patch) {
        (Value::Object(b), Value::Object(p)) => {
            for (k, v) in p {
                match b.get_mut(k) {
                    Some(slot) if slot.is_object() && v.is_object() => merge(slot, v),
                    _ => {
                        b.insert(k.clone(), v.clone());
                    }
                }
            }
        }
        (b, p) => *b = p.clone(),
    }
}

fn call(name: &str, args: &Value) -> Result<String, String> {
    let pretty = |v: &Value| serde_json::to_string_pretty(v).map_err(|e| e.to_string());
    match name {
        "proyecto_ejemplo" => Ok(example::house().to_json()),
        "calcular_proyecto" => {
            let project = load_project(args)?;
            let result = calculate(&project)?;
            let mut value = serde_json::to_value(&result).map_err(|e| e.to_string())?;
            if !args
                .get("detalle")
                .and_then(Value::as_bool)
                .unwrap_or(false)
            {
                strip_steps(&mut value);
            }
            pretty(&value)
        }
        "calculo_rapido" => {
            let mut input =
                serde_json::to_value(QuickInput::default()).map_err(|e| e.to_string())?;
            if let Some(patch) = args.get("entrada") {
                merge(&mut input, patch);
            }
            let input: QuickInput =
                serde_json::from_value(input).map_err(|e| format!("entrada inválida: {e}"))?;
            let result = quick_circuit(Profile::nom_001_sede_2012(), &input)?;
            let value = json!({
                "entrada": input,
                "resultado": result,
                "veredicto": result.verdict(),
            });
            pretty(&value)
        }
        "escribir_memoria" => {
            let project = load_project(args)?;
            let result = calculate(&project)?;
            let pdf = ieestudio_reports::project_pdf(&project, &result)?;
            let out = output(args)?;
            std::fs::write(out, pdf).map_err(|e| format!("no se pudo escribir {out}: {e}"))?;
            Ok(format!("Memoria de cálculo escrita en {out}"))
        }
        "exportar_excel" => {
            let project = load_project(args)?;
            let result = calculate(&project).ok();
            let bytes = ieestudio_exchange::export_xlsx(&project, result.as_ref())?;
            let out = output(args)?;
            std::fs::write(out, bytes).map_err(|e| format!("no se pudo escribir {out}: {e}"))?;
            Ok(format!("Cuadro de cargas escrito en {out}"))
        }
        "exportar_dxf" => {
            let project = load_project(args)?;
            let result = calculate(&project)?;
            let bytes = ieestudio_diagrams::single_line(&project, &result).to_dxf();
            let out = output(args)?;
            std::fs::write(out, bytes).map_err(|e| format!("no se pudo escribir {out}: {e}"))?;
            Ok(format!("Diagrama unifilar escrito en {out}"))
        }
        "reglas_normativas" => {
            let profile = Profile::nom_001_sede_2012();
            let rules: Vec<Value> = Rule::ALL
                .iter()
                .map(|&r| {
                    let mut m = Map::new();
                    m.insert("regla".into(), json!(r.id()));
                    m.insert("clausula".into(), json!(profile.clause(r)));
                    m.insert(
                        "descripcion".into(),
                        json!(profile.rule_description(r).unwrap_or_default()),
                    );
                    Value::Object(m)
                })
                .collect();
            pretty(&json!({"norma": profile.document.code, "reglas": rules}))
        }
        other => Err(format!("herramienta desconocida: {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(id: i64, method: &str, params: Value) -> Value {
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
    }

    fn call_tool(name: &str, args: Value) -> (bool, String) {
        let reply = handle(&request(
            3,
            "tools/call",
            json!({"name": name, "arguments": args}),
        ))
        .unwrap();
        let result = &reply["result"];
        (
            result["isError"].as_bool().unwrap(),
            result["content"][0]["text"].as_str().unwrap().to_string(),
        )
    }

    #[test]
    fn handshake_and_tool_list() {
        let reply = handle(&request(
            1,
            "initialize",
            json!({"protocolVersion": "2025-03-26", "capabilities": {}, "clientInfo": {"name": "prueba"}}),
        ))
        .unwrap();
        assert_eq!(reply["result"]["protocolVersion"], "2025-03-26");
        assert_eq!(reply["result"]["serverInfo"]["name"], "ieestudio");
        // Las notificaciones no llevan respuesta.
        assert!(
            handle(&json!({"jsonrpc": "2.0", "method": "notifications/initialized"})).is_none()
        );
        let tools = handle(&request(2, "tools/list", json!({}))).unwrap();
        let names: Vec<&str> = tools["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|t| t["name"].as_str().unwrap())
            .collect();
        assert!(names.contains(&"calcular_proyecto"));
        assert!(names.contains(&"calculo_rapido"));
        let unknown = handle(&request(9, "no/existe", json!({}))).unwrap();
        assert_eq!(unknown["error"]["code"], -32601);
    }

    #[test]
    fn project_and_quick_tools() {
        let (error, text) = call_tool("proyecto_ejemplo", json!({}));
        assert!(!error);
        let project: Value = serde_json::from_str(&text).unwrap();
        let (error, text) = call_tool("calcular_proyecto", json!({"proyecto": project}));
        assert!(!error, "{text}");
        let result: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(result["panels"][0]["feeder"]["status"], "Complies");
        assert!(!text.contains("\"steps\""));
        // Solo los campos que cambian; el resto toma los de omisión.
        let (error, text) = call_tool(
            "calculo_rapido",
            json!({"entrada": {"size": "300", "length_m": 50.0}}),
        );
        assert!(!error, "{text}");
        let quick: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(quick["entrada"]["size"], "300");
        let (error, text) = call_tool("calculo_rapido", json!({"entrada": {"kind": "Cuatro"}}));
        assert!(error);
        assert!(text.contains("entrada inválida"));
        let (error, _) = call_tool("calcular_proyecto", json!({}));
        assert!(error);
        let (error, text) = call_tool("reglas_normativas", json!({}));
        assert!(!error);
        assert!(text.contains("110-9"));
    }
}
