//! OCEL2 Format Support
//!
//! Support for OCEL2 (Object-Centric Event Log 2.0) format.

use crate::ocpm::ObjectCentricEventLog;
use anyhow::Result;
use std::path::Path;

/// Read OCEL2 file (auto-detect format)
///
/// Reads an OCEL2 file, detecting the format from extension.
pub fn read_ocel2(path: &Path) -> Result<ObjectCentricEventLog> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("xml") => read_ocel2_xml(path),
        Some("json") => read_ocel2_json(path),
        Some("sqlite") | Some("db") => read_ocel2_sqlite(path),
        _ => Err(anyhow::anyhow!("Unsupported OCEL2 format")),
    }
}

/// Read OCEL2 XML format
///
/// Reads an OCEL2 file in XML format.
pub fn read_ocel2_xml(path: &Path) -> Result<ObjectCentricEventLog> {
    let content = std::fs::read_to_string(path)?;
    let document = roxmltree::Document::parse(&content)?;

    let mut ocel = ObjectCentricEventLog::new();

    // Parse object types
    for node in document.descendants() {
        if node.tag_name().name() == "object-type" {
            let name = node.attribute("name").unwrap_or("unknown").to_string();
            let obj_type = crate::ocpm::ObjectType::new(&name);
            ocel.register_object_type(obj_type);
        }
    }

    // Parse objects
    for node in document.descendants() {
        if node.tag_name().name() == "object" {
            let id = node
                .attribute("id")
                .unwrap_or(&uuid::Uuid::new_v4().to_string())
                .to_string();
            let type_name = node.attribute("type").unwrap_or("unknown").to_string();

            let obj_type = crate::ocpm::ObjectType::new(&type_name);
            let object = crate::ocpm::Object::new(&id, obj_type, chrono::Utc::now());
            ocel.add_object(object);
        }
    }

    // Parse events
    for node in document.descendants() {
        if node.tag_name().name() == "event" {
            let default_id = uuid::Uuid::new_v4().to_string();
            let id_str = node.attribute("id").unwrap_or(&default_id);
            let event_id = uuid::Uuid::parse_str(id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());

            let activity = node.attribute("activity").unwrap_or("unknown").to_string();
            let timestamp = node
                .attribute("timestamp")
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now);

            let resource = node.attribute("resource").map(|r| r.to_string());

            ocel.add_event(event_id, &activity, timestamp, resource);

            // Parse object references
            for child in node.children() {
                if child.tag_name().name() == "object-ref" {
                    if let Some(obj_id) = child.attribute("id") {
                        if let Some(mapping) = ocel
                            .event_object_mappings
                            .iter_mut()
                            .find(|m| m.event_id == event_id)
                        {
                            mapping.add_object(obj_id);
                        } else {
                            let mut mapping = crate::ocpm::EventToObjectMapping::new(event_id);
                            mapping.add_object(obj_id);
                            ocel.add_event_object_mapping(mapping);
                        }
                    }
                }
            }
        }
    }

    Ok(ocel)
}

/// Read OCEL2 JSON format
///
/// Reads an OCEL2 file in JSON format.
pub fn read_ocel2_json(path: &Path) -> Result<ObjectCentricEventLog> {
    let content = std::fs::read_to_string(path)?;
    let data: serde_json::Value = serde_json::from_str(&content)?;

    let mut ocel = ObjectCentricEventLog::new();

    // Parse object types
    if let Some(types) = data.get("objectTypes").and_then(|t| t.as_array()) {
        for type_obj in types {
            if let Some(name) = type_obj.get("name").and_then(|n| n.as_str()) {
                let obj_type = crate::ocpm::ObjectType::new(name);
                ocel.register_object_type(obj_type);
            }
        }
    }

    // Parse objects
    if let Some(objects) = data.get("objects").and_then(|o| o.as_array()) {
        for obj in objects {
            let default_id = uuid::Uuid::new_v4().to_string();
            let id = obj
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or(&default_id);
            let type_name = obj
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");

            let obj_type = crate::ocpm::ObjectType::new(type_name);
            let object = crate::ocpm::Object::new(id, obj_type, chrono::Utc::now());
            ocel.add_object(object);
        }
    }

    // Parse events
    if let Some(events) = data.get("events").and_then(|e| e.as_array()) {
        for event in events {
            let default_id = uuid::Uuid::new_v4().to_string();
            let id_str = event
                .get("id")
                .and_then(|i| i.as_str())
                .unwrap_or(&default_id);
            let event_id = uuid::Uuid::parse_str(id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());

            let activity = event
                .get("activity")
                .and_then(|a| a.as_str())
                .unwrap_or("unknown")
                .to_string();

            let timestamp = event
                .get("timestamp")
                .and_then(|t| t.as_str())
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now);

            let resource = event
                .get("resource")
                .and_then(|r| r.as_str())
                .map(|r| r.to_string());

            ocel.add_event(event_id, &activity, timestamp, resource.clone());

            // Parse object references
            if let Some(obj_refs) = event.get("objects").and_then(|o| o.as_array()) {
                let mut mapping = crate::ocpm::EventToObjectMapping::new(event_id);
                for obj_ref in obj_refs {
                    if let Some(obj_id) = obj_ref.as_str() {
                        mapping.add_object(obj_id);
                    }
                }
                ocel.add_event_object_mapping(mapping);
            }
        }
    }

    Ok(ocel)
}

/// Read OCEL2 SQLite format
///
/// Reads an OCEL2 file from a SQLite database written by `write_ocel2_sqlite`.
/// Tables expected:
///   `ocel2_events(event_id TEXT, activity TEXT, timestamp TEXT)`
///   `ocel2_objects(object_id TEXT, object_type TEXT)`
///   `ocel2_event_object_map(event_id TEXT, object_id TEXT, qualifier TEXT)` — optional (graceful fallback for old DBs)
pub fn read_ocel2_sqlite(path: &Path) -> Result<ObjectCentricEventLog> {
    use rusqlite::Connection;

    let conn = Connection::open(path)?;
    let mut ocel = ObjectCentricEventLog::new();

    // Read objects
    {
        let mut stmt = conn.prepare("SELECT object_id, object_type FROM ocel2_objects")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (object_id, object_type_name) = row?;
            let obj_type = crate::ocpm::ObjectType::new(&object_type_name);
            let object = crate::ocpm::Object::new(&object_id, obj_type, chrono::Utc::now());
            ocel.add_object(object);
        }
    }

    // Read events
    {
        let mut stmt = conn.prepare("SELECT event_id, activity, timestamp FROM ocel2_events")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (event_id_str, activity, timestamp_str) = row?;
            let event_id =
                uuid::Uuid::parse_str(&event_id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
            let timestamp = chrono::DateTime::parse_from_rfc3339(&timestamp_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());
            ocel.add_event(event_id, &activity, timestamp, None);
        }
    }

    // Read event-object mappings — graceful fallback if table absent (old DBs)
    if table_exists(&conn, "ocel2_event_object_map") {
        use std::collections::HashMap;
        let mut mappings: HashMap<uuid::Uuid, crate::ocpm::EventToObjectMapping> = HashMap::new();

        let mut stmt = conn.prepare("SELECT event_id, object_id FROM ocel2_event_object_map")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        for row in rows {
            let (event_id_str, object_id) = row?;
            let event_id =
                uuid::Uuid::parse_str(&event_id_str).unwrap_or_else(|_| uuid::Uuid::new_v4());
            mappings
                .entry(event_id)
                .or_insert_with(|| crate::ocpm::EventToObjectMapping::new(event_id))
                .add_object(object_id);
        }

        for mapping in mappings.into_values() {
            ocel.add_event_object_mapping(mapping);
        }
    }

    Ok(ocel)
}

/// Check whether a table exists in the SQLite database.
fn table_exists(conn: &rusqlite::Connection, table_name: &str) -> bool {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
            rusqlite::params![table_name],
            |row| row.get(0),
        )
        .unwrap_or(0);
    count > 0
}

/// Write OCEL2 file (auto-detect format)
///
/// Writes an OCEL2 file, detecting format from extension.
pub fn write_ocel2(ocel: &ObjectCentricEventLog, path: &Path) -> Result<()> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("xml") => write_ocel2_xml(ocel, path),
        Some("json") => write_ocel2_json(ocel, path),
        Some("sqlite") | Some("db") => write_ocel2_sqlite(ocel, path),
        _ => Err(anyhow::anyhow!("Unsupported OCEL2 format")),
    }
}

/// Write OCEL2 XML format
///
/// Writes an OCEL2 file in XML format.
pub fn write_ocel2_xml(ocel: &ObjectCentricEventLog, path: &Path) -> Result<()> {
    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<ocel2>\n");

    // Write object types
    xml.push_str("  <object-types>\n");
    for obj_type in &ocel.object_types {
        xml.push_str(&format!("    <object-type name=\"{}\"/>\n", obj_type.name));
    }
    xml.push_str("  </object-types>\n");

    // Write objects
    xml.push_str("  <objects>\n");
    for object in ocel.objects.values() {
        xml.push_str(&format!(
            "    <object id=\"{}\" type=\"{}\"/>\n",
            object.id, object.object_type.name
        ));
    }
    xml.push_str("  </objects>\n");

    // Write events
    xml.push_str("  <events>\n");
    for (event_id, (activity, timestamp, resource)) in &ocel.events {
        xml.push_str(&format!(
            "    <event id=\"{}\" activity=\"{}\" timestamp=\"{}\"",
            event_id,
            activity,
            timestamp.to_rfc3339()
        ));
        if let Some(ref res) = resource {
            xml.push_str(&format!(" resource=\"{}\"", res));
        }
        xml.push_str(">\n");

        // Write object references
        if let Some(mapping) = ocel
            .event_object_mappings
            .iter()
            .find(|m| &m.event_id == event_id)
        {
            for obj_id in &mapping.object_ids {
                xml.push_str(&format!("      <object-ref id=\"{}\"/>\n", obj_id));
            }
        }

        xml.push_str("    </event>\n");
    }
    xml.push_str("  </events>\n");

    xml.push_str("</ocel2>\n");

    std::fs::write(path, xml)?;
    Ok(())
}

/// Write OCEL2 JSON format
///
/// Writes an OCEL2 file in JSON format.
pub fn write_ocel2_json(ocel: &ObjectCentricEventLog, path: &Path) -> Result<()> {
    let mut data = serde_json::Map::new();

    // Object types
    let types: Vec<serde_json::Value> = ocel
        .object_types
        .iter()
        .map(|t| serde_json::json!({"name": t.name}))
        .collect();
    data.insert("objectTypes".to_string(), serde_json::Value::Array(types));

    // Objects
    let objects: Vec<serde_json::Value> = ocel
        .objects
        .values()
        .map(|o| {
            serde_json::json!({
                "id": o.id,
                "type": o.object_type.name
            })
        })
        .collect();
    data.insert("objects".to_string(), serde_json::Value::Array(objects));

    // Events
    let events: Vec<serde_json::Value> = ocel
        .events
        .iter()
        .map(|(eid, (act, ts, res))| {
            let mut event_data = serde_json::Map::new();
            event_data.insert("id".to_string(), serde_json::Value::String(eid.to_string()));
            event_data.insert(
                "activity".to_string(),
                serde_json::Value::String(act.clone()),
            );
            event_data.insert(
                "timestamp".to_string(),
                serde_json::Value::String(ts.to_rfc3339()),
            );

            if let Some(ref r) = res {
                event_data.insert("resource".to_string(), serde_json::Value::String(r.clone()));
            }

            // Get object references
            let obj_refs: Vec<String> = ocel
                .event_object_mappings
                .iter()
                .filter(|m| &m.event_id == eid)
                .flat_map(|m| m.object_ids.iter().cloned())
                .collect();
            event_data.insert(
                "objects".to_string(),
                serde_json::Value::Array(
                    obj_refs
                        .into_iter()
                        .map(serde_json::Value::String)
                        .collect(),
                ),
            );

            serde_json::Value::Object(event_data)
        })
        .collect();
    data.insert("events".to_string(), serde_json::Value::Array(events));

    std::fs::write(path, serde_json::to_string_pretty(&data)?)?;
    Ok(())
}

/// Write OCEL2 SQLite format
///
/// Creates (or overwrites) a SQLite database at `path` with three tables:
///   `ocel2_events(event_id TEXT PRIMARY KEY, activity TEXT, timestamp TEXT)`
///   `ocel2_objects(object_id TEXT PRIMARY KEY, object_type TEXT)`
///   `ocel2_event_object_map(event_id TEXT, object_id TEXT, qualifier TEXT)`
pub fn write_ocel2_sqlite(ocel: &ObjectCentricEventLog, path: &Path) -> Result<()> {
    use rusqlite::{params, Connection};

    // Remove existing file so we start fresh (avoids primary-key conflicts
    // when the caller round-trips through write → read).
    if path.exists() {
        std::fs::remove_file(path)?;
    }

    let conn = Connection::open(path)?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS ocel2_events (
             event_id  TEXT PRIMARY KEY,
             activity  TEXT NOT NULL,
             timestamp TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS ocel2_objects (
             object_id   TEXT PRIMARY KEY,
             object_type TEXT NOT NULL
         );
         CREATE TABLE IF NOT EXISTS ocel2_event_object_map (
             event_id   TEXT NOT NULL,
             object_id  TEXT NOT NULL,
             qualifier  TEXT NOT NULL DEFAULT '',
             PRIMARY KEY (event_id, object_id)
         );",
    )?;

    // Insert events
    {
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO ocel2_events (event_id, activity, timestamp) \
             VALUES (?1, ?2, ?3)",
        )?;
        for (event_id, (activity, timestamp, _resource)) in &ocel.events {
            stmt.execute(params![
                event_id.to_string(),
                activity,
                timestamp.to_rfc3339(),
            ])?;
        }
    }

    // Insert objects
    {
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO ocel2_objects (object_id, object_type) \
             VALUES (?1, ?2)",
        )?;
        for object in ocel.objects.values() {
            stmt.execute(params![object.id, object.object_type.name])?;
        }
    }

    // Insert event-object mappings (G3 fix: previously LOST on round-trip)
    {
        let mut stmt = conn.prepare(
            "INSERT OR REPLACE INTO ocel2_event_object_map (event_id, object_id, qualifier) \
             VALUES (?1, ?2, ?3)",
        )?;
        for mapping in &ocel.event_object_mappings {
            for obj_id in &mapping.object_ids {
                let qualifier = mapping
                    .object_roles
                    .get(obj_id)
                    .map(|s| s.as_str())
                    .unwrap_or("");
                stmt.execute(params![mapping.event_id.to_string(), obj_id, qualifier])?;
            }
        }
    }

    Ok(())
}
